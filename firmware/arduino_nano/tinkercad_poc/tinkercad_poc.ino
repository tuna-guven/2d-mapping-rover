// --- Pin Definitions ---
const int stepPin = 3;   // Connect to A4988 STEP
const int dirPin = 4;    // Connect to A4988 DIR
const int trigPin = 5;   // Connect to HC-SR04 TRIG
const int echoPin = 2;   // Connect to HC-SR04 ECHO (Supports INT0)

// --- Motor State Variables ---
unsigned long previousStepMicros = 0;
// 2000 microseconds = 2 milliseconds between steps. 
// Adjust this to change motor speed.
const unsigned long stepInterval = 2000; 

// --- Sensor State Variables ---
unsigned long previousPingMillis = 0;
const unsigned long pingInterval = 50; // Fire a ping every 50ms
bool isTriggering = false;
unsigned long triggerStartMicros = 0;

void setup() {
  Serial.begin(115200);
  
  pinMode(stepPin, OUTPUT);
  pinMode(dirPin, OUTPUT);
  pinMode(trigPin, OUTPUT);
  pinMode(echoPin, INPUT);
  
  // Set initial motor direction
  digitalWrite(dirPin, HIGH); 
  
  // Ensure TRIG is low to start
  digitalWrite(trigPin, LOW);
  
  Serial.println("System Initialized. Starting non-blocking sweep...");
}

void loop() {
  unsigned long currentMicros = micros();
  unsigned long currentMillis = millis();

  // ==========================================
  // 1. NON-BLOCKING STEPPER CONTROL (VISUAL TEST)
  // ==========================================
  // Slowed down to 250,000 microseconds (0.25 seconds) so you can see it
  if (currentMicros - previousStepMicros >= 250000) { 
    previousStepMicros = currentMicros;
    
    // Read the current state of the LED and flip it
    int currentState = digitalRead(stepPin);
    digitalWrite(stepPin, !currentState); 
  }

  // ==========================================
  // 2. NON-BLOCKING HC-SR04 TRIGGER SEQUENCE
  // ==========================================
  if (!isTriggering && (currentMillis - previousPingMillis >= pingInterval)) {
    digitalWrite(trigPin, HIGH);
    triggerStartMicros = currentMicros;
    isTriggering = true;
  }

  if (isTriggering && (currentMicros - triggerStartMicros >= 10)) {
    digitalWrite(trigPin, LOW); 
    isTriggering = false;
    previousPingMillis = currentMillis; 
  }
}
