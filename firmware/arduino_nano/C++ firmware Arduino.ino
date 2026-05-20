#define STEP_PIN 3
#define DIR_PIN 4
#define EN_PIN 7
#define TRIG_PIN 9
#define ECHO_PIN 2 // D2 Pini (Donanımsal Kesme - Interrupt 0)

// Sensör Değişkenleri (Interrupt içinde kullanılacağı için 'volatile' olmalı)
volatile unsigned long echo_start = 0;
volatile unsigned long echo_end = 0;
volatile bool new_distance_ready = false;

// Motor ve Tarama Değişkenleri
int current_step = 0;
int max_steps = 100; // 100 adım = 180 derece (1.8 deg/step Nema 17 için)
bool moving_forward = true;

unsigned long last_step_time = 0;
int step_interval_us = 3000; // Motor hızı (mikrosaniye) - Titrememesi için biraz yumuşattım

unsigned long last_ping_time = 0;
int ping_interval_ms = 50; // Saniyede 20 kere ping atar (20Hz)

void setup() {
  Serial.begin(115200);

  pinMode(STEP_PIN, OUTPUT);
  pinMode(DIR_PIN, OUTPUT);
  pinMode(EN_PIN, OUTPUT);
  digitalWrite(EN_PIN, LOW); // A4988'i uyandır

  pinMode(TRIG_PIN, OUTPUT);
  pinMode(ECHO_PIN, INPUT);

  // SİHİRLİ DOKUNUŞ: Donanımsal Kesme (Interrupt)
  // ECHO pini her değiştiğinde (HIGH veya LOW) echo_isr fonksiyonunu tetikle
  attachInterrupt(digitalPinToInterrupt(ECHO_PIN), echo_isr, CHANGE);

  // İlk kalibrasyon için bekleme
  delay(1000); 
}

void loop() {
  unsigned long current_micros = micros();
  unsigned long current_millis = millis();

  // --- 1. KASLAR: Motoru Hareket Ettir (Non-blocking) ---
  if (current_micros - last_step_time >= step_interval_us) {
    last_step_time = current_micros;
    
    digitalWrite(DIR_PIN, moving_forward ? HIGH : LOW);
    
    // Adım at
    digitalWrite(STEP_PIN, HIGH);
    delayMicroseconds(2); // Çok kısa bir tetikleme, sistemi kilitlemez
    digitalWrite(STEP_PIN, LOW);

    // Açıyı hesaplamak için pozisyonu takip et
    if (moving_forward) {
      current_step++;
      if (current_step >= max_steps) moving_forward = false;
    } else {
      current_step--;
      if (current_step <= 0) moving_forward = true;
    }
  }

  // --- 2. GÖZLER: Sensörü Tetikle (Non-blocking) ---
  if (current_millis - last_ping_time >= ping_interval_ms) {
    last_ping_time = current_millis;
    digitalWrite(TRIG_PIN, LOW);
    delayMicroseconds(2);
    digitalWrite(TRIG_PIN, HIGH);
    delayMicroseconds(10);
    digitalWrite(TRIG_PIN, LOW);
  }

  // --- 3. BEYİN: Veri Geldiğinde İşle ve Rust'a Gönder ---
  if (new_distance_ready) {
    new_distance_ready = false;
    
    unsigned long duration = echo_end - echo_start;
    float distance_cm = (duration * 0.0343) / 2.0;

    // Sensör gürültüsünü filtrele (Örn: 2cm altı ve 400cm üstü hatalıdır)
    if (distance_cm > 2.0 && distance_cm < 400.0) {
      
      // --- EKSİK OLAN HIZ KONTROLÜ (DYNAMIC SPEED) ---
      // Mesafeyi 5cm ile 50cm arasına sıkıştır
      float limitedDistance = constrain(distance_cm, 5.0, 50.0);
      // Yakınsa 400 mikrosaniye (Hızlı), Uzaksa 3000 mikrosaniye (Yavaş)
      step_interval_us = map(limitedDistance, 5, 50, 400, 3000);
      // -----------------------------------------------

      // 0-100 adımı, 0-180 dereceye çevir
      float scan_angle = (float)current_step * 1.8; 

      // Rust'ın doğrudan parse edebileceği CSV formatında yazdır
      Serial.print("0.0, 0.0, 0.0, ");
      Serial.print(scan_angle);
      Serial.print(", ");
      Serial.println(distance_cm);
    }
  }
}

// --- DONANIMSAL KESME RUTİNİ (ISR) ---
// CPU'dan bağımsız arka planda çalışır
void echo_isr() {
  if (digitalRead(ECHO_PIN) == HIGH) {
    echo_start = micros(); // Ses dalgası çıktı
  } else {
    echo_end = micros();   // Ses dalgası geri döndü
    new_distance_ready = true;
  }
}