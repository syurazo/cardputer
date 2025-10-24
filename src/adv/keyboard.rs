//! Keyboard scanner that converts the TCA8418RTWR decoding results
//! into the Vector of pressed key codes.
//!
use anyhow::Result;
use esp_idf_hal::{
    delay::TickType,
    gpio::{Gpio11, Gpio8, Gpio9},
    i2c::{I2C0, I2cConfig, I2cDriver},
    peripheral::Peripheral,
    units::Hertz
};
use std::collections::HashMap;

use crate::keyboard::{KeyImprint};

/// I2C timeout in milliseconds
const I2C_TIMEOUT_MS: u64 = 500u64;

/// 7bit I2C address
const I2C_ADDRESS: u8 = 0x34;

/// Configuration Register
///
/// Addr | bit7 | bit6 | bit5 | bit4 | bit3 | bit2 | bit1 | bit0
/// :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :---
/// 0x01 | `--` | `--` | `--` | `--` | `--` | `--` | `--` | `KE_IEN`
///
/// ## `KEIEN`: Key Event Interrupt Enable
///
/// * 0: disabled
/// * 1: enabled
///
const ADDR_CFG: u8 = 0x01;
/// Interrupt Status Register
///
/// Addr | bit7 | bit6 | bit5 | bit4 | bit3 | bit2 | bit1 | bit0
/// :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :------
/// 0x02 | `--` | `--` | `--` | `--` | `--` | `--` | `--` | `K_INT`
///
/// ## `K_INT`: Key Event Interrupt Status
///
/// * 0: not detected
/// * 1: detected
///
const INT_STATUS: u8 = 0x02;
/// Key Lock / Event Counter Register
///
/// Addr | bit7 | bit6 | bit5 | bit4 | bit3 | bit2 | bit1 | bit0
/// :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :---
/// 0x03 | `--` | `--` | `--` | `--` | KEC3 | KEC2 | KEC1 | KEC0
///
const REG_KEY_LCK_EC: u8 = 0x03;
/// Key Event Register
///
/// Addr | bit7 | bit6 | bit5 | bit4 | bit3 | bit2 | bit1 | bit0
/// :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :---
/// 0x04 | KEA7 | KEA6 | KEA5 | KEA4 | KEA3 | KEA2 | KEA1 | KEA0
///
const REG_KEY_EVENT_A: u8 = 0x04;
/// Keypad or GPIO Selection Register (1)
///
/// Addr | bit7 | bit6 | bit5 | bit4 | bit3 | bit2 | bit1 | bit0
/// :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :---
/// 0x1D | ROW7 | ROW6 | ROW5 | ROW4 | ROW3 | ROW2 | ROW1 | ROW0
///
const ADDR_KP_GPIO1: u8 = 0x1D;
/// Keypad or GPIO Selection Register (2)
///
/// Addr | bit7 | bit6 | bit5 | bit4 | bit3 | bit2 | bit1 | bit0
/// :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :---
/// 0x1E | COL7 | COL6 | COL5 | COL4 | COL3 | COL2 | COL1 | COL0
///
const ADDR_KP_GPIO2: u8 = 0x1E;
/// Keypad or GPIO Selection Register (3)
///
/// Addr | bit7 | bit6 | bit5 | bit4 | bit3 | bit2 | bit1 | bit0
/// :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :---
/// 0x1F | N/A  | N/A  | N/A  | N/A  | N/A  | N/A  | COL9 | COL8
///
const ADDR_KP_GPIO3: u8 = 0x1F;

/// Key conversion table indexed from bit 7 to bit 0 of `REG_KEY_EVENT_A`
///
///  H/L | 1       | 2    | 3    | 4    | 5    | 6    | 7    | 8    | 9    | 10
///  --: | :---    | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :---
///   00 | `` ` `` | TAB  | FN   | CTRL | `1`  | `Q`  | SHFT | OPT  | n/a  | n/a
///   10 | `2`     | `W`  | `A`  | ALT  | `3`  | `E`  | `S`  | `Z`  | n/a  | n/a
///   20 | `4`     | `R`  | `D`  | `X`  | `5`  | `T`  | `F`  | `C`  | n/a  | n/a
///   30 | `6`     | `Y`  | `G`  | `V`  | `7`  | `U`  | `H`  | `B`  | n/a  | n/a
///   40 | `8`     | `I`  | `J`  | `N`  | `9`  | `O`  | `K`  | `M`  | n/a  | n/a
///   50 | `0`     | `P`  | `L`  | `,`  | `-`  | `[`  | `;`  | `.`  | n/a  | n/a
///   60 | `=`     | `]`  | `'`  | `/`  | BS   | `\`  | ENTR | SPC  | n/a  | n/a
///
const KEY_MATRIX: [KeyImprint; 56] = [
    KeyImprint::Backquote,
    KeyImprint::Tab,
    KeyImprint::LeftFn,
    KeyImprint::LeftCtrl,
    KeyImprint::One,
    KeyImprint::Q,
    KeyImprint::LeftShift,
    KeyImprint::LeftOpt,

    KeyImprint::Two,
    KeyImprint::W,
    KeyImprint::A,
    KeyImprint::LeftAlt,
    KeyImprint::Three,
    KeyImprint::E,
    KeyImprint::S,
    KeyImprint::Z,

    KeyImprint::Four,
    KeyImprint::R,
    KeyImprint::D,
    KeyImprint::X,
    KeyImprint::Five,
    KeyImprint::T,
    KeyImprint::F,
    KeyImprint::C,

    KeyImprint::Six,
    KeyImprint::Y,
    KeyImprint::G,
    KeyImprint::V,
    KeyImprint::Seven,
    KeyImprint::U,
    KeyImprint::H,
    KeyImprint::B,

    KeyImprint::Eight,
    KeyImprint::I,
    KeyImprint::J,
    KeyImprint::N,
    KeyImprint::Nine,
    KeyImprint::O,
    KeyImprint::K,
    KeyImprint::M,

    KeyImprint::Zero,
    KeyImprint::P,
    KeyImprint::L,
    KeyImprint::Comma,
    KeyImprint::Minus,
    KeyImprint::OpenSquareBracket,
    KeyImprint::SemiColon,
    KeyImprint::Period,

    KeyImprint::Equal,
    KeyImprint::CloseSquareBracket,
    KeyImprint::Quote,
    KeyImprint::Slash,
    KeyImprint::Backspace,
    KeyImprint::Backslash,
    KeyImprint::Enter,
    KeyImprint::Space,
];

#[derive(Debug, Clone, PartialEq)]
/// A specific key combined with modifier keys such as Ctrl, Alt, or Shift
pub struct KeyChord {
    key_imprint: KeyImprint,

    is_fn_pressed: bool,
    is_ctrl_pressed: bool,
    is_shift_pressed: bool,
    is_alt_pressed: bool,
    is_opt: bool,
}
impl KeyChord {
    pub fn imprint(&self) -> KeyImprint {
        self.key_imprint
    }

    pub fn is_fn_pressed(&self) -> bool {
        self.is_fn_pressed
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        self.is_ctrl_pressed
    }

    pub fn is_shift_pressed(&self) -> bool {
        self.is_shift_pressed
    }

    pub fn is_alt_pressed(&self) -> bool {
        self.is_alt_pressed
    }

    pub fn is_opt_pressed(&self) -> bool {
        self.is_opt
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyEvent<T> {
    Pressed(T),
    Released(T),
}

pub type KeyImprintEvent = KeyEvent<KeyImprint>;
pub type KeyChordEvent = KeyEvent<KeyChord>;

/// TCA8418RTWR Driver
///
/// **Key conversion using modifier keys is not performed.**
///
/// # Pins Assignment
///
/// * SDA: GPIO8
/// * SLC: GPIO9
/// * INT: GPIO11
///
/// # Examples
///
/// ```
/// use cardputer::keyboard::adv::{KeyImprintEvent, TCA8418RTWR};
///
/// let mut tca8418 = TCA8418RTWR::new(
///     peripherals.i2c0,
///     peripherals.pins.gpio8,
///     peripherals.pins.gpio9,
///     peripherals.pins.gpio11,
/// ).unwrap();
///
/// let keys: Vec<KeyImprintEvent> = tca8418.get_key_events().unwrap();
/// ```
pub struct TCA8418RTWR<'a> {
    i2c: I2cDriver<'a>,
}
impl<'a> TCA8418RTWR<'a> {
    pub fn new(
        i2c: impl Peripheral<P = I2C0> + 'a,
        sda: impl Peripheral<P = Gpio8> + 'a,
        scl: impl Peripheral<P = Gpio9> + 'a,
        _interrupt: impl Peripheral<P = Gpio11> + 'a,
    ) -> Result<Self> {
        let config = I2cConfig::new().baudrate(Hertz(400_000));
        let mut tca8418 = Self {
            i2c: I2cDriver::new(i2c, sda, scl, &config)?,
        };
        tca8418.reset()?;
        tca8418.fifo(true)?;
        Ok(tca8418)
    }

    fn default_timeout(&self) -> TickType {
        TickType::new_millis(I2C_TIMEOUT_MS)
    }
    fn reset(&mut self) -> Result<()> {
        let timeout = self.default_timeout();
        self.i2c.write(I2C_ADDRESS, &[ADDR_KP_GPIO1, 0x7F], timeout.ticks())?;
        self.i2c.write(I2C_ADDRESS, &[ADDR_KP_GPIO2, 0xFF], timeout.ticks())?;
        self.i2c.write(I2C_ADDRESS, &[ADDR_KP_GPIO3, 0x00], timeout.ticks())?;
        Ok(())
    }

    fn fifo(&mut self, enabled: bool) -> Result<()> {
        let timeout = self.default_timeout();
        let mode: u8 = if enabled { 0x01 } else { 0x00 };
        self.i2c.write(I2C_ADDRESS, &[ADDR_CFG, mode], timeout.ticks())?;
        self.i2c.write(I2C_ADDRESS, &[INT_STATUS, 0x00], timeout.ticks())?;
        Ok(())
    }

    pub fn get_key_event(&mut self) -> Result<Option<KeyImprintEvent>> {
        let timeout = self.default_timeout();

        let mut key_data: [u8;1] = [0xff];
        self.i2c.write_read(I2C_ADDRESS, &[REG_KEY_EVENT_A], &mut key_data, timeout.ticks())?;
        if key_data[0] == 0xff {
            return Ok(None);
        }

        let pressed = key_data[0] & 0x80 == 0x80;
        let key = key_data[0] & 0x7f;
        let imprint = KEY_MATRIX[ (key - (key / 10) * 2 - 1) as usize];
        Ok(Some(if pressed {
            KeyEvent::Pressed(imprint)
        } else {
            KeyEvent::Released(imprint)
        }))
    }

    pub fn get_key_events(&mut self) -> Result<Vec<KeyImprintEvent>> {
        let timeout = self.default_timeout();

        let mut read_buf: [u8;1] = [0xff];
        self.i2c.write_read(I2C_ADDRESS, &[REG_KEY_LCK_EC], &mut read_buf, timeout.ticks())?;
        let event_count: u8 = read_buf[0] & 0x0f;

        let mut key_events: Vec<KeyImprintEvent> = Vec::new();
        if event_count > 0 {
            for _ in 0..event_count {
                if let Some(event) = self.get_key_event()? {
                    key_events.push(event);
                }
            }
        }
        Ok(key_events)
    }
}

/// A structure for capturing key events and tracking state changes
///
/// # Examples
///
/// ```
/// use cardputer::keyboard::adv::{KeyboardState KeyImprintEvent, TCA8418RTWR};
///
/// let tca8418 = TCA8418RTWR::new(
///     peripherals.i2c0,
///     peripherals.pins.gpio8,
///     peripherals.pins.gpio9,
///     peripherals.pins.gpio11,
/// ).unwrap();
///
/// let mut keyboard = KeyboardState::new(tca8418).unwrap();
/// let keys: Vec<KeyChordEvent> = keyboard.get_key_events().unwrap();
/// ```
pub struct KeyboardState<'a> {
    tca8418: TCA8418RTWR<'a>,

    key_state: HashMap<KeyImprint, KeyChord>,

    is_fn_pressed: bool,
    is_ctrl_pressed: bool,
    is_shift_pressed: bool,
    is_alt_pressed: bool,
    is_opt_pressed: bool,
}
impl<'a> KeyboardState<'a> {
    pub fn new(
        tca8418: TCA8418RTWR<'a>,
    ) -> Result<Self> {
        Ok(Self {
            tca8418,
            key_state: HashMap::new(),
            is_fn_pressed: false,
            is_ctrl_pressed: false,
            is_shift_pressed: false,
            is_alt_pressed: false,
            is_opt_pressed: false,
        })
    }

    fn handle_pressed_key(&mut self, imprint: KeyImprint) -> Option<KeyChord> {
        match imprint {
            KeyImprint::LeftFn => self.is_fn_pressed = true,
            KeyImprint::LeftCtrl => self.is_ctrl_pressed = true,
            KeyImprint::LeftShift => self.is_shift_pressed = true,
            KeyImprint::LeftAlt => self.is_alt_pressed = true,
            KeyImprint::LeftOpt => self.is_opt_pressed = true,
            _ => {
                let chord = KeyChord {
                    key_imprint: imprint,
                    is_fn_pressed: self.is_fn_pressed,
                    is_ctrl_pressed: self.is_ctrl_pressed,
                    is_shift_pressed: self.is_shift_pressed,
                    is_alt_pressed: self.is_alt_pressed,
                    is_opt: self.is_opt_pressed,
                };
                self.key_state.insert(imprint, chord.clone());
                return Some(chord);
            }
        }
        None
    }

    fn handle_released_key(&mut self, imprint: KeyImprint) -> Option<KeyChord> {
        match imprint {
            KeyImprint::LeftFn => self.is_fn_pressed = false,
            KeyImprint::LeftCtrl => self.is_ctrl_pressed = false,
            KeyImprint::LeftShift => self.is_shift_pressed = false,
            KeyImprint::LeftAlt => self.is_alt_pressed = false,
            KeyImprint::LeftOpt => self.is_opt_pressed = false,
            _ => {
                if let Some(chord) = self.key_state.remove(&imprint) {
                    return Some(chord);
                }
            }
        }
        None
    }

    pub fn get_key_events(&mut self) -> Result<Vec<KeyChordEvent>> {
        let mut events: Vec<KeyChordEvent> = Vec::new();

        for event in self.tca8418.get_key_events()?.into_iter() {
            match event {
                KeyEvent::Pressed(imprint) => {
                    if let Some(chord) = self.handle_pressed_key(imprint) {
                        events.push(KeyChordEvent::Pressed(chord));
                    }
                },
                KeyEvent::Released(imprint) => {
                    if let Some(chord) = self.handle_released_key(imprint) {
                        events.push(KeyChordEvent::Released(chord));
                    }
                },
            }
        }

        Ok(events)
    }
}
