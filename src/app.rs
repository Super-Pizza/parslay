use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Read},
    rc::Rc,
};

use crate::sys;

pub struct App {
    pub(crate) windows: RefCell<HashMap<u64, Rc<crate::Window>>>,
    pub(crate) inner: sys::app::App,
    pub(crate) font: ab_glyph::FontArc,
}

impl App {
    pub fn new() -> crate::Result<Rc<Self>> {
        let inner = sys::app::App::new()?;
        let font = get_default_font()?;
        Ok(Rc::new(Self {
            windows: RefCell::new(HashMap::new()),
            inner,
            font,
        }))
    }
    pub fn run(&self) -> crate::Result<()> {
        self.inner.run()
    }

    pub(crate) fn add_window(&self, window: Rc<crate::Window>) {
        self.windows.borrow_mut().insert(window.inner.id(), window);
    }
}

fn get_default_font() -> crate::Result<ab_glyph::FontArc> {
    let mut font = sys::get_font(None)?.0;
    let mut buf = vec![];
    font.read_to_end(&mut buf)?;
    ab_glyph::FontArc::try_from_vec(buf)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Font Used").into())
}
