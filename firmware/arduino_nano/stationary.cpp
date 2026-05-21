#define F_CPU 16000000UL

#include <avr/io.h>
#include <avr/interrupt.h>
#include <util/delay.h>
#include <stdlib.h>

// --- System Configuration ---
#define BAUD 38400
#define BRC ((F_CPU / 16 / BAUD) - 1)

// Timer1 Prescaler is 64 -> 1 Tick = 4 microseconds.
#define STEP_INTERVAL_TICKS 1000    // 2000 us per step (Smooth radar sweep speed)
#define PING_INTERVAL_TICKS 10000  // 40 ms between sensor pings
#define SENSOR_TIMEOUT_TICKS 15000 // 60 ms timeout

// --- Globals (Volatile for ISR) ---
volatile uint16_t echo_start = 0;
volatile uint16_t echo_end = 0;
volatile uint8_t new_distance_ready = 0;

// --- State Machine & Logic Variables ---
enum MotorState { MOTOR_WAITING, MOTOR_PULSING };
enum SensorState { SENSOR_IDLE, SENSOR_WAITING_ECHO };

MotorState motor_state = MOTOR_WAITING;
SensorState sensor_state = SENSOR_IDLE;

// Motor tracking: 0 to 200 steps (360 degrees for a 1.8 deg/step motor)
int current_step = 100; // Start at center (0 degrees)
const int max_steps = 200;
uint8_t moving_forward = 1;

uint16_t last_step_ticks = 0;
uint16_t last_ping_ticks = 0;

// --- Initialization Functions ---
void init_hardware(void) {
    // 1. Pins Configuration
    DDRD |= (1 << DDD3) | (1 << DDD4) | (1 << DDD7); // Motor Pins
    PORTD &= ~(1 << PORTD7);                         // Enable Driver
    DDRD &= ~(1 << DDD2);                            // ECHO as INPUT
    DDRB |= (1 << DDB1);                             // TRIG as OUTPUT
    PORTB &= ~(1 << PORTB1); 

    // 2. Timer1 Configuration (Master Clock)
    TCCR1A = 0;
    TCCR1B = (1 << CS11) | (1 << CS10); // Normal mode, Prescaler 64
    TCNT1 = 0;

    // 3. External Interrupt 0 Configuration (PD2 / ECHO)
    EICRA |= (1 << ISC00);
    EICRA &= ~(1 << ISC01);
    EIMSK |= (1 << INT0);

    // 4. UART Configuration
    UBRR0H = (BRC >> 8);
    UBRR0L = BRC;
    UCSR0B = (1 << TXEN0); 
    UCSR0C = (1 << UCSZ01) | (1 << UCSZ00); 
}

// --- UART Transmit Helpers ---
void uart_tx_char(char data) {
    while (!(UCSR0A & (1 << UDRE0)));
    UDR0 = data;
}

void uart_tx_string(const char* str) {
    while (*str) uart_tx_char(*str++);
}

void uart_tx_float(float val) {
    char buffer[10];
    dtostrf(val, 4, 2, buffer);
    uart_tx_string(buffer);
}

// --- Update FSM Functions ---
void update_motor(uint16_t current_ticks) {
    if (motor_state == MOTOR_WAITING) {
        if ((uint16_t)(current_ticks - last_step_ticks) >= STEP_INTERVAL_TICKS) {
            last_step_ticks = current_ticks;
            
            if (moving_forward) {
                PORTD |= (1 << PORTD4);
            } else {
                PORTD &= ~(1 << PORTD4);
            }
            
            PORTD |= (1 << PORTD3); // STEP HIGH
            motor_state = MOTOR_PULSING;
        }
    } else if (motor_state == MOTOR_PULSING) {
        __builtin_avr_delay_cycles(32); // 2us pulse
        
        PORTD &= ~(1 << PORTD3); // STEP LOW
        
        // Update limits: Sweep between 0 and 200 steps
        if (moving_forward) {
            current_step++;
            if (current_step >= max_steps) moving_forward = 0;
        } else {
            current_step--;
            if (current_step <= 0) moving_forward = 1;
        }
        
        motor_state = MOTOR_WAITING;
    }
}

void update_sensor(uint16_t current_ticks) {
    if (sensor_state == SENSOR_IDLE) {
        if ((uint16_t)(current_ticks - last_ping_ticks) >= PING_INTERVAL_TICKS) {
            last_ping_ticks = current_ticks;
            
            PORTB |= (1 << PORTB1);
            _delay_us(10);
            PORTB &= ~(1 << PORTB1);
            
            sensor_state = SENSOR_WAITING_ECHO;
        }
    } else if (sensor_state == SENSOR_WAITING_ECHO) {
        if ((uint16_t)(current_ticks - last_ping_ticks) > SENSOR_TIMEOUT_TICKS) {
            sensor_state = SENSOR_IDLE; // Timeout safely
        }
    }
}

void process_and_transmit_data(void) {
    if (new_distance_ready) {
        uint16_t start = echo_start;
        uint16_t end = echo_end;
        new_distance_ready = 0; 
        sensor_state = SENSOR_IDLE;
        
        uint16_t duration_ticks = end - start;
        float duration_us = (float)duration_ticks * 4.0;
        float distance_cm = (duration_us * 0.0343) / 2.0;
        
        // Map 0 -> 200 steps to -180.0 -> +180.0 degrees
        float current_angle = ((float)current_step * 1.8) - 180.0;
        
        // Filter out bad sensor readings
        if (distance_cm > 2.0 && distance_cm < 400.0) {
            // Strict output formatting: Angle,Distance\n
            uart_tx_float(current_angle);
            uart_tx_char(',');
            uart_tx_float(distance_cm);
            uart_tx_char('\n');
        }
    }
}

// --- Interrupt Service Routine ---
ISR(INT0_vect) {
    uint16_t current_ticks = TCNT1;
    if (PIND & (1 << PIND2)) {
        echo_start = current_ticks;
    } else {
        echo_end = current_ticks;
        new_distance_ready = 1;
    }
}

// --- Main Loop ---
int main(void) {
    init_hardware();
    sei();
    
    while (1) {
        uint16_t current_ticks = TCNT1;
        
        update_motor(current_ticks);
        update_sensor(current_ticks);
        process_and_transmit_data();
    }
    return 0;
}
