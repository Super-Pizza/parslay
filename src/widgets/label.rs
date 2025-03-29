use lite_graphics::{draw::Rgba, Rect};

use crate::text::Text;

use super::{Buffer, Offset, Size, Widget, WidgetBase, WidgetExt, WidgetInternal};

pub struct Label {
    base: Widget,
    text: Text,
}

impl Label {
    pub fn set_color(&mut self, color: Rgba) {
        self.text.set_color(color);
    }
}

impl WidgetBase for Label {
    fn set_size(&mut self, size: Size) {
        self.base.size = size
    }
    fn set_pos(&mut self, pos: Offset) {
        self.base.pos = pos;
    }
    fn set_background_color(&mut self, color: Rgba) {
        self.base.bg_color = color;
        self.text.set_background_color(color);
    }
    fn set_padding(&mut self, padding: u32) {
        self.base.padding = [padding; 4].into();
    }
    fn set_border_radius(&mut self, radius: u32) {
        self.base.border_radius = radius;
    }
}

impl WidgetExt for Label {
    fn new() -> Self {
        Self {
            base: Widget::new(),
            text: Text::new("", 12.0),
        }
    }
}

impl WidgetInternal for Label {
    fn compute_size(&mut self, font: ab_glyph::FontArc) {
        self.text.get_text_size(font);
        self.base.size.w = self.text.width_bounds().1;
        self.base.size.h = self.text.text_height();
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&mut self, pos: Offset) {
        self.base.set_offset(pos);
    }
    fn draw(&mut self, buf: &Buffer) {
        buf.fill_round_rect_aa(
            Rect::from((self.base.pos, self.base.size)),
            self.base.border_radius as i32,
            self.base.bg_color,
        );
        let pos =
            self.base.pos + Offset::from((self.base.padding.3 as i32, self.base.padding.0 as i32));
        let size = self.base.size;
        self.text.draw(buf, Rect::from((pos, size)));
    }
    fn handle_click(&mut self, _: Offset) {}
}

pub fn label<S: AsRef<str>>(label: S) -> Label {
    Label {
        base: Widget::new(),
        text: Text::new(label, 12.0),
    }
}
