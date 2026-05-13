
## LED vs. Stepper Motor (Notes for the Real Hardware)

When we transition from this Tinkercad LED to our physical Nema 17 stepper motor, here is exactly how that behavior translates.

**1. The Intermediary (A4988 Driver)**
We cannot plug a stepper motor directly into an Arduino. The motor draws way too much power. Instead, Pin 3 will connect to the `STEP` pin on our **A4988 Motor Driver**, and the driver connects to the motor. The Arduino is the brain, the A4988 is the muscle.

**2. How the Pulses Translate**

* **In Tinkercad:** Every time Pin 3 goes `HIGH`, the LED turns on. Every time it goes `LOW`, it turns off. A rapid HIGH/LOW sequence looks like a blinking light.
* **In Real Life:** Every time Pin 3 pulses `HIGH` and then `LOW`, the A4988 driver interprets that as a command to move the motor by **exactly one step**.
* For a standard Nema 17, one step equals **1.8 degrees** of rotation.

**3. Speed and Motion**

* **Blink Rate = Motor Speed:** If our code makes the LED blink slowly, the stepper motor will tick forward slowly, one step at a time. If we make the LED blink incredibly fast (like in our non-blocking code), the individual 1.8-degree steps blur together into a **smooth, continuous rotation**.
* **Direction:** Pin 4 from our code connects to the `DIR` (Direction) pin on the A4988. If Pin 4 is `HIGH`, the motor spins clockwise. If Pin 4 is `LOW`, it spins counter-clockwise.

So, when we test this in Tinkercad, a steady, uninterrupted blinking LED proves that our physical motor will spin smoothly without stuttering while the sensor takes its readings!
