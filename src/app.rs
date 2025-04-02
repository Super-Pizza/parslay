use std::{cell::RefCell, collections::HashMap, rc::Rc};

use lite_graphics::Offset;

use crate::{
    event::{Event, RawEvent, WidgetEvent},
    sys,
};

pub struct App {
    pub(crate) windows: RefCell<HashMap<u64, Rc<crate::Window>>>,
    pub(crate) inner: sys::app::App,
    pub(crate) font: ab_glyph::FontArc,
}

impl App {
    pub fn new() -> crate::Result<Rc<Self>> {
        let inner = sys::app::App::new()?;
        let font = sys::get_default_font()?;
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
                Event::Widget(WidgetEvent::ButtonPress(_, x, y)) => {
                    win.widget.borrow_mut().handle_click(Offset::new(x, y));
                    win.redraw()?;
                }
                Event::Widget(WidgetEvent::ButtonRelease(_, _, _)) => {
                    win.redraw()?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub(crate) fn add_window(&self, window: Rc<crate::Window>) {
        self.windows.borrow_mut().insert(window.inner.id(), window);
    }
}
