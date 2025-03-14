use std::rc::Rc;

use lite_graphics::{draw::Buffer, Offset};

use crate::{sys, WidgetExt};

#[derive(Clone)]
pub struct Window {
    pub(crate) inner: sys::window::Window,
    pub(crate) font: ab_glyph::FontArc,
}

impl Window {
    pub fn new(app: &Rc<crate::App>) -> crate::Result<Rc<Self>> {
        let inner = sys::window::Window::new(&app.inner)?;

        let font = app.font.clone();

        let this = Rc::new(Self { inner, font });

        app.add_window(this.clone());
        Ok(this)
    }
    pub fn render<W: WidgetExt + 'static>(&self, f: impl Fn() -> W + 'static) -> crate::Result<()> {
        let mut widget = f();
        let buffer = Buffer::new(800, 600);
        widget.compute_size(self.font.clone());
        widget.set_offset(Offset::default());
        widget.draw(self.font.clone(), &buffer);
        self.inner.draw(buffer)
    }
}
