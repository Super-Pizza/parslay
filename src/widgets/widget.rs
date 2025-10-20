use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use lite_graphics::{Drawable, Offset, Rect, color::Rgba};

use crate::{
    Sizing,
    app::{CursorType, FRAMES, HoverResult},
    reactive::{RwSignal, SignalGet, SignalUpdate},
    themes,
    window::Window,
};

use super::{ComputedSize, Size, WidgetBase, WidgetExt, WidgetInternal};

pub struct Widget {
    size: Cell<Size>,
    computed_size: Cell<ComputedSize>,
    pos: Cell<Offset>,
    frame: RefCell<themes::FrameFn>,
    padding: Cell<(u32, u32, u32, u32)>,
    bg_color: Cell<Rgba>,
    border_radius: Cell<u32>,
    disabled: RwSignal<bool>,
}

impl Widget {
    pub(crate) fn new_internal() -> Self {
        Self {
            size: Default::default(),
            computed_size: Default::default(),
            pos: Default::default(),
            frame: RefCell::new(themes::NONE_FN.with(Rc::clone)),
            padding: Cell::new((0, 0, 0, 0)),
            bg_color: Cell::new(Rgba::WHITE),
            border_radius: Default::default(),
            disabled: RwSignal::new(false),
        }
    }
}

impl WidgetBase for Widget {
    fn set_size(&self, size: Size) {
        self.size.set(size);
    }
    fn get_size(&self) -> Size {
        self.size.get()
    }
    fn set_pos(&self, pos: Offset) {
        self.pos.set(pos);
    }
    fn set_frame(&self, frame: String) {
        *self.frame.borrow_mut() = FRAMES.with_borrow(|map| {
            map.get(&frame)
                .map(Rc::clone)
                .unwrap_or(themes::NONE_FN.with(Rc::clone))
                .clone()
        })
    }
    fn set_background_color(&self, color: Rgba) {
        self.bg_color.set(color);
    }
    fn get_background_color(&self) -> Rgba {
        self.bg_color.get()
    }
    fn set_padding(&self, padding: u32) {
        self.padding.set([padding; 4].into());
    }
    fn get_padding(&self) -> (u32, u32, u32, u32) {
        self.padding.get()
    }
    fn set_border_radius(&self, radius: u32) {
        self.border_radius.set(radius);
    }
    fn get_border_radius(&self) -> u32 {
        self.border_radius.get()
    }
    fn set_color(&self, _color: Rgba) {}
    fn set_text(&self, _text: &str) {}
    fn get_text(&self) -> String {
        "".to_owned()
    }
    fn set_disabled(&self, disable: bool) {
        self.disabled.set(disable);
    }
    fn is_disabled(&self) -> bool {
        self.disabled.get()
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
    fn set_font(&self, _: ab_glyph::FontArc) {}
    fn width_bounds(&self) -> (u32, u32) {
        let padding = self.get_padding();
        match self.get_size().w {
            Sizing::Fixed(w) => (w, w),
            _ => (padding.1 + padding.3, padding.1 + padding.3),
        }
    }
    fn set_width(&self, width: u32) {
        self.computed_size
            .update(|s| ComputedSize { w: width, h: s.h });
    }
    fn height_bounds(&self) -> (u32, u32) {
        let padding = self.get_padding();
        match self.get_size().h {
            Sizing::Fixed(h) => (h, h),
            _ => (padding.0 + padding.2, padding.0 + padding.2),
        }
    }
    fn set_height(&self, height: u32) {
        self.computed_size
            .update(|s| ComputedSize { w: s.w, h: height });
    }
    fn get_computed_size(&self) -> ComputedSize {
        self.computed_size.get()
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
    fn draw_frame(&self, buf: &dyn Drawable) {
        let frame = self.get_frame();
        frame(buf, self.computed_size.get(), self.bg_color.get())
    }
    fn draw(&self, buf: &mut dyn Drawable) {
        let bounds = Rect::new(self.get_offset(), self.get_computed_size());
        buf.subregion(bounds);
        self.draw_frame(buf);
        buf.end_subregion();
    }
    fn handle_button(self: Rc<Self>, _: Offset, _: Option<Rc<Window>>) {}
    fn handle_hover(self: Rc<Self>, _: Offset) -> HoverResult {
        HoverResult {
            redraw: false,
            cursor: CursorType::Arrow,
        }
    }
}
