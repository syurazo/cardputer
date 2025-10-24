//! Keyboard scanner that converts the 74HC138 decoding results
//! into the Vector of pressed key codes.
//!
//! ### 4x14 keymap
//!
//! ```text
//!  A2 A1 A0 |    Y0  |  Y1   |  Y2   |  Y3   |  Y4   |  Y5   |  Y6
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
    gpio::{AnyIOPin, AnyOutputPin, IOPin, Input, Level, Output, OutputPin, PinDriver, Pull},
    gpio::{Gpio11, Gpio13, Gpio15, Gpio3, Gpio4, Gpio5, Gpio6, Gpio7, Gpio8, Gpio9},
    peripheral::Peripheral,
};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modified {
    Graph(char),
    Escape,
    Enter,
    Space,
    Tab,
    LeftCursor,
    DownCursor,
    UpCursor,
    RightCursor,
    Backspace,
    Delete,
}
macro_rules! graph {
    ($x:expr) => {
        Modified::Graph($x)
    };
}

#[derive(Debug, PartialEq, Clone, Copy)]
/// Conversion rule
pub struct ConversionRule(KeyImprint, Modified, Modified);
impl ConversionRule {
    /// Convert according to the state of Fn and Shift key
    pub fn modified(&self, is_fn_pressed: bool, is_shift_pressed: bool) -> Modified {
        match (self.0, is_fn_pressed, is_shift_pressed) {
            (KeyImprint::SemiColon, true, _) => Modified::UpCursor,
            (KeyImprint::Period, true, _) => Modified::DownCursor,
            (KeyImprint::Slash, true, _) => Modified::RightCursor,
            (KeyImprint::Comma, true, _) => Modified::LeftCursor,
            (KeyImprint::Backquote, true, _) => Modified::Escape,
            (KeyImprint::Backspace, true, _) => Modified::Delete,
            (_, _, true) => self.2,
            (_, _, _) => self.1,
        }
    }

    /// Returns the imprint of the key assigned to the rule
    pub fn imprint(&self) -> KeyImprint {
        self.0
    }
}

#[derive(Debug, Copy, Clone)]
/// Define the type of key as modifier key and normal key
pub enum KeyType {
    Modifier(KeyImprint),
    Normal(ConversionRule),
}
impl KeyType {
    pub fn imprint(&self) -> KeyImprint {
        match self {
            KeyType::Modifier(x) => *x,
            KeyType::Normal(x) => x.imprint(),
        }
    }
}
macro_rules! normal {
    ($x:expr,$y:expr,$z:expr) => {
        KeyType::Normal(ConversionRule($x, $y, $z))
    };
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

/// Keyboard scanner trait
pub trait KeyboardScanner {
    /// Scan the keyboard and return the Vector of KeyImprint.
    fn scan_pressed_keys(&mut self) -> Result<Vec<KeyImprint>>;
}

/// Keyboard scanner for Cardputer
///
/// # Examples
///
/// ```
/// use cardputer::keyboard::{Keyboard, KeyImprint};
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
/// let keys: Vec<KeyImprint> = keyboard.scan_pressed_keys().unwrap();
/// ```
pub struct Keyboard<'a> {
    addr: [PinDriver<'a, AnyOutputPin, Output>; 3],
    inputs: [PinDriver<'a, AnyIOPin, Input>; 7],
}
impl<'a> Keyboard<'a> {
    /// Create new scanner.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        a0: impl Peripheral<P = Gpio8> + 'a + OutputPin,
        a1: impl Peripheral<P = Gpio9> + 'a + OutputPin,
        a2: impl Peripheral<P = Gpio11> + 'a + OutputPin,
        y0: impl Peripheral<P = Gpio13> + 'a + IOPin,
        y1: impl Peripheral<P = Gpio15> + 'a + IOPin,
        y2: impl Peripheral<P = Gpio3> + 'a + IOPin,
        y3: impl Peripheral<P = Gpio4> + 'a + IOPin,
        y4: impl Peripheral<P = Gpio5> + 'a + IOPin,
        y5: impl Peripheral<P = Gpio6> + 'a + IOPin,
        y6: impl Peripheral<P = Gpio7> + 'a + IOPin,
    ) -> Result<Self> {
        let addr = [
            PinDriver::output(a0.downgrade_output())?,
            PinDriver::output(a1.downgrade_output())?,
            PinDriver::output(a2.downgrade_output())?,
        ];
        let mut inputs = [
            PinDriver::input(y0.downgrade())?,
            PinDriver::input(y1.downgrade())?,
            PinDriver::input(y2.downgrade())?,
            PinDriver::input(y3.downgrade())?,
            PinDriver::input(y4.downgrade())?,
            PinDriver::input(y5.downgrade())?,
            PinDriver::input(y6.downgrade())?,
        ];
        for pin in inputs.iter_mut() {
            pin.set_pull(Pull::Up)?;
        }
        Ok(Self { addr, inputs })
    }
}

impl KeyboardScanner for Keyboard<'_> {
    fn scan_pressed_keys(&mut self) -> Result<Vec<KeyImprint>> {
        let mut keys: Vec<KeyImprint> = vec![];
        for i in 0..8 {
            for (j, ad) in self.addr.iter_mut().enumerate() {
                ad.set_level(pin_level!(i & (0b00000001 << j)))?;
            }
            let inputs: Vec<Level> = self.inputs.iter().map(|x| x.get_level()).collect();
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

/// Structure that scans the keyboard and keeps track of state changes
///
/// # Examples
///
/// ```
/// use cardputer::keyboard::{Keyboard, KeyboardState};
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
///
/// let mut keyboard_state = KeyboardState::default();
/// keyboard_state.update(&mut keyboard).unwrap();
/// log::info!("{:?}", keyboard_state.pressed_keys());
/// log::info!("{:?}", keyboard_state.released_keys());
/// ```
#[derive(Debug, Default)]
pub struct KeyboardState {
    is_fn_pressed: bool,
    is_ctrl_pressed: bool,
    is_shift_pressed: bool,
    is_alt_pressed: bool,

    hold_keys: Vec<ConversionRule>,
    pressed_keys: Vec<ConversionRule>,
    released_keys: Vec<ConversionRule>,
}

impl KeyboardState {
    /// Get the latest key state and update the Pressed/Released state
    pub fn update(&mut self, keyboard: &mut impl KeyboardScanner) -> Result<()> {
        let mut new_hold_keys: Vec<ConversionRule> = Vec::new();

        self.pressed_keys.clear();
        self.released_keys.clear();

        self.is_fn_pressed = false;
        self.is_ctrl_pressed = false;
        self.is_shift_pressed = false;
        self.is_alt_pressed = false;

        for pressed in keyboard.scan_pressed_keys()?.into_iter() {
            let key_type: KeyType = pressed.into();
            match key_type {
                KeyType::Modifier(KeyImprint::LeftFn) => self.is_fn_pressed = true,
                KeyType::Modifier(KeyImprint::LeftCtrl) => self.is_ctrl_pressed = true,
                KeyType::Modifier(KeyImprint::LeftShift) => self.is_shift_pressed = true,
                KeyType::Modifier(KeyImprint::LeftAlt) => self.is_alt_pressed = true,
                KeyType::Normal(h) => {
                    new_hold_keys.push(h);
                    if !self.hold_keys.contains(&h) {
                        self.pressed_keys.push(h);
                    }
                }
                _ => {}
            }
        }

        for key in self.hold_keys.iter() {
            if !new_hold_keys.contains(key) {
                self.released_keys.push(*key);
            }
        }

        self.hold_keys = new_hold_keys;

        Ok(())
    }

    pub fn pressed_keys(&self) -> Vec<Modified> {
        self.pressed_keys
            .iter()
            .map(|x| x.modified(self.is_fn_pressed, self.is_shift_pressed))
            .collect()
    }

    pub fn released_keys(&self) -> Vec<Modified> {
        self.released_keys
            .iter()
            .map(|x| x.modified(self.is_fn_pressed, self.is_shift_pressed))
            .collect()
    }

    pub fn hold_keys(&self) -> Vec<Modified> {
        self.hold_keys
            .iter()
            .map(|x| x.modified(self.is_fn_pressed, self.is_shift_pressed))
            .collect()
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
}

impl From<KeyImprint> for KeyType {
    fn from(imprint: KeyImprint) -> Self {
        match imprint {
            KeyImprint::Backquote => normal!(KeyImprint::Backquote, graph!('`'), graph!('~')),
            KeyImprint::One => normal!(KeyImprint::One, graph!('1'), graph!('!')),
            KeyImprint::Two => normal!(KeyImprint::Two, graph!('2'), graph!('@')),
            KeyImprint::Three => normal!(KeyImprint::Three, graph!('3'), graph!('#')),
            KeyImprint::Four => normal!(KeyImprint::Four, graph!('4'), graph!('$')),
            KeyImprint::Five => normal!(KeyImprint::Five, graph!('5'), graph!('%')),
            KeyImprint::Six => normal!(KeyImprint::Six, graph!('6'), graph!('^')),
            KeyImprint::Seven => normal!(KeyImprint::Seven, graph!('7'), graph!('&')),
            KeyImprint::Eight => normal!(KeyImprint::Eight, graph!('8'), graph!('*')),
            KeyImprint::Nine => normal!(KeyImprint::Nine, graph!('9'), graph!('(')),
            KeyImprint::Zero => normal!(KeyImprint::Zero, graph!('0'), graph!(')')),
            KeyImprint::Minus => normal!(KeyImprint::Minus, graph!('-'), graph!('_')),
            KeyImprint::Equal => normal!(KeyImprint::Equal, graph!('='), graph!('+')),
            KeyImprint::Backspace => normal!(
                    KeyImprint::Backspace,
                    Modified::Backspace,
                    Modified::Backspace
                ),
            KeyImprint::Tab => normal!(KeyImprint::Tab, Modified::Tab, Modified::Tab),
            KeyImprint::Q => normal!(KeyImprint::Q, graph!('q'), graph!('Q')),
            KeyImprint::W => normal!(KeyImprint::W, graph!('w'), graph!('W')),
            KeyImprint::E => normal!(KeyImprint::E, graph!('e'), graph!('E')),
            KeyImprint::R => normal!(KeyImprint::R, graph!('r'), graph!('R')),
            KeyImprint::T => normal!(KeyImprint::T, graph!('t'), graph!('T')),
            KeyImprint::Y => normal!(KeyImprint::Y, graph!('y'), graph!('Y')),
            KeyImprint::U => normal!(KeyImprint::U, graph!('u'), graph!('U')),
            KeyImprint::I => normal!(KeyImprint::I, graph!('i'), graph!('I')),
            KeyImprint::O => normal!(KeyImprint::O, graph!('o'), graph!('O')),
            KeyImprint::P => normal!(KeyImprint::P, graph!('p'), graph!('P')),
            KeyImprint::OpenSquareBracket => normal!(KeyImprint::OpenSquareBracket, graph!('['), graph!('{')),
            KeyImprint::CloseSquareBracket => normal!(KeyImprint::CloseSquareBracket, graph!(']'), graph!('}')),
            KeyImprint::Backslash => normal!(KeyImprint::Backslash, graph!('\\'), graph!('|')),
            KeyImprint::LeftFn => KeyType::Modifier(KeyImprint::LeftFn),
            KeyImprint::LeftShift => KeyType::Modifier(KeyImprint::LeftShift),
            KeyImprint::A => normal!(KeyImprint::A, graph!('a'), graph!('A')),
            KeyImprint::S => normal!(KeyImprint::S, graph!('s'), graph!('S')),
            KeyImprint::D => normal!(KeyImprint::D, graph!('d'), graph!('D')),
            KeyImprint::F => normal!(KeyImprint::F, graph!('f'), graph!('F')),
            KeyImprint::G => normal!(KeyImprint::G, graph!('g'), graph!('G')),
            KeyImprint::H => normal!(KeyImprint::H, graph!('h'), graph!('H')),
            KeyImprint::J => normal!(KeyImprint::J, graph!('j'), graph!('J')),
            KeyImprint::K => normal!(KeyImprint::K, graph!('k'), graph!('K')),
            KeyImprint::L => normal!(KeyImprint::L, graph!('l'), graph!('L')),
            KeyImprint::SemiColon => normal!(KeyImprint::SemiColon, graph!(';'), graph!(':')),
            KeyImprint::Quote => normal!(KeyImprint::Quote, graph!('\''), graph!('"')),
            KeyImprint::Enter => normal!(KeyImprint::Enter, Modified::Enter, Modified::Enter),
            KeyImprint::LeftCtrl => KeyType::Modifier(KeyImprint::LeftCtrl),
            KeyImprint::LeftOpt => KeyType::Modifier(KeyImprint::LeftOpt),
            KeyImprint::LeftAlt => KeyType::Modifier(KeyImprint::LeftAlt),
            KeyImprint::Z => normal!(KeyImprint::Z, graph!('z'), graph!('Z')),
            KeyImprint::X => normal!(KeyImprint::X, graph!('x'), graph!('X')),
            KeyImprint::C => normal!(KeyImprint::C, graph!('c'), graph!('C')),
            KeyImprint::V => normal!(KeyImprint::V, graph!('v'), graph!('V')),
            KeyImprint::B => normal!(KeyImprint::B, graph!('b'), graph!('B')),
            KeyImprint::N => normal!(KeyImprint::N, graph!('n'), graph!('N')),
            KeyImprint::M => normal!(KeyImprint::M, graph!('m'), graph!('M')),
            KeyImprint::Comma => normal!(KeyImprint::Comma, graph!(','), graph!('<')),
            KeyImprint::Period => normal!(KeyImprint::Period, graph!('.'), graph!('>')),
            KeyImprint::Slash => normal!(KeyImprint::Slash, graph!('/'), graph!('?')),
            KeyImprint::Space => normal!(KeyImprint::Space, Modified::Space, Modified::Space),
        }
    }
}
