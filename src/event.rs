use std::ops::{BitAnd, BitOr, Not};

pub(crate) struct RawEvent {
    pub(crate) window: u64,
    pub(crate) event: Event,
}

pub enum Event {
    Window(WindowEvent),
    Widget(WidgetEvent),
    Unknown,
}

pub enum WindowEvent {
    /// Sent when minimized(true) or when un-minimized(false)
    Minimized(bool),
    /// Sent when window is asked to close
    Closing,
}

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
    /// Keyboard key press
    KeyPress(Modifiers, Key),
    /// Keyboard key release
    KeyRelease(Modifiers, Key),
}

#[repr(u8)]
#[non_exhaustive]
pub enum Button {
    Left = 1,
    Middle = 3,
    Right = 2,
    Forward = 4,
    Back = 5,
}

pub enum Key {
    Escape,
    Tab,
    Space,
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
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Backtick,
    Minus,
    Equals,
    LeftBracket,
    RightBracket,
    Backslash,
    Colon,
    Apostrophe,
    Comma,
    Period,
    Slash,
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
    PauseBreak,
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
}

#[repr(transparent)]
pub struct Modifiers(u8);

impl Modifiers {
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
