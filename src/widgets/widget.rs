use lite_graphics::{
    draw::{Buffer, Rgba},
    Offset, Rect, Size,
};

use super::{WidgetBase, WidgetExt, WidgetInternal};

#[derive(Clone)]
pub struct Widget {
    size: Size,
    pos: Offset,
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
    fn set_background_color(&mut self, color: Rgba) {
        self.bg_color = color;
    }
    fn set_padding(&mut self, padding: u32) {
        self.padding = [padding; 4].into();
    }
    fn set_border_radius(&mut self, radius: u32) {
        self.border_radius = radius;
    }
    fn get_backgounr_color(&self) -> Rgba {
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
            padding: (0, 0, 0, 0),
            bg_color: Rgba::WHITE,
            border_radius: 0,
        }
    }
    fn on_hover<F: FnMut(&mut Self, Offset)>(&mut self, _f: F) {}
    fn on_click<F: FnMut(&mut Self, Offset)>(&mut self, _f: F) {}
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
    fn draw(&mut self, buf: &Buffer) {
        buf.fill_round_rect_aa(
            Rect::from((self.pos, self.size)),
            self.border_radius,
            self.bg_color,
        );
    }
    fn handle_click(&mut self, _: Offset) {}
    fn handle_hover(&mut self, _: Offset) -> bool {
        false
    }
}
