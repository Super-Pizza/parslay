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
        self.base.set_size(size);
    }
    fn set_pos(&mut self, pos: Offset) {
        self.base.set_pos(pos);
    }
    fn set_background_color(&mut self, color: Rgba) {
        self.base.set_background_color(color);
        self.text.set_background_color(color);
    }
    fn set_padding(&mut self, padding: u32) {
        self.base.set_padding(padding);
    }
    fn set_border_radius(&mut self, radius: u32) {
        self.base.set_border_radius(radius);
    }
    fn get_backgounr_color(&self) -> Rgba {
        self.base.get_backgounr_color()
    }
    fn get_padding(&self) -> (u32, u32, u32, u32) {
        self.base.get_padding()
    }
    fn get_border_radius(&self) -> u32 {
        self.base.get_border_radius()
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
        let padding = self.get_padding();
        let base_size = Size {
            w: self.text.width_bounds().1 + padding.1 + padding.3,
            h: self.text.text_height() + padding.0 + padding.2,
        };
        self.base.set_size(base_size);
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
            Rect::from((self.get_offset(), self.get_size())),
            self.get_border_radius(),
            self.get_backgounr_color(),
        );
        let padding = self.get_padding();
        let pos = self.get_offset() + Offset::from((padding.3 as i32, padding.0 as i32));
        let size = self.get_size();
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
