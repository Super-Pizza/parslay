use ab_glyph::{Font, ScaleFont};
use lite_graphics::{draw::Rgba, Rect};

use super::{Buffer, Offset, Size, Widget, WidgetBase, WidgetExt, WidgetInternal};

pub struct Label {
    base: Widget,
    color: Rgba,
}

impl WidgetBase for Label {
    fn set_label(&mut self, label: &str) {
        self.base.label = label.to_owned();
    }
    fn set_size(&mut self, size: Size) {
        self.base.size = size
    }
    fn set_pos(&mut self, pos: Offset) {
        self.base.pos = pos;
    }
    fn set_font_size(&mut self, size: f32) {
        self.base.font_size = size;
    }
    fn set_background_color(&mut self, color: Rgba) {
        self.base.background_color = color;
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
            color: Rgba::BLACK,
        }
    }
}

impl WidgetInternal for Label {
    fn compute_size(&mut self, font: ab_glyph::FontArc) {
        let text = &self.base.label;
        let mut cursor = 0;
        let mut max_y = 0;
        let mut min_y = i32::MAX;
        let scaled = font.as_scaled(font.pt_to_px_scale(self.base.font_size).unwrap());
        let mut iter = text.chars().peekable();
        while let Some(c) = iter.next() {
            let glyph_id = font.glyph_id(c);
            let next_c = *iter.peek().unwrap_or(&' ');
            let next_id = font.glyph_id(next_c);
            let glyph = glyph_id.with_scale_and_position(scaled.scale, (0i16, 0));
            if let Some(q) = font.outline_glyph(glyph) {
                let bounds = q.px_bounds();
                cursor += bounds.max.x as u32;
                max_y = max_y.max(bounds.max.y as i32);
                min_y = min_y.min(bounds.min.y as i32);
                cursor += scaled.kern(glyph_id, next_id) as u32;
            } else {
                cursor += scaled.h_advance(glyph_id) as u32;
            }
        }
        let padding = Size::from((
            self.base.padding.1 + self.base.padding.3,
            self.base.padding.0 + self.base.padding.2,
        ));
        self.base.size = Size::from((cursor + padding.w, (max_y - min_y) as u32 + padding.h));
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
    fn draw(&mut self, font: ab_glyph::FontArc, buf: &Buffer) {
        buf.fill_round_rect_aa(
            Rect::from((self.base.pos, self.base.size)),
            self.base.border_radius as i32,
            self.base.background_color,
        );
        let text = &self.base.label;
        let pos =
            self.base.pos + Offset::from((self.base.padding.3 as i32, self.base.padding.0 as i32));
        let mut cursor = 0;
        let scaled = font.as_scaled(font.pt_to_px_scale(self.base.font_size).unwrap());
        let mut iter = text.chars().peekable();
        while let Some(c) = iter.next() {
            let glyph_id = font.glyph_id(c);
            let next_c = *iter.peek().unwrap_or(&' ');
            let next_id = font.glyph_id(next_c);
            let glyph = glyph_id.with_scale_and_position(scaled.scale, (0i16, 0));
            let ascent = scaled.ascent() as i32;
            let descent = scaled.descent() as i32;
            if let Some(q) = font.outline_glyph(glyph) {
                let bounds = q.px_bounds();
                q.draw(|x, y, c| {
                    buf.point(
                        x as i32 + pos.x + cursor + bounds.min.x as i32,
                        y as i32 + pos.y + ascent + descent + bounds.min.y as i32,
                        self.base
                            .background_color
                            .lerp(self.color, (c * 255.0) as u8),
                    )
                });
                cursor += bounds.max.x as i32;
                cursor += scaled.kern(glyph_id, next_id) as i32;
            } else {
                cursor += scaled.h_advance(glyph_id) as i32;
            }
        }
    }
    fn handle_click(&mut self, _: Offset) {}
}

pub fn label<S: AsRef<str>>(label: S) -> Label {
    Label {
        base: Widget::new().label(label),
        color: Rgba::BLACK,
    }
}
