# ESP8266 Wi-Fi Telemetry Bridge

This module acts as the wireless backbone for the SLAM rover. The Arduino Nano is too busy handling precise motor timing and interrupts to manage a network stack. Therefore, the Nano streams its data over a hardwired UART connection to this ESP8266.

## Network Architecture (SoftAP)
Instead of relying on a local home or university Wi-Fi network (which might block peer-to-peer UDP traffic), the ESP8266 is configured as a **Software Access Point (SoftAP)**. 
* **SSID:** `Rover_Radar`
* **Password:** `slam_rover123`
* **IP Range:** The ESP8266 defaults to `192.168.4.1`.

The host PC connects directly to the rover's Wi-Fi network. 

## The UDP Pipeline
1. **Ingestion:** The ESP8266 listens to its hardware `Serial` RX pin at 115200 baud for incoming strings from the Nano.
2. **Buffering:** It buffers the characters until it reads a newline (`\n`), ensuring packets are not fragmented.
3. **Broadcast:** It immediately blasts the assembled string over **UDP Port 4210** to the broadcast address (`192.168.4.255`), meaning any device connected to the rover's network will receive the telemetry stream.
