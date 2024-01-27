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
const KEY_MAP: [[KeyType; 14]; 4] = [
    [
        KeyType::Modifier(KeyImprint::LeftCtrl),
        KeyType::Modifier(KeyImprint::LeftOpt),
        KeyType::Modifier(KeyImprint::LeftAlt),
        normal!(KeyImprint::Z, graph!('z'), graph!('Z')),
        normal!(KeyImprint::X, graph!('x'), graph!('X')),
        normal!(KeyImprint::C, graph!('c'), graph!('C')),
        normal!(KeyImprint::V, graph!('v'), graph!('V')),
        normal!(KeyImprint::B, graph!('b'), graph!('B')),
        normal!(KeyImprint::N, graph!('n'), graph!('N')),
        normal!(KeyImprint::M, graph!('m'), graph!('M')),
        normal!(KeyImprint::Comma, graph!(','), graph!('<')),
        normal!(KeyImprint::Period, graph!('.'), graph!('>')),
        normal!(KeyImprint::Slash, graph!('/'), graph!('?')),
        normal!(KeyImprint::Space, Modified::Space, Modified::Space),
    ],
    [
        KeyType::Modifier(KeyImprint::LeftFn),
        KeyType::Modifier(KeyImprint::LeftShift),
        normal!(KeyImprint::A, graph!('a'), graph!('A')),
        normal!(KeyImprint::S, graph!('s'), graph!('S')),
        normal!(KeyImprint::D, graph!('d'), graph!('D')),
        normal!(KeyImprint::F, graph!('f'), graph!('F')),
        normal!(KeyImprint::G, graph!('g'), graph!('G')),
        normal!(KeyImprint::H, graph!('h'), graph!('H')),
        normal!(KeyImprint::J, graph!('j'), graph!('J')),
        normal!(KeyImprint::K, graph!('k'), graph!('K')),
        normal!(KeyImprint::L, graph!('l'), graph!('L')),
        normal!(KeyImprint::SemiColon, graph!(';'), graph!(':')),
        normal!(KeyImprint::Quote, graph!('\''), graph!('"')),
        normal!(KeyImprint::Enter, Modified::Enter, Modified::Enter),
    ],
    [
        normal!(KeyImprint::Tab, Modified::Tab, Modified::Tab),
        normal!(KeyImprint::Q, graph!('q'), graph!('Q')),
        normal!(KeyImprint::W, graph!('w'), graph!('W')),
        normal!(KeyImprint::E, graph!('e'), graph!('E')),
        normal!(KeyImprint::R, graph!('r'), graph!('R')),
        normal!(KeyImprint::T, graph!('t'), graph!('T')),
        normal!(KeyImprint::Y, graph!('y'), graph!('Y')),
        normal!(KeyImprint::U, graph!('u'), graph!('U')),
        normal!(KeyImprint::I, graph!('i'), graph!('I')),
        normal!(KeyImprint::O, graph!('o'), graph!('O')),
        normal!(KeyImprint::P, graph!('p'), graph!('P')),
        normal!(KeyImprint::OpenSquareBracket, graph!('['), graph!('{')),
        normal!(KeyImprint::CloseSquareBracket, graph!(']'), graph!('}')),
        normal!(KeyImprint::Backslash, graph!('\\'), graph!('|')),
    ],
    [
        normal!(KeyImprint::Backquote, graph!('`'), graph!('~')),
        normal!(KeyImprint::One, graph!('1'), graph!('!')),
        normal!(KeyImprint::Two, graph!('2'), graph!('@')),
        normal!(KeyImprint::Three, graph!('3'), graph!('#')),
        normal!(KeyImprint::Four, graph!('4'), graph!('$')),
        normal!(KeyImprint::Five, graph!('5'), graph!('%')),
        normal!(KeyImprint::Six, graph!('6'), graph!('^')),
        normal!(KeyImprint::Seven, graph!('7'), graph!('&')),
        normal!(KeyImprint::Eight, graph!('8'), graph!('*')),
        normal!(KeyImprint::Nine, graph!('9'), graph!('(')),
        normal!(KeyImprint::Zero, graph!('0'), graph!(')')),
        normal!(KeyImprint::Minus, graph!('-'), graph!('_')),
        normal!(KeyImprint::Equal, graph!('='), graph!('+')),
        normal!(
            KeyImprint::Backspace,
            Modified::Backspace,
            Modified::Backspace
        ),
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
    /// Scan the keyboard and return the Vector of KeyType.
    fn scan_pressed_keytypes(&mut self) -> Result<Vec<KeyType>>;
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
    ///
    /// **This method may be deprecated**
    pub fn scan_pressed_keys(&mut self) -> Result<Vec<KeyImprint>> {
        let keys = self
            .scan_pressed_keytypes()?
            .iter()
            .map(|x| x.imprint())
            .collect();
        Ok(keys)
    }
}

impl KeyboardScanner for Keyboard<'_> {
    fn scan_pressed_keytypes(&mut self) -> Result<Vec<KeyType>> {
        let mut keys: Vec<KeyType> = vec![];
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

        for pressed in keyboard.scan_pressed_keytypes()?.iter() {
            match pressed {
                KeyType::Modifier(KeyImprint::LeftFn) => self.is_fn_pressed = true,
                KeyType::Modifier(KeyImprint::LeftCtrl) => self.is_ctrl_pressed = true,
                KeyType::Modifier(KeyImprint::LeftShift) => self.is_shift_pressed = true,
                KeyType::Modifier(KeyImprint::LeftAlt) => self.is_alt_pressed = true,
                KeyType::Normal(h) => {
                    new_hold_keys.push(*h);
                    if !self.hold_keys.contains(h) {
                        self.pressed_keys.push(*h);
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
