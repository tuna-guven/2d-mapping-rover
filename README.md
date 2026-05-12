# 2D Topographical Radar/Scanner

This repository contains the source code, hardware specifications, and documentation for a 2D Topographical Radar/Scanner. This project was developed by Group 10 for the Embedded Systems Course Project at Muğla Sıtkı Koçman University.

## Project Overview

The core objective is to design an autonomous unit that sweeps an environment to capture distance data and generates a live 2D Cartesian map on a host PC. 

The system relies on a software-driven architecture, prioritizing sophisticated software algorithms and data processing over complex mechanical assemblies. By utilizing digital signal processing, the system is designed to handle physical inaccuracies such as multipath acoustic interference. This is a full-stack embedded systems project that covers hardware interfacing, firmware logic, serial communication protocols, and host-side UI rendering.

## Hardware Architecture

The hardware stack centers around an autonomous scanning unit equipped with an ultrasonic sensor and a central microcontroller. 

* **Microcontroller (Brain):** An Arduino Nano operates at 16MHz to handle real-time peripheral control.
* **Actuator:** A Nema 17 stepper motor paired with an A4988 driver. The microcontroller interfaces with the driver via STEP and DIR pins. The motor requires 200 steps to complete a full 360-degree rotation, translating to 1.8 degrees per step.
* **Sensor:** An HC-SR04 Ultrasonic sensor is mounted directly to the motor shaft. It emits a 40kHz acoustic pulse to time the reflection and interfaces with the microcontroller via TRIG (Output) and ECHO (Input) pins.

## Firmware Implementation

The firmware is written in non-blocking C++ to ensure real-time responsiveness.

* **Hardware Timers:** Standard `delay()` functions pause the CPU, which causes motor stuttering and missed sensor readings. Instead, the firmware utilizes hardware timers (Timer1) or `millis()` to continuously pulse the A4988 driver in the background.
* **Interrupt-Driven Sensing:** * A 10µs pulse is sent to the TRIG pin.
  * An external interrupt (e.g., INT0) is attached to the ECHO pin.
  * A timer starts on the rising edge of the echo and stops on the falling edge.
* **Distance Calculation:** Distance is calculated using the formula: d = (Time * 0.0343) / 2.
* **Telemetry:** Data is transmitted to the host via UART at a 115200 baud rate, formatted exactly as `angle, distance\n`.

## Data Pipeline & Coordinate Mathematics

The HC-SR04 sensor outputs data in Polar Coordinates (r, theta). To render this data on a 2D screen, the host application converts the polar data into Cartesian Coordinates (x, y) relative to the scanner's center (0,0).

The conversion relies on the following mathematical operations:
1.  Convert the stepper motor's degree angle to radians: Radians = theta * (pi / 180).
2.  Calculate the X coordinate: x = r * cos(Radians).
3.  Calculate the Y coordinate: y = r * sin(Radians).

## Host Application

The host application is developed in Python. 

* **Data Ingestion:** The script uses the `pyserial` library to parse the high-speed CSV telemetry stream coming from the microcontroller.
* **Digital Filtering:** Ultrasonic sensors often report false spikes, or "ghosts," due to acoustic reflections off angled walls. To mitigate this, the firmware takes three rapid readings per step angle. The Python script then applies a median filter by sorting the array of readings and selecting the middle value, which mathematically eliminates outlier spikes without skewing the underlying data.
* **Rendering:** The processed (x, y) coordinates are plotted in real-time utilizing either `matplotlib` or `pygame`.

## Advanced Features Roadmap

To elevate the system capabilities beyond basic point-cloud mapping, the following features are planned for implementation:

* **Object Clustering (DBSCAN):** An algorithm to group tightly packed coordinate points together, allowing the system to identify solid objects rather than a scattered point cloud.
* **Bounding Boxes:** Upon identifying a valid cluster, the software will calculate the minimum and maximum Cartesian bounds and render a distinct box around the obstacle.
* **Sweep Fading:** Older data points will slowly fade in opacity on the user interface. This ensures that moving objects leave a visual trail while preventing permanent screen clutter.
