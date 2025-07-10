use std::{cell::RefCell, collections::HashMap, ops, rc::Rc};

use lite_graphics::{Offset, Size, color::Rgba, draw::Buffer};

use crate::{
    event::{Event, Modifiers, RawEvent, WidgetEvent},
    sys, themes,
};

thread_local! {
    pub(crate) static FRAMES: RefCell<HashMap<String, themes::FrameFn>> = RefCell::new(themes::get_default_theme());
}

pub struct App {
    pub(crate) windows: RefCell<HashMap<u64, Rc<crate::Window>>>,
    pub(crate) inner: sys::app::App,
    pub(crate) font: ab_glyph::FontArc,
}

impl App {
    pub fn new() -> crate::Result<Rc<Self>> {
        let inner = sys::app::App::new()?;
        let font: ab_glyph::FontArc = sys::get_default_font()?;
        Ok(Rc::new(Self {
            windows: RefCell::new(HashMap::new()),
            inner,
            font,
        }))
    }
    pub fn run(&self) -> crate::Result<()> {
        while let Some(ev) = self.inner.get_events()? {
            let RawEvent { window, event } = ev;
            let windows = self.windows.borrow_mut();
            let Some(win) = windows.get(&window) else {
                continue;
            };
            match event {
                Event::Window(crate::event::WindowEvent::Resize(w, h)) => {
                    win.resize(w, h);
                }
                Event::Window(crate::event::WindowEvent::KeyPress(mods, key)) => {
                    if mods & (Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER)
                        != Modifiers::NONE
                    {
                        // TODO: Shortcuts
                        continue;
                    }
                    let char = key.to_string();
                    if char == "\x1b" {
                        *win.focus.borrow_mut() = None;
                        continue;
                    }
                    if let Some(w) = win.focus.borrow_mut().as_mut() {
                        w.handle_key(key);
                        win.redraw()?;
                    }
                }
                Event::Widget(WidgetEvent::ButtonPress(_, x, y)) => {
                    *win.focus.borrow_mut() = None;
                    win.widget
                        .borrow()
                        .clone()
                        .handle_button(Offset::new(x, y), Some(win.clone()));
                    win.redraw()?;
                }
                Event::Widget(WidgetEvent::ButtonRelease(_, x, y)) => {
                    win.widget
                        .borrow()
                        .clone()
                        .handle_button(Offset::new(x, y), None);
                    win.redraw()?;
                }
                Event::Widget(WidgetEvent::Move(x, y)) => {
                    let result = win.widget.borrow().clone().handle_hover(Offset::new(x, y));
                    win.set_cursor(result.cursor);
                    if result.redraw {
                        win.redraw()?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Add a frame. Will not insert if one already exists.
    pub fn add_frame<F: Fn(&Buffer, Size, Rgba) + 'static>(name: String, f: F) {
        FRAMES.with_borrow_mut(|frames| {
            if let std::collections::hash_map::Entry::Vacant(e) = frames.entry(name) {
                e.insert(Rc::new(f));
            }
        });
    }

    pub(crate) fn add_window(&self, window: Rc<crate::Window>) {
        self.windows.borrow_mut().insert(window.inner.id(), window);
    }
}

pub struct HoverResult {
    pub redraw: bool,
    pub cursor: CursorType,
}

#[repr(u8)]
#[derive(Default, Hash, PartialEq, Eq, Clone, Copy)]
pub enum CursorType {
    #[default]
    Arrow = 0,
    Pointer = 1,
    Text = 2,
    NResize = 4,
    SResize = 8,
    EResize = 16,
    WResize = 32,
    NEResize = 20,
    NWResize = 36,
    SEResize = 24,
    SWResize = 40,
    NSResize = 12,
    EWResize = 48,
    Unknown = 255,
}

impl ops::BitOrAssign for HoverResult {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self {
            redraw: self.redraw | rhs.redraw,
            cursor: self.cursor | rhs.cursor,
        }
    }
}

impl TryFrom<u8> for CursorType {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Arrow),
            1 => Ok(Self::Pointer),
            2 => Ok(Self::Text),
            4 => Ok(Self::NResize),
            8 => Ok(Self::SResize),
            16 => Ok(Self::EResize),
            32 => Ok(Self::WResize),
            20 => Ok(Self::NEResize),
            36 => Ok(Self::NWResize),
            24 => Ok(Self::SEResize),
            40 => Ok(Self::SWResize),
            12 => Ok(Self::NSResize),
            48 => Ok(Self::EWResize),
            _ => Err(()),
        }
    }
}

impl ops::BitOr for CursorType {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        // Precedence: Unknown <  Arrow < Text < Pointer < -Resize < --Resize
        if self == rhs || rhs == Self::Unknown {
            self
        } else if self == Self::Unknown {
            rhs
        } else if rhs == Self::Arrow {
            self
        } else if self == Self::Arrow {
            rhs
        } else if (self == Self::Text && rhs == Self::Pointer)
            || (self == Self::Pointer && rhs == Self::Text)
        {
            Self::Pointer
        } else if matches!(
            self,
            Self::NEResize
                | Self::NWResize
                | Self::SEResize
                | Self::SWResize
                | Self::NSResize
                | Self::EWResize
        ) {
            self
        } else if matches!(
            rhs,
            Self::NEResize
                | Self::NWResize
                | Self::SEResize
                | Self::SWResize
                | Self::NSResize
                | Self::EWResize
        ) {
            rhs
        } else if matches!(
            self,
            Self::NResize | Self::SResize | Self::EResize | Self::WResize
        ) && matches!(
            rhs,
            Self::NResize | Self::SResize | Self::EResize | Self::WResize
        ) {
            Self::try_from(self as u8 | rhs as u8).unwrap()
        } else if !matches!(
            self,
            Self::NResize | Self::SResize | Self::EResize | Self::WResize
        ) {
            self
        } else {
            rhs
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for CursorType {
    fn to_string(&self) -> String {
        match self {
            CursorType::Unknown => "default".to_string(),
            CursorType::Arrow => "arrow".to_string(),
            CursorType::Pointer => "pointer".to_string(),
            CursorType::Text => "text".to_string(),
            CursorType::NResize => "n-resize".to_string(),
            CursorType::SResize => "s-resize".to_string(),
            CursorType::EResize => "e-resize".to_string(),
            CursorType::WResize => "w-resize".to_string(),
            CursorType::NEResize => "ne-resize".to_string(),
            CursorType::NWResize => "nw-resize".to_string(),
            CursorType::SEResize => "se-resize".to_string(),
            CursorType::SWResize => "sw-resize".to_string(),
            CursorType::NSResize => "ns-resize".to_string(),
            CursorType::EWResize => "ew-resize".to_string(),
        }
    }
}
