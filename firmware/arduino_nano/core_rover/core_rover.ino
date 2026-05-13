#include <Wire.h>

// --- Pin Definitions ---
const int trigPin = 5;
const int echoPin = 2; // Must be Pin 2 or 3 for hardware interrupts (INT0)

// --- MPU6050 I2C Definitions ---
const int MPU_ADDR = 0x68; // Default I2C address for MPU6050
float gyroZ_Offset = 0.0;
float currentHeading = 0.0;
unsigned long lastGyroTime = 0;

// --- HC-SR04 Interrupt Variables ---
volatile unsigned long echoStart = 0;
volatile unsigned long echoEnd = 0;
volatile bool newDistanceReady = false;
float currentDistance = 0.0;

// --- Odometry/State Variables ---
long stepX = 0; // Wheel steps in X direction
long stepY = 0; // Wheel steps in Y direction
float sensorAngle = 0.0; // Current angle of the scanning motor
unsigned long lastPingTime = 0;

// ==========================================
// INTERRUPT SERVICE ROUTINE (ISR)
// ==========================================
void echoISR() {
  if (digitalRead(echoPin) == HIGH) {
    echoStart = micros(); // Rising edge: Ping just sent
  } else {
    echoEnd = micros();   // Falling edge: Ping returned
    newDistanceReady = true;
  }
}

void setup() {
  Serial.begin(115200);
  
  // 1. Setup HC-SR04
  pinMode(trigPin, OUTPUT);
  pinMode(echoPin, INPUT);
  digitalWrite(trigPin, LOW);
  
  // Attach interrupt to the ECHO pin. Trigger on CHANGE (both HIGH and LOW edges)
  attachInterrupt(digitalPinToInterrupt(echoPin), echoISR, CHANGE);

  // 2. Setup MPU6050 via Raw I2C
  Wire.begin();
  
  // Wake up MPU6050 (Write 0 to Power Management 1 register 0x6B)
  Wire.beginTransmission(MPU_ADDR);
  Wire.write(0x6B); 
  Wire.write(0x00); 
  Wire.endTransmission(true);

  // Configure Gyro Sensitivity (Write 0x08 to Gyro Config register 0x1B for ±500 deg/s)
  Wire.beginTransmission(MPU_ADDR);
  Wire.write(0x1B);
  Wire.write(0x08);
  Wire.endTransmission(true);

  // Calculate Gyro Z Error (Offset) - Keep sensor completely still during boot!
  long gyroZ_Sum = 0;
  for(int i = 0; i < 500; i++) {
    Wire.beginTransmission(MPU_ADDR);
    Wire.write(0x47); // Start reading at GYRO_ZOUT_H
    Wire.endTransmission(false);
    Wire.requestFrom(MPU_ADDR, 2, true);
    int16_t rawZ = Wire.read() << 8 | Wire.read();
    gyroZ_Sum += rawZ;
    delay(3);
  }
  gyroZ_Offset = gyroZ_Sum / 500.0;
  
  lastGyroTime = micros();
}

void loop() {
  unsigned long currentMicros = micros();

  // ==========================================
  // 1. READ MPU6050 & INTEGRATE HEADING
  // ==========================================
  // Request 2 bytes starting from GYRO_ZOUT_H (0x47)
  Wire.beginTransmission(MPU_ADDR);
  Wire.write(0x47);
  Wire.endTransmission(false);
  Wire.requestFrom(MPU_ADDR, 2, true);
  
  int16_t rawGyroZ = Wire.read() << 8 | Wire.read();
  
  // Convert raw value to Degrees Per Second (Divider 65.5 for ±500 deg/s scale)
  float gyroRateZ = (rawGyroZ - gyroZ_Offset) / 65.5;
  
  // Calculate Delta Time in seconds
  float dt = (currentMicros - lastGyroTime) / 1000000.0;
  lastGyroTime = currentMicros;

  // Integrate to get Yaw (Heading)
  // Ignore tiny noise fluctuations below 1 deg/s
  if (abs(gyroRateZ) > 1.0) { 
    currentHeading += gyroRateZ * dt;
  }

  // ==========================================
  // 2. FIRE SENSOR (Non-Blocking)
  // ==========================================
  // Fire a ping every 50ms
  if (millis() - lastPingTime >= 50) {
    digitalWrite(trigPin, HIGH);
    delayMicroseconds(10); // 10us is acceptable here, won't ruin loops
    digitalWrite(trigPin, LOW);
    lastPingTime = millis();
  }

  // ==========================================
  // 3. PROCESS INTERRUPT & SEND TELEMETRY
  // ==========================================
  if (newDistanceReady) {
    // Calculate distance mathematically
    unsigned long duration = echoEnd - echoStart;
    currentDistance = (duration * 0.0343) / 2.0;
    
    // Transmit Payload to ESP8266: step_x, step_y, heading, sensor_angle, distance
    Serial.print(stepX);
    Serial.print(",");
    Serial.print(stepY);
    Serial.print(",");
    Serial.print(currentHeading);
    Serial.print(",");
    Serial.print(sensorAngle);
    Serial.print(",");
    Serial.println(currentDistance);
    
    newDistanceReady = false; // Reset flag
  }
  
  // Note: Non-blocking stepper movement logic will be implemented in future.
}
