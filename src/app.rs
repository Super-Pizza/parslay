use std::{cell::RefCell, collections::HashMap, rc::Rc};

use lite_graphics::{color::Rgba, draw::Buffer, Offset, Size};

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
                    let redraw = win.widget.borrow().clone().handle_hover(Offset::new(x, y));
                    if redraw {
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
