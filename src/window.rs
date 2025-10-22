use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use lite_graphics::{Offset, Overlay, Rect, color::Rgba, draw::Buffer};

use crate::{
    ComputedSize, Size, WidgetBase, WidgetExt,
    app::CursorType,
    button, sys, vstack,
    widgets::{IntoWidget, Widget, input::InputBase},
};

#[derive(Clone)]
pub struct Window {
    pub(crate) inner: sys::window::Window,
    pub(crate) font: ab_glyph::FontArc,
    pub(crate) widget: RefCell<Rc<dyn WidgetBase>>,
    pub(crate) focus: RefCell<Option<Rc<dyn InputBase>>>,
    pub(crate) size: RefCell<ComputedSize>,
    pub(crate) rclick_widget: RefCell<Rc<dyn WidgetBase>>,
    pub(crate) rclick_offset: Cell<Option<Offset>>,
}

impl Window {
    pub fn new(app: &Rc<crate::App>) -> crate::Result<Rc<Self>> {
        let inner = sys::window::Window::new(&app.inner)?;

        let font = app.font.clone();

        let this = Rc::new(Self {
            inner,
            font,
            widget: RefCell::new(Widget::new()),
            focus: RefCell::new(None),
            size: RefCell::new(ComputedSize::new(800, 600)),
            rclick_widget: RefCell::new(Widget::new()),
            rclick_offset: Cell::new(None),
        });
        let win = Rc::downgrade(&this);
        *this.rclick_widget.borrow_mut() = vstack(
            4,
            button("Quit...")
                .padding(4)
                .background_color(Rgba::SILVER)
                .on_click(move |_, _| win.upgrade().unwrap().destroy())
                .size(Size::stretch(1, 0)),
        )
        .padding(4)
        .background_color(Rgba::SILVER);

        app.add_window(this.clone());
        Ok(this)
    }
    pub fn render<W: IntoWidget + 'static>(
        &self,
        f: impl FnOnce() -> W + 'static,
    ) -> crate::Result<()> {
        let widget = f();
        *self.widget.borrow_mut() = widget.into_widget();
        self.widget.borrow().set_font(self.font.clone());
        self.rclick_widget.borrow().set_font(self.font.clone());
        self.redraw()
    }
    pub fn resize(&self, w: u32, h: u32) {
        *self.size.borrow_mut() = ComputedSize::new(w, h);
        self.redraw();
    }
    pub fn redraw(&self) -> crate::Result<()> {
        let size = self.size.borrow();
        let mut buffer = Buffer::new(size.w as _, size.h as _);
        let widget = self.widget.borrow();

        widget.set_width(size.w);
        widget.set_height(size.h);
        widget.set_offset(Offset::default());
        widget.draw(&mut buffer);
        widget.draw_overlays(&mut buffer);

        if let Some(offs) = self.rclick_offset.get() {
            let rclick_widget = self.rclick_widget.borrow();
            let width = rclick_widget.width_bounds().0;
            rclick_widget.set_width(width);
            let height = rclick_widget.height_bounds().0;
            rclick_widget.set_height(height);
            rclick_widget.set_offset(Offset::default());
            let mut rclick_overlay =
                Overlay::new(buffer, Rect::new(offs, rclick_widget.get_computed_size()));
            rclick_widget.draw(&mut rclick_overlay);

            let buffer = rclick_overlay.write();
            self.inner.draw(buffer)
        } else {
            self.inner.draw(buffer)
        }
    }
    pub fn set_cursor(&self, cursor: CursorType) {
        self.inner.set_cursor(cursor);
    }
    pub fn hide_menu(&self) {
        self.rclick_offset.set(None);
    }
    pub fn destroy(&self) {
        self.inner.destroy();
    }
}
