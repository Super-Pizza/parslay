use std::rc::Rc;

use lite_graphics::{Drawable, Rect, color::Rgba};

use crate::{
    Sizing,
    app::{CursorType, HoverResult},
    reactive::{RwSignal, SignalGet as _, SignalUpdate as _, create_effect},
    text::Text,
    window::Window,
};

use super::{ComputedSize, Offset, Size, Widget, WidgetBase, WidgetExt, WidgetInternal};

pub struct Label {
    base: Widget,
    text: RwSignal<Text>,
}

impl Label {
    pub(crate) fn new_internal() -> Self {
        let text = RwSignal::new(Text::new("", 12.0));
        Self {
            base: Widget::new_internal(),
            text,
        }
    }
    pub(crate) fn new_dyn_internal<S: AsRef<str> + 'static>(
        label: impl Fn() -> S + 'static,
    ) -> Self {
        let text = RwSignal::new(Text::new("", 12.0));
        create_effect(move |_| text.update(|text| text.set_text(label())));
        Self {
            base: Widget::new_internal(),
            text,
        }
    }
    pub(crate) fn get_text_signal(&self) -> RwSignal<Text> {
        self.text
    }
}

impl WidgetBase for Label {
    fn set_size(&self, size: Size) {
        self.base.set_size(size);
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn set_pos(&self, pos: Offset) {
        self.base.set_pos(pos);
    }
    fn set_frame(&self, frame: String) {
        self.base.set_frame(frame);
    }
    fn set_background_color(&self, color: Rgba) {
        self.base.set_background_color(color);
    }
    fn get_background_color(&self) -> Rgba {
        self.base.get_background_color()
    }
    fn set_padding(&self, padding: u32) {
        self.base.set_padding(padding);
    }
    fn get_padding(&self) -> (u32, u32, u32, u32) {
        self.base.get_padding()
    }
    fn set_border_radius(&self, radius: u32) {
        self.base.set_border_radius(radius);
    }
    fn get_border_radius(&self) -> u32 {
        self.base.get_border_radius()
    }
    fn set_color(&self, color: Rgba) {
        self.text.update(move |text| text.set_color(color));
    }
    fn set_text(&self, string: &str) {
        self.text.update(move |text| text.set_text(string));
    }
    fn get_text(&self) -> String {
        self.text.get().get_text().to_owned()
    }
    fn set_text_align(&self, align: core::fmt::Alignment) {
        self.text.update(move |text| text.set_align(align));
    }
    fn set_disabled(&self, disable: bool) {
        self.base.set_disabled(disable);
    }
    fn is_disabled(&self) -> bool {
        self.base.is_disabled()
    }
}

impl WidgetExt for Label {
    fn new() -> Rc<Self> {
        Rc::new(Label::new_internal())
    }

    fn on_hover<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, _f: F) -> Rc<Self> {
        self
    }
    fn on_click<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, _f: F) -> Rc<Self> {
        self
    }
}

impl WidgetInternal for Label {
    fn set_font(&self, font: ab_glyph::FontArc) {
        self.text.update(|text| text.set_font(font));
    }
    fn width_bounds(&self) -> (u32, u32) {
        let padding = self.get_padding();
        let widths = self.text.get().width_bounds();
        match self.get_size().w {
            Sizing::Fixed(w) => (
                w.max(widths.0 + padding.1 + padding.3),
                w.max(widths.0 + padding.1 + padding.3),
            ),
            _ => (
                widths.0 + padding.1 + padding.3,
                widths.1 + padding.1 + padding.3,
            ),
        }
    }
    fn set_width(&self, width: u32) {
        self.base.set_width(width);
    }
    fn height_bounds(&self) -> (u32, u32) {
        let padding = self.get_padding();
        let height = self.text.get().text_height();
        match self.get_size().h {
            Sizing::Fixed(h) => (
                h.max(height + padding.0 + padding.2),
                h.max(height + padding.0 + padding.2),
            ),
            _ => (
                height + padding.1 + padding.3,
                height + padding.1 + padding.3,
            ),
        }
    }
    fn set_height(&self, height: u32) {
        self.base.set_height(height);
    }
    fn get_computed_size(&self) -> ComputedSize {
        self.base.get_computed_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&self, pos: Offset) {
        self.base.set_offset(pos);
    }
    fn get_frame(&self) -> crate::themes::FrameFn {
        self.base.get_frame()
    }
    fn draw_frame(&self, buf: &dyn Drawable) {
        let frame = self.get_frame();
        frame(buf, self.get_computed_size(), self.get_background_color())
    }
    fn draw(&self, buf: &mut dyn Drawable) {
        let bounds = Rect::new(self.get_offset(), self.get_computed_size());
        buf.subregion(bounds);
        self.draw_frame(buf);

        let padding = self.get_padding();
        let text_bounds = Rect::from((
            Offset::from((padding.3 as i32, padding.0 as i32)),
            ComputedSize::new(
                bounds.w - padding.1 - padding.3,
                bounds.h - padding.0 - padding.2,
            ),
        ));
        self.text
            .update(|text| text.draw(buf, text_bounds).unwrap_or_default());

        buf.end_subregion();
    }

    fn handle_button(self: Rc<Self>, _: Offset, _: Option<Rc<Window>>) {}
    fn handle_hover(self: Rc<Self>, pos: Offset) -> HoverResult {
        let pos = pos - self.get_offset();
        let size = self.get_computed_size();
        if pos.x < 0 || pos.y < 0 || pos.x > size.w as i32 || pos.y > size.h as i32 {
            return HoverResult {
                redraw: false,
                cursor: CursorType::Arrow,
            };
        }
        HoverResult {
            redraw: false,
            cursor: CursorType::Text,
        }
    }
}

pub fn label<S: AsRef<str> + 'static>(label: S) -> Rc<Label> {
    let text = RwSignal::new(Text::new(label.as_ref(), 12.0));
    Rc::new(Label {
        base: Widget::new_internal(),
        text,
    })
}

pub fn dyn_label<S: AsRef<str> + 'static>(label: impl Fn() -> S + 'static) -> Rc<Label> {
    let text = RwSignal::new(Text::new("", 12.0));
    create_effect(move |_| text.update(|text| text.set_text(label())));
    Rc::new(Label {
        base: Widget::new_internal(),
        text,
    })
}
