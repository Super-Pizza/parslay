use lite_graphics::draw::Rgba;

use super::label;
use super::{Buffer, Offset, Size, WidgetBase, WidgetExt, WidgetInternal};

pub struct Button {
    base: label::Label,
}

impl WidgetBase for Button {
    fn set_label(&mut self, label: &str) {
        self.base.set_label(label);
    }
    fn set_size(&mut self, size: Size) {
        self.base.set_size(size);
    }
    fn set_pos(&mut self, pos: Offset) {
        self.base.set_pos(pos);
    }
    fn set_font_size(&mut self, size: f32) {
        self.base.set_font_size(size);
    }
    fn set_background_color(&mut self, color: Rgba) {
        self.base.set_background_color(color);
    }
    fn set_padding(&mut self, padding: u32) {
        self.base.set_padding(padding);
    }
    fn set_border_radius(&mut self, radius: u32) {
        self.base.set_border_radius(radius);
    }
}

impl WidgetExt for Button {
    fn new() -> Self {
        Self {
            base: label::Label::new(),
        }
    }
}

impl WidgetInternal for Button {
    fn compute_size(&mut self, font: ab_glyph::FontArc) {
        self.base.compute_size(font);
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn set_offset(&mut self, pos: Offset) {
        self.base.set_offset(pos);
    }
    fn draw(&self, font: ab_glyph::FontArc, buf: &Buffer) {
        self.base.draw(font, buf);
    }
}

pub fn button<S: AsRef<str>>(label: S) -> Button {
    Button {
        base: label::label(label),
    }
}
