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
    pub fn render<V: super::IntoView + 'static>(
        &self,
        f: impl Fn() -> V + 'static,
    ) -> crate::Result<()> {
        let view = f();
        let mut widget = view.create(self.clone());
        let buffer = Buffer::new(800, 600);
        widget.compute_size();
        widget.set_pos(Offset::default());
        widget.draw(&buffer);
        self.inner.draw(buffer)
    }
}
