# Pressure Recorder

Command-line utility for recording inlet and outlet pressure data from pressure transducers using a Raspberry Pi and an ADS1115.

## Equipment

You will need 2 pressure transducers with a range of 0 to 10 psi, a Raspberry Pi, and an ADS1115 analog-to-digital converter.
While any Pi will work, a Zero, Zero W, or Zero 2 W are recommended as they are the cheapest option.

## Setup

Connect the ADS1115 to the one of the 3.3v and ground pins on the raspberry pi, then the SCL (clock) and SDA (data) pins should be connected 
to the corresponding i2c1 pins on the Raspberry Pi, which are GPIO 3 and GPIO 2 for the Raspberry Pi Zero models. The inlet and outlet transducers
should be connected to the A2 and A3 channels of the ADS1115, respectively; their analog outputs should be the green wire. Finally, use a 5V pin 
on the pi to power ***both*** pressure transudcers and connect them back to ground (black), as using both will result in 5V being run into the
ADS1115, which is more than it can safely handle as its only powered by 3.3V.

## Running the code

Pre-compiled source code for the Raspberry Pi Zero (W) can be found under /target/arm-unknown-linux-gnueabihf/debug, file name `pressure_transducer` 
(with no file extension). This can be moved to the Raspberry Pi and ran with `chmod +x pressure_transducer && ./pressure_transducer` in the command
line.

## Commands

`time` Sets the total time to record for in seconds. For example, `time 10` will set the program to record for 10 seconds.

`interval` Sets the recording interval in seconds. For example, `interval 0.1` will set the program to record every 0.1 seconds.

`multiplier` Sets the value to multiply the voltage by; use when the transducers are powered with more or less than 4.5V. For example, 
            `multiplier 1.8` will set the recorded voltages from the transducers to be multiplied by 1.8, allowing the correct pressure
            to be recorded if the transducers are each being powered with only 2.5 volts. This is needed as reducing the input voltage
            does not reduce the pressure range of the transducers but the output pressure at each voltage.
            
`start` starts recording. After recording, a `plot.png` graphing the data and a `data.xlsx` file containing the data will be produces.

`exit` Quits the program.

## Modifying and rebuilding

In order to modify and/or build the source code, you will need to install rust, which you can do [here](https://www.rust-lang.org/tools/install).
You will also need to use `cross` to build it for any Raspberry pi or similar computer, which you can learn about [here](https://kerkour.com/rust-cross-compilation)
