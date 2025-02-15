use std::io::{self, Read};

use crate::sys;

pub struct App {
    pub(crate) inner: sys::app::App,
    pub(crate) font: ab_glyph::FontArc,
}

impl App {
    pub fn new() -> crate::Result<Self> {
        let inner = sys::app::App::new()?;
        let font = get_default_font()?;
        Ok(Self { inner, font })
    }
    pub fn run(&mut self) -> crate::Result<()> {
        self.inner.run()
    }
}

fn get_default_font() -> crate::Result<ab_glyph::FontArc> {
    let mut font = sys::get_font(None)?.0;
    let mut buf = vec![];
    font.read_to_end(&mut buf)?;
    ab_glyph::FontArc::try_from_vec(buf)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Font Used").into())
}
