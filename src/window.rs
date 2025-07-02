use std::{cell::RefCell, rc::Rc};

use lite_graphics::{draw::Buffer, Offset, Size};

use crate::{
    sys,
    widgets::{IntoWidget, Widget},
    WidgetBase, WidgetExt,
};

#[derive(Clone)]
pub struct Window {
    pub(crate) inner: sys::window::Window,
    pub(crate) font: ab_glyph::FontArc,
    pub(crate) widget: RefCell<Rc<dyn WidgetBase>>,
    pub(crate) size: RefCell<Size>,
}

impl Window {
    pub fn new(app: &Rc<crate::App>) -> crate::Result<Rc<Self>> {
        let inner = sys::window::Window::new(&app.inner)?;

        let font = app.font.clone();

        let this = Rc::new(Self {
            inner,
            font,
            widget: RefCell::new(Widget::new()),
            size: RefCell::new(Size::new(800, 600)),
        });

        app.add_window(this.clone());
        Ok(this)
    }
    pub fn render<W: IntoWidget + 'static>(
        &self,
        f: impl FnOnce() -> W + 'static,
    ) -> crate::Result<()> {
        let widget = f();
        *self.widget.borrow_mut() = widget.into();
        self.redraw()
    }
    pub fn resize(&self, w: u32, h: u32) {
        *self.size.borrow_mut() = Size::new(w, h);
    }
    pub fn redraw(&self) -> crate::Result<()> {
        let size = self.size.borrow();
        let buffer = Buffer::new(size.w as _, size.h as _);
        let widget = self.widget.borrow();

        widget.compute_size(self.font.clone());
        widget.set_offset(Offset::default());
        widget.draw(&buffer);
        self.inner.draw(buffer)
    }
}
