//! Simple example of keyboard scanning
//!
//! ```sh
//! % cargo build --example key_monitor_adv
//! % espflash flash --monitor -p <serial port> target/xtensa-esp32s3-espidf/debug/examples/key_monitor_adv
//! ```
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::timer::EspTaskTimerService;
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time::Duration};

use cardputer::adv::keyboard::{TCA8418RTWR, KeyboardState, KeyChordEvent};

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let tca8418 = TCA8418RTWR::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        peripherals.pins.gpio11,
    ).unwrap();
    let mut keyboard = KeyboardState::new(tca8418).unwrap();

    let (tx, rx): (Sender<Vec<KeyChordEvent>>, Receiver<Vec<KeyChordEvent>>) = std::sync::mpsc::channel();

    let timer_service = EspTaskTimerService::new().unwrap();
    let keyboard_task = Box::new(
        timer_service
            .timer(move || {
                if let Ok(keys) = keyboard.get_key_events() && !keys.is_empty() {
                    let _ = tx.send(keys.to_vec());
                }
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
