//! Simple example of keyboard scanning
//!
//! ```sh
//! % cargo build --bin key_monitor
//! % espflash flash --monitor -p <serial port> target/xtensa-esp32s3-espidf/debug/key_monitor
//! ```
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::timer::EspTaskTimerService;
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time::Duration};

use cardputer::keyboard::{KeyImprint, Keyboard};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

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

    let (tx, rx): (Sender<Vec<KeyImprint>>, Receiver<Vec<KeyImprint>>) = std::sync::mpsc::channel();

    let timer_service = EspTaskTimerService::new().unwrap();
    let keyboard_task = Box::new(
        timer_service
            .timer(move || {
                let keys = keyboard.scan_pressed_keys().unwrap();
                let _ = tx.send(keys);
            })
            .unwrap(),
    );
    keyboard_task.every(Duration::from_millis(500u64)).unwrap();

    loop {
        let Ok(keys) = rx.try_recv() else {
            thread::sleep(Duration::from_millis(100));
            continue;
        };

        log::info!("{:?}", keys);
    }
}
