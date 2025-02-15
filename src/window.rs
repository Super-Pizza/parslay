use lite_graphics::draw::Buffer;

use crate::{sys, WidgetExt};

#[derive(Clone)]
pub struct Window {
    pub(crate) inner: sys::window::Window,
    pub(crate) font: ab_glyph::FontArc,
}

impl Window {
    pub fn new(app: &mut crate::app::App) -> crate::Result<Self> {
        let inner = sys::window::Window::new(&app.inner)?;

        let font = app.font.clone();

        let this = Self { inner, font };
        Ok(this)
    }
    pub fn render<V: super::IntoView + 'static>(
        &mut self,
        f: impl Fn() -> V + 'static,
    ) -> crate::Result<()> {
        let view = f();
        let widget = view.create(self.clone());
        let buffer = Buffer::new(800, 600);
        widget.draw(&buffer);
        self.inner.draw(buffer)
    }
}
