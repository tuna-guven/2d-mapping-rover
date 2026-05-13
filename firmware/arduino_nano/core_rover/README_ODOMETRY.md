# Rover Odometry & Sensor Fusion

For the robot to autonomously map its environment using SLAM (Simultaneous Localization and Mapping), it must constantly calculate its exact position and orientation in the room. This process is called **Odometry**. 

Because wheel slip and mechanical imperfections cause pure motor-based calculations to drift over time, we use a technique called **Sensor Fusion**, combining two data sources to calculate the robot's pose:

## 1. Dead Reckoning (Stepper Motors)
Unlike standard DC motors, our Nema 17 stepper motors move in exact, discrete increments ($1.8^\circ$ per step). By counting the exact number of pulses sent to the left and right wheels, we can calculate the linear distance traveled:
* If both wheels step forward, the robot moves precisely along its local X-axis.
* The local coordinate displacement is tracked using `step_x` and `step_y` counters.

## 2. Inertial Navigation (MPU6050 Gyroscope)
When the robot turns, differential drive math (calculating rotation based on wheel steps) is highly vulnerable to wheel slip. To fix this, we read the Z-axis of the MPU6050 gyroscope using raw I2C register calls. 
* The gyroscope measures the rate of rotation (degrees per second). 
* By integrating this angular velocity over time ($\Delta t$), we calculate the robot's exact global **Heading (Yaw)**.

## The Telemetry Payload
The Arduino Nano runs a non-blocking loop, fusing these inputs and the ultrasonic ping data, transmitting a continuous CSV string over UART to the ESP8266 bridge:
`step_x, step_y, heading, sensor_angle, distance`

This guarantees the Rust host application always knows exactly where the ping originated globally.
