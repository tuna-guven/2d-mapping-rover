#define STEP_PIN 3
#define DIR_PIN 4
#define EN_PIN 7
#define TRIG_PIN 9
#define ECHO_PIN 2 // D2 Pini (Donanımsal Kesme)

volatile unsigned long echo_start = 0;
volatile unsigned long echo_end = 0;
volatile bool new_distance_ready = false;

int current_step = 0;
int max_steps = 100; // 180 derece tarama (1.8 deg/step)
bool moving_forward = true;

unsigned long last_step_time = 0;
// RADAR İÇİN SABİT HIZ (Değiştirilmez)
const int step_interval_us = 3000; 

unsigned long last_ping_time = 0;
const int ping_interval_ms = 50; // 20Hz Ping Rate

void setup() {
  Serial.begin(115200);

  pinMode(STEP_PIN, OUTPUT);
  pinMode(DIR_PIN, OUTPUT);
  pinMode(EN_PIN, OUTPUT);
  digitalWrite(EN_PIN, LOW); // Sürücüyü uyandır

  pinMode(TRIG_PIN, OUTPUT);
  pinMode(ECHO_PIN, INPUT);

  attachInterrupt(digitalPinToInterrupt(ECHO_PIN), echo_isr, CHANGE);
  delay(1000); 
}

void loop() {
  unsigned long current_micros = micros();
  unsigned long current_millis = millis();

  // --- 1. KASLAR: Sabit Hızda Motor Kontrolü ---
  if (current_micros - last_step_time >= step_interval_us) {
    last_step_time = current_micros;
    
    digitalWrite(DIR_PIN, moving_forward ? HIGH : LOW);
    digitalWrite(STEP_PIN, HIGH);
    delayMicroseconds(2);
    digitalWrite(STEP_PIN, LOW);

    if (moving_forward) {
      current_step++;
      if (current_step >= max_steps) moving_forward = false;
    } else {
      current_step--;
      if (current_step <= 0) moving_forward = true;
    }
  }

  // --- 2. GÖZLER: Sensörü Tetikle ---
  if (current_millis - last_ping_time >= ping_interval_ms) {
    last_ping_time = current_millis;
    digitalWrite(TRIG_PIN, LOW);
    delayMicroseconds(2);
    digitalWrite(TRIG_PIN, HIGH);
    delayMicroseconds(10);
    digitalWrite(TRIG_PIN, LOW);
  }

  // --- 3. BEYİN: Veriyi Rust'a Gönder ---
  if (new_distance_ready) {
    new_distance_ready = false;
    
    unsigned long duration = echo_end - echo_start;
    float distance_cm = (duration * 0.0343) / 2.0;

    // Sadece mantıklı ölçümleri kabul et (Gürültü filtresi)
    if (distance_cm > 2.0 && distance_cm < 400.0) {
      float scan_angle = (float)current_step * 1.8; 

      Serial.print("0.0, 0.0, 0.0, ");
      Serial.print(scan_angle);
      Serial.print(", ");
      Serial.println(distance_cm);
    }
  }
}

// --- DONANIMSAL KESME (ISR) ---
void echo_isr() {
  if (digitalRead(ECHO_PIN) == HIGH) {
    echo_start = micros(); 
  } else {
    echo_end = micros();   
    new_distance_ready = true;
  }
}