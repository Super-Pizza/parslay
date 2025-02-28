use std::ops::{BitAnd, BitOr, Not};

pub(crate) struct RawEvent {
    pub(crate) window: u64,
    pub(crate) event: Event,
}

#[derive(Debug)]
pub enum Event {
    Window(WindowEvent),
    Widget(WidgetEvent),
    Unknown,
}

#[derive(Debug)]
pub enum WindowEvent {
    /// Keyboard key press
    KeyPress(Modifiers, Key),
    /// Keyboard key release
    KeyRelease(Modifiers, Key),
    /// Sent when window changes state
    StateChange(WindowState),
    /// Sent when window is asked to close
    Closing,
}

#[derive(Clone, Copy, Debug)]
pub enum WindowState {
    Maximized,
    Fullscreen,
    Suspended,
    Activated,
}

#[derive(Debug)]
pub enum WidgetEvent {
    /// Sent when entering(true) or leaving(false)
    Hover(bool),
    /// Cursor is moving. (x,y) is in pixels
    Move(u32, u32),
    /// Cursor is moving while holding a button down. (x,y) is in pixels
    Drag(Button, u32, u32),
    /// Pointer button press. (x,y) is in pixels
    ButtonPress(Button, u32, u32),
    /// Pointer button release. (x,y) is in pixels
    ButtonRelease(Button, u32, u32),
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

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default)]
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
