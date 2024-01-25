# Utilities for M5Stack Cardputer

## Features

* Initialize ST7789 driver
* LCD backlight control
* Decode 74HC138 and convert to keycode
* Initialize I2C driver for Grove I/F

## Usage

### Dependencies:

```
[dependencies]
cardputer = "0.1"
```

### Code:

```rust
use cardputer::keyboard::Keyboard;

let peripherals = Peripherals::take().unwrap();

let mut keyboard = Keyboard::new(
    peripherals.pins.gpio8,
    peripherals.pins.gpio9,
    peripherals.pins.gpio11,
    peripherals.pins.gpio13,
    peripherals.pins.gpio15,
    peripherals.pins.gpio3,
    peripherals.pins.gpio4,
    peripherals.pins.gpio5,
    peripherals.pins.gpio6,
    peripherals.pins.gpio7,
)
.unwrap();
let keys = keyboard.scan_pressed_keys().unwrap();
```

## Examples

Simple example that just outputs the pressed keys to log:info


```sh
% cargo run --example key_monitor
  :
I (2642) key_monitor: [Q]
I (3142) key_monitor: [W]
I (3642) key_monitor: [E]
I (4142) key_monitor: [R]
I (4642) key_monitor: [T]
I (5142) key_monitor: [Y]
I (5642) key_monitor: [Space]
  :
```
