use std::rc::Rc;

use lite_graphics::{color::Rgba, draw::Buffer, Offset, Size};

use crate::{app, themes};

use super::{WidgetBase, WidgetExt, WidgetInternal};

#[derive(Clone)]
pub struct Widget {
    size: Size,
    pos: Offset,
    frame: themes::FrameFn,
    padding: (u32, u32, u32, u32),
    bg_color: Rgba,
    border_radius: u32,
}

impl WidgetBase for Widget {
    fn set_size(&mut self, size: Size) {
        self.size = size;
    }
    fn set_pos(&mut self, pos: Offset) {
        self.pos = pos;
    }
    fn set_frame(&mut self, frame: String) {
        self.frame = app::FRAMES.with_borrow(|map| {
            map.get(&frame)
                .map(Rc::clone)
                .unwrap_or(themes::NONE_FN.with(Rc::clone))
                .clone()
        })
    }
    fn set_background_color(&mut self, color: Rgba) {
        self.bg_color = color;
    }
    fn set_padding(&mut self, padding: u32) {
        self.padding = [padding; 4].into();
    }
    fn set_border_radius(&mut self, radius: u32) {
        self.border_radius = radius;
    }
    fn set_color(&mut self, _color: Rgba) {}
    fn set_text(&mut self, _text: &str) {}
    fn get_background_color(&self) -> Rgba {
        self.bg_color
    }
    fn get_padding(&self) -> (u32, u32, u32, u32) {
        self.padding
    }
    fn get_border_radius(&self) -> u32 {
        self.border_radius
    }
}

impl WidgetExt for Widget {
    fn new() -> Self {
        Self {
            size: Default::default(),
            pos: Default::default(),
            frame: themes::NONE_FN.with(Rc::clone),
            padding: (0, 0, 0, 0),
            bg_color: Rgba::WHITE,
            border_radius: 0,
        }
    }

    fn on_hover<F: FnMut(&mut Self, Offset) + 'static>(self, _f: F) -> Self {
        self
    }
    fn on_click<F: FnMut(&mut Self, Offset) + 'static>(self, _f: F) -> Self {
        self
    }
}

impl WidgetInternal for Widget {
    fn compute_size(&mut self, _: ab_glyph::FontArc) {}
    fn get_size(&self) -> Size {
        self.size
    }
    fn get_offset(&self) -> Offset {
        self.pos
    }
    fn set_offset(&mut self, pos: Offset) {
        self.pos = pos;
    }
    fn get_frame(&self) -> themes::FrameFn {
        self.frame.clone()
    }
    fn draw_frame(&mut self, buf: &Buffer) {
        let frame = self.get_frame();
        frame(buf, self.size, self.bg_color)
    }
    fn draw(&mut self, buf: &Buffer) {
        let bounds = (self.get_offset(), self.get_size()).into();
        let offs_buf = buf.subregion(bounds);
        self.draw_frame(&offs_buf);
    }
    fn handle_button(&mut self, _: Offset, _: bool) {}
    fn handle_hover(&mut self, _: Offset) -> bool {
        false
    }
}
