use crate::reactive::{create_effect, RwSignal, SignalGet as _, SignalUpdate as _};
use lite_graphics::{draw::Rgba, Rect};

use crate::text::Text;

use super::{Buffer, Offset, Size, Widget, WidgetBase, WidgetExt, WidgetInternal};

#[derive(Clone)]
pub struct Label {
    base: Widget,
    text: RwSignal<Text>,
}

impl Label {
    pub fn set_color(&mut self, color: Rgba) {
        self.text.update(move |text| text.set_color(color));
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
    }
    fn set_padding(&mut self, padding: u32) {
        self.base.set_padding(padding);
    }
    fn set_border_radius(&mut self, radius: u32) {
        self.base.set_border_radius(radius);
    }
    fn set_text(&mut self, string: &str) {
        self.text.update(move |text| text.set_text(string));
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
        let signal = RwSignal::new(Text::new("", 12.0));
        Self {
            base: Widget::new(),
            text: signal,
        }
    }

    fn on_hover<F: FnMut(&mut Self, Offset) + 'static>(self, _f: F) -> Self {
        self
    }
    fn on_click<F: FnMut(&mut Self, Offset) + 'static>(self, _f: F) -> Self {
        self
    }
}

impl WidgetInternal for Label {
    fn compute_size(&mut self, font: ab_glyph::FontArc) {
        self.text.update(|text| text.get_text_size(font));
        let padding = self.get_padding();
        let base_size = Size {
            w: self.text.get().width_bounds().1 + padding.1 + padding.3,
            h: self.text.get().text_height() + padding.0 + padding.2,
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
        self.text.get().draw(
            buf,
            Rect::from((pos, size)),
            self.base.get_backgounr_color(),
        );
    }
    fn handle_click(&mut self, _: Offset) {}
    fn handle_hover(&mut self, _: Offset) -> bool {
        false
    }
}

pub fn label<S: AsRef<str> + 'static>(label: S) -> Label {
    let text = RwSignal::new(Text::new(label.as_ref(), 12.0));
    Label {
        base: Widget::new(),
        text,
    }
}

pub fn dyn_label<S: AsRef<str> + 'static>(label: impl Fn() -> S + 'static) -> Label {
    let text = RwSignal::new(Text::new("", 12.0));
    create_effect(move |_| text.update(|text| text.set_text(label())));
    Label {
        base: Widget::new(),
        text,
    }
}
