use ab_glyph::{Font, ScaleFont};
use lite_graphics::draw::Rgba;

use crate::IntoView;

use super::{Buffer, Offset, Size, Widget, WidgetBase, WidgetExt, WidgetView};

pub struct LabelView {
    base: WidgetView,
}

pub struct Label {
    base: Widget,
}

impl IntoView for LabelView {
    type Widget = Label;
    fn create(self, window: crate::window::Window) -> Self::Widget
    where
        Self::Widget: super::WidgetExt,
    {
        Label {
            base: self.base.create(window),
        }
    }
}

impl WidgetBase for LabelView {
    fn label<S: AsRef<str>>(mut self, label: S) -> Self {
        self.base.label = label.as_ref().to_owned();
        self
    }
    fn size<S: Into<Size>>(mut self, size: S) -> Self {
        self.base.size = size.into();
        self
    }
    fn pos<P: Into<Offset>>(mut self, pos: P) -> Self {
        self.base.pos = pos.into();
        self
    }
    fn font_size<S: Into<f32>>(mut self, size: S) -> Self {
        self.base.font_size = size.into();
        self
    }
}

impl WidgetExt for Label {
    fn compute_size(&mut self) {
        let window = &self.base.window;
        let text = &self.base.label;
        let mut cursor = 0;
        let mut max_y = 0;
        let font = &window.font;
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
                max_y = max_y.max((bounds.max.y - bounds.min.y) as u32);
                cursor += scaled.kern(glyph_id, next_id) as u32;
            } else {
                cursor += scaled.h_advance(glyph_id) as u32;
            }
        }
        self.base.size = Size::from((cursor, max_y));
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn set_pos(&mut self, pos: Offset) {
        self.base.set_pos(pos);
    }
    fn draw(&self, buf: &Buffer) {
        let window = &self.base.window;
        let text = &self.base.label;
        let pos = self.base.pos;
        let mut cursor = 0;
        let font = &window.font;
        let scaled = font.as_scaled(font.pt_to_px_scale(self.base.font_size).unwrap());
        let mut iter = text.chars().peekable();
        while let Some(c) = iter.next() {
            let glyph_id = font.glyph_id(c);
            let next_c = *iter.peek().unwrap_or(&' ');
            let next_id = font.glyph_id(next_c);
            let glyph = glyph_id.with_scale_and_position(scaled.scale, (0i16, 0));
            let ascent = scaled.ascent() as i32;
            if let Some(q) = font.outline_glyph(glyph) {
                let bounds = q.px_bounds();
                q.draw(|x, y, c| {
                    buf.point(
                        x as i32 + pos.x + cursor + bounds.min.x as i32,
                        y as i32 + pos.y + ascent + bounds.min.y as i32,
                        Rgba::from([(255.0 - c * 255.0) as u8; 3]),
                    )
                });
                cursor += bounds.max.x as i32;
                cursor += scaled.kern(glyph_id, next_id) as i32;
            } else {
                cursor += scaled.h_advance(glyph_id) as i32;
            }
        }
    }
}

pub fn label<S: AsRef<str>>(label: S) -> LabelView {
    LabelView {
        base: WidgetView::new().label(label),
    }
}
