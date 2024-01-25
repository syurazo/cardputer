//! Keyboard scanner that converts the 74HC138 decoding results
//! into the Vector of pressed key codes.
//!
//! ### 4x14 keymap
//!
//! ```text
//!  A2 A2 A0 |    Y0  |  Y1   |  Y2   |  Y3   |  Y4   |  Y5   |  Y6
//!  H  -  -  |  L     | L     | L     | L     | L     | L     | L
//!  L  -  -  |      L |     L |     L |     L |     L |     L |     L
//! ----------+--------+-------+-------+-------+-------+-------+-------
//!  -  H  H  |  `   1   2   3   4   5   6   7   8   9   0   -   =  DEL
//!  -  H  L  | TAB  q   w   e   r   t   y   u   i   o   p   [   ]   \
//!  -  L  H  | FN  SHT  a   s   d   f   g   h   j   k   l   ;   '  ENT
//!  -  L  L  | CTL OPT ALT  z   x   c   v   b   n   m   ,   .   /  SPC
//! ```
use anyhow::Result;
use esp_idf_hal::{
    gpio::{Gpio11, Gpio13, Gpio15, Gpio3, Gpio4, Gpio5, Gpio6, Gpio7, Gpio8, Gpio9},
    gpio::{Input, Level, Output, PinDriver},
    peripheral::Peripheral,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyImprint {
    Backquote,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Zero,
    Minus,
    Equal,
    Backspace,
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    OpenSquareBracket,
    CloseSquareBracket,
    Backslash,
    LeftFn,
    LeftShift,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    SemiColon,
    Quote,
    Enter,
    LeftCtrl,
    LeftOpt,
    LeftAlt,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Period,
    Slash,
    Space,
}

const COLUMN_MAP: [[usize; 7]; 2] = [[1, 3, 5, 7, 9, 11, 13], [0, 2, 4, 6, 8, 10, 12]];
const KEY_MAP: [[KeyImprint; 14]; 4] = [
    [
        KeyImprint::LeftCtrl,
        KeyImprint::LeftOpt,
        KeyImprint::LeftAlt,
        KeyImprint::Z,
        KeyImprint::X,
        KeyImprint::C,
        KeyImprint::V,
        KeyImprint::B,
        KeyImprint::N,
        KeyImprint::M,
        KeyImprint::Comma,
        KeyImprint::Period,
        KeyImprint::Slash,
        KeyImprint::Space,
    ],
    [
        KeyImprint::LeftFn,
        KeyImprint::LeftShift,
        KeyImprint::A,
        KeyImprint::S,
        KeyImprint::D,
        KeyImprint::F,
        KeyImprint::G,
        KeyImprint::H,
        KeyImprint::J,
        KeyImprint::K,
        KeyImprint::L,
        KeyImprint::SemiColon,
        KeyImprint::Quote,
        KeyImprint::Enter,
    ],
    [
        KeyImprint::Tab,
        KeyImprint::Q,
        KeyImprint::W,
        KeyImprint::E,
        KeyImprint::R,
        KeyImprint::T,
        KeyImprint::Y,
        KeyImprint::U,
        KeyImprint::I,
        KeyImprint::O,
        KeyImprint::P,
        KeyImprint::OpenSquareBracket,
        KeyImprint::CloseSquareBracket,
        KeyImprint::Backslash,
    ],
    [
        KeyImprint::Backquote,
        KeyImprint::One,
        KeyImprint::Two,
        KeyImprint::Three,
        KeyImprint::Four,
        KeyImprint::Five,
        KeyImprint::Six,
        KeyImprint::Seven,
        KeyImprint::Eight,
        KeyImprint::Nine,
        KeyImprint::Zero,
        KeyImprint::Minus,
        KeyImprint::Equal,
        KeyImprint::Backspace,
    ],
];

macro_rules! pin_level {
    ($x:expr) => {
        match $x {
            0 => Level::Low,
            _ => Level::High,
        }
    };
}

/// Keyboard scanner for Cardputer
///
/// # Examples
///
/// ```
/// use cardputer::keyboard::Keyboard;
///
/// let peripherals = Peripherals::take().unwrap();
///
/// let mut keyboard = Keyboard::new(
///     peripherals.pins.gpio8,
///     peripherals.pins.gpio9,
///     peripherals.pins.gpio11,
///     peripherals.pins.gpio13,
///     peripherals.pins.gpio15,
///     peripherals.pins.gpio3,
///     peripherals.pins.gpio4,
///     peripherals.pins.gpio5,
///     peripherals.pins.gpio6,
///     peripherals.pins.gpio7,
/// )
/// .unwrap();
/// let keys = keyboard.scan_pressed_keys().unwrap();
/// ```
pub struct Keyboard<'a> {
    addr0: PinDriver<'a, Gpio8, Output>,
    addr1: PinDriver<'a, Gpio9, Output>,
    addr2: PinDriver<'a, Gpio11, Output>,
    y0: PinDriver<'a, Gpio13, Input>,
    y1: PinDriver<'a, Gpio15, Input>,
    y2: PinDriver<'a, Gpio3, Input>,
    y3: PinDriver<'a, Gpio4, Input>,
    y4: PinDriver<'a, Gpio5, Input>,
    y5: PinDriver<'a, Gpio6, Input>,
    y6: PinDriver<'a, Gpio7, Input>,
}
impl<'a> Keyboard<'a> {
    /// Create new scanner.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        a0: impl Peripheral<P = Gpio8> + 'a,
        a1: impl Peripheral<P = Gpio9> + 'a,
        a2: impl Peripheral<P = Gpio11> + 'a,
        y0: impl Peripheral<P = Gpio13> + 'a,
        y1: impl Peripheral<P = Gpio15> + 'a,
        y2: impl Peripheral<P = Gpio3> + 'a,
        y3: impl Peripheral<P = Gpio4> + 'a,
        y4: impl Peripheral<P = Gpio5> + 'a,
        y5: impl Peripheral<P = Gpio6> + 'a,
        y6: impl Peripheral<P = Gpio7> + 'a,
    ) -> Result<Self> {
        Ok(Self {
            addr0: PinDriver::output(a0)?,
            addr1: PinDriver::output(a1)?,
            addr2: PinDriver::output(a2)?,
            y0: PinDriver::input(y0)?,
            y1: PinDriver::input(y1)?,
            y2: PinDriver::input(y2)?,
            y3: PinDriver::input(y3)?,
            y4: PinDriver::input(y4)?,
            y5: PinDriver::input(y5)?,
            y6: PinDriver::input(y6)?,
        })
    }

    /// Scan the keyboard and return the Vector of KeyImprint.
    pub fn scan_pressed_keys(&mut self) -> Result<Vec<KeyImprint>> {
        let mut keys: Vec<KeyImprint> = vec![];
        for i in 0..8 {
            self.addr0.set_level(pin_level!(i & 0b00000001))?;
            self.addr1.set_level(pin_level!(i & 0b00000010))?;
            self.addr2.set_level(pin_level!(i & 0b00000100))?;

            let inputs: [Level; 7] = [
                self.y0.get_level(),
                self.y1.get_level(),
                self.y2.get_level(),
                self.y3.get_level(),
                self.y4.get_level(),
                self.y5.get_level(),
                self.y6.get_level(),
            ];
            for (j, decoded) in inputs.iter().enumerate() {
                if *decoded == Level::High {
                    continue;
                }
                let (col, row) = if i < 4 {
                    (COLUMN_MAP[0][j], i)
                } else {
                    (COLUMN_MAP[1][j], i - 4)
                };
                keys.push(KEY_MAP[row][col]);
            }
        }

        Ok(keys)
    }
}
