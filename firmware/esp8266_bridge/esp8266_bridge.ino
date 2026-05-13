#include <ESP8266WiFi.h>
#include <WiFiUdp.h>

// --- Network Configuration ---
const char *ssid = "Rover_Radar";
const char *password = "slam_rover123";
const int udpPort = 4210;

// SoftAP default IP is usually 192.168.4.1. 
// We broadcast to .255 so any connected laptop receives it without needing a hardcoded IP.
IPAddress broadcastIP(192, 168, 4, 255);
WiFiUDP udp;

// --- Serial Buffer ---
const int MAX_PACKET_SIZE = 128;
char packetBuffer[MAX_PACKET_SIZE];
int bufferIndex = 0;

void setup() {
  // Must match the Arduino Nano's baud rate exactly
  Serial.begin(115200);
  
  // Create the standalone Wi-Fi Network
  WiFi.softAP(ssid, password);
  
  // Start the UDP protocol
  udp.begin(udpPort);
}

void loop() {
  // Read incoming data from the Arduino Nano
  while (Serial.available() > 0) {
    char incomingChar = Serial.read();

    // If we hit a newline, the payload is complete. Send it!
    if (incomingChar == '\n') {
      packetBuffer[bufferIndex] = '\0'; // Null-terminate the string
      
      // Broadcast over UDP
      udp.beginPacket(broadcastIP, udpPort);
      udp.write(packetBuffer);
      udp.endPacket();
      
      // Reset buffer for the next reading
      bufferIndex = 0;
    } 
    // Otherwise, keep adding to the buffer
    else if (bufferIndex < MAX_PACKET_SIZE - 1) {
      packetBuffer[bufferIndex] = incomingChar;
    }
  }
}
