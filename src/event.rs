use std::ops::{BitAnd, BitOr, Not};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RawEvent {
    pub(crate) window: u64,
    pub(crate) event: Event,
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Window(WindowEvent),
    Widget(WidgetEvent),
    Unknown,
}

#[derive(Clone, Copy, Debug)]
pub enum WindowEvent {
    /// Keyboard key press
    KeyPress(Modifiers, Key),
    /// Keyboard key release
    KeyRelease(Modifiers, Key),
    /// Sent when window changes state
    StateChange(WindowState),
    /// Sent when the window is resized
    Resize(u32, u32),
    /// Sent when window is asked to close
    Closing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowState {
    Maximized,
    Fullscreen,
    Suspended,
    Activated,
}

#[derive(Clone, Copy, Debug)]
pub enum WidgetEvent {
    /// Sent when entering(true) or leaving(false)
    Hover(bool),
    /// Cursor is moving. (x,y) is in pixels
    Move(i32, i32),
    /// Pointer button press. (x,y) is in pixels
    ButtonPress(Button, i32, i32),
    /// Pointer button release. (x,y) is in pixels
    ButtonRelease(Button, i32, i32),
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum Button {
    Left = 1,
    Middle = 3,
    Right = 2,
    Forward = 4,
    Back = 5,
    Other = 0,
}

impl Button {
    pub fn from_code(code: u8) -> Self {
        match code {
            1 => Self::Left,
            2 => Self::Middle,
            3 => Self::Right,
            4 => Self::Forward,
            5 => Self::Back,
            _ => Self::Other,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Key {
    Escape,
    Tab,
    Space,
    a,
    b,
    c,
    d,
    e,
    f,
    g,
    h,
    i,
    j,
    k,
    l,
    m,
    n,
    o,
    p,
    q,
    r,
    s,
    t,
    u,
    v,
    w,
    x,
    y,
    z,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Key1,
    Exclam,
    Key2,
    At,
    Key3,
    NumberSign,
    Key4,
    Dollar,
    Key5,
    Percent,
    Key6,
    Circum,
    Key7,
    Amp,
    Key8,
    Asterisk,
    Key9,
    LeftParen,
    Key0,
    RightParen,
    Backtick,
    Tilde,
    Minus,
    Underscore,
    Equals,
    Plus,
    LeftBracket,
    LeftBrace,
    RightBracket,
    RightBrace,
    Backslash,
    Bar,
    Colon,
    Semicolon,
    Apostrophe,
    Quote,
    Comma,
    Less,
    Period,
    Greater,
    Slash,
    Question,
    /// 103rd key, sometimes found between LShift and Z.
    Oem103,
    Backspace,
    Enter,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScreen,
    /// Mostly outdated
    Pause,
    Break,
    Insert,
    Delete,
    Home,
    End,
    PgUp,
    PgDn,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    NumAdd,
    NumSub,
    NumMul,
    NumDiv,
    NumEnter,
    NumDecimal,
    Menu,
    Other,
}

impl Key {
    pub fn shift(self) -> Self {
        match self {
            Self::a => Self::A,
            Self::b => Self::B,
            Self::c => Self::C,
            Self::d => Self::D,
            Self::e => Self::E,
            Self::f => Self::F,
            Self::g => Self::G,
            Self::h => Self::H,
            Self::i => Self::I,
            Self::j => Self::J,
            Self::k => Self::K,
            Self::l => Self::L,
            Self::m => Self::M,
            Self::n => Self::N,
            Self::o => Self::O,
            Self::p => Self::P,
            Self::q => Self::Q,
            Self::r => Self::R,
            Self::s => Self::S,
            Self::t => Self::T,
            Self::u => Self::U,
            Self::v => Self::V,
            Self::w => Self::W,
            Self::x => Self::X,
            Self::y => Self::Y,
            Self::z => Self::Z,
            Self::Key1 => Self::Exclam,
            Self::Key2 => Self::At,
            Self::Key3 => Self::NumberSign,
            Self::Key4 => Self::Dollar,
            Self::Key5 => Self::Percent,
            Self::Key6 => Self::Circum,
            Self::Key7 => Self::Amp,
            Self::Key8 => Self::Asterisk,
            Self::Key9 => Self::LeftParen,
            Self::Key0 => Self::RightParen,
            Self::Backtick => Self::Tilde,
            Self::Minus => Self::Underscore,
            Self::Equals => Self::Plus,
            Self::LeftBracket => Self::LeftBrace,
            Self::RightBracket => Self::RightBrace,
            Self::Backslash => Self::Bar,
            Self::Colon => Self::Semicolon,
            Self::Apostrophe => Self::Quote,
            Self::Comma => Self::Less,
            Self::Period => Self::Greater,
            Self::Slash => Self::Question,
            other => other,
        }
    }
    pub fn unshift(self) -> Self {
        match self {
            Self::A => Self::a,
            Self::B => Self::b,
            Self::C => Self::c,
            Self::D => Self::d,
            Self::E => Self::e,
            Self::F => Self::f,
            Self::G => Self::g,
            Self::H => Self::h,
            Self::I => Self::i,
            Self::J => Self::j,
            Self::K => Self::k,
            Self::L => Self::l,
            Self::M => Self::m,
            Self::N => Self::n,
            Self::O => Self::o,
            Self::P => Self::p,
            Self::Q => Self::q,
            Self::R => Self::r,
            Self::S => Self::s,
            Self::T => Self::t,
            Self::U => Self::u,
            Self::V => Self::v,
            Self::W => Self::w,
            Self::X => Self::x,
            Self::Y => Self::y,
            Self::Z => Self::z,
            Self::Exclam => Self::Key1,
            Self::At => Self::Key2,
            Self::NumberSign => Self::Key3,
            Self::Dollar => Self::Key4,
            Self::Percent => Self::Key5,
            Self::Circum => Self::Key6,
            Self::Amp => Self::Key7,
            Self::Asterisk => Self::Key8,
            Self::LeftParen => Self::Key9,
            Self::RightParen => Self::Key0,
            Self::Tilde => Self::Backtick,
            Self::Underscore => Self::Minus,
            Self::Plus => Self::Equals,
            Self::LeftBrace => Self::LeftBracket,
            Self::RightBrace => Self::RightBracket,
            Self::Bar => Self::Backslash,
            Self::Semicolon => Self::Colon,
            Self::Quote => Self::Apostrophe,
            Self::Less => Self::Comma,
            Self::Greater => Self::Period,
            Self::Question => Self::Slash,
            other => other,
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers(pub(crate) u8);

impl Modifiers {
    pub const NONE: Self = Self(0);
    /// Shift Key
    pub const SHIFT: Self = Self(1);
    /// Control Key
    pub const CONTROL: Self = Self(2);
    /// Alt Key
    pub const ALT: Self = Self(4);
    /// Super / Windows / Command key
    pub const SUPER: Self = Self(8);
}

impl BitOr for Modifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for Modifiers {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl Not for Modifiers {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
