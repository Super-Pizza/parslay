use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use lite_graphics::{Buffer, Drawable, Offset, Size, color::Rgba};

use crate::{
    app::{self, CursorType, HoverResult},
    themes,
    window::Window,
};

use super::{WidgetBase, WidgetExt, WidgetInternal};

pub struct Widget {
    size: Cell<Size>,
    pos: Cell<Offset>,
    frame: RefCell<themes::FrameFn>,
    padding: Cell<(u32, u32, u32, u32)>,
    bg_color: Cell<Rgba>,
    border_radius: Cell<u32>,
}

impl Widget {
    pub(crate) fn new_internal() -> Self {
        Self {
            size: Default::default(),
            pos: Default::default(),
            frame: RefCell::new(themes::NONE_FN.with(Rc::clone)),
            padding: Cell::new((0, 0, 0, 0)),
            bg_color: Cell::new(Rgba::WHITE),
            border_radius: Cell::new(0),
        }
    }
}

impl WidgetBase for Widget {
    fn set_size(&self, size: Size) {
        self.size.set(size);
    }
    fn set_pos(&self, pos: Offset) {
        self.pos.set(pos);
    }
    fn set_frame(&self, frame: String) {
        *self.frame.borrow_mut() = app::FRAMES.with_borrow(|map| {
            map.get(&frame)
                .map(Rc::clone)
                .unwrap_or(themes::NONE_FN.with(Rc::clone))
                .clone()
        })
    }
    fn set_background_color(&self, color: Rgba) {
        self.bg_color.set(color);
    }
    fn set_padding(&self, padding: u32) {
        self.padding.set([padding; 4].into());
    }
    fn set_border_radius(&self, radius: u32) {
        self.border_radius.set(radius);
    }
    fn set_color(&self, _color: Rgba) {}
    fn set_text(&self, _text: &str) {}
    fn get_background_color(&self) -> Rgba {
        self.bg_color.get()
    }
    fn get_padding(&self) -> (u32, u32, u32, u32) {
        self.padding.get()
    }
    fn get_border_radius(&self) -> u32 {
        self.border_radius.get()
    }
    fn get_text(&self) -> String {
        "".to_owned()
    }
}

impl WidgetExt for Widget {
    fn new() -> Rc<Self> {
        Rc::new(Self::new_internal())
    }

    fn on_hover<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, _f: F) -> Rc<Self> {
        self
    }
    fn on_click<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, _f: F) -> Rc<Self> {
        self
    }
}

impl WidgetInternal for Widget {
    fn compute_size(&self, _: ab_glyph::FontArc) {}
    fn get_size(&self) -> Size {
        self.size.get()
    }
    fn get_offset(&self) -> Offset {
        self.pos.get()
    }
    fn set_offset(&self, pos: Offset) {
        self.pos.set(pos);
    }
    fn get_frame(&self) -> themes::FrameFn {
        self.frame.borrow().clone()
    }
    fn draw_frame(&self, buf: &Buffer) {
        let frame = self.get_frame();
        frame(buf, self.size.get(), self.bg_color.get())
    }
    fn draw(&self, buf: &Buffer) {
        let bounds = (self.get_offset(), self.get_size()).into();
        let offs_buf = buf.subregion(bounds);
        self.draw_frame(&offs_buf);
    }
    fn handle_button(self: Rc<Self>, _: Offset, _: Option<Rc<Window>>) {}
    fn handle_hover(self: Rc<Self>, _: Offset) -> HoverResult {
        HoverResult {
            redraw: false,
            cursor: CursorType::Arrow,
        }
    }
}
