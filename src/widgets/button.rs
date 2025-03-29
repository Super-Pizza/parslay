use lite_graphics::draw::Rgba;

use super::label;
use super::{Buffer, Offset, Size, WidgetBase, WidgetExt, WidgetInternal};

pub struct Button {
    base: label::Label,
    base_clicked: label::Label,
    clicked: bool,
}

impl WidgetBase for Button {
    fn set_size(&mut self, size: Size) {
        self.base.set_size(size);
        self.base_clicked.set_size(size);
    }
    fn set_pos(&mut self, pos: Offset) {
        self.base.set_pos(pos);
        self.base_clicked.set_pos(pos);
    }
    fn set_background_color(&mut self, color: Rgba) {
        self.base.set_background_color(color);
    }
    fn set_padding(&mut self, padding: u32) {
        self.base.set_padding(padding);
        self.base_clicked.set_padding(padding);
    }
    fn set_border_radius(&mut self, radius: u32) {
        self.base.set_border_radius(radius);
        self.base_clicked.set_border_radius(radius);
    }
}

impl WidgetExt for Button {
    fn new() -> Self {
        Self {
            base: label::Label::new(),
            base_clicked: label::Label::new().background_color(Rgba::from([128, 128, 128, 255])),
            clicked: false,
        }
    }
}

impl WidgetInternal for Button {
    fn compute_size(&mut self, font: ab_glyph::FontArc) {
        self.base.compute_size(font.clone());
        self.base_clicked.compute_size(font);
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&mut self, pos: Offset) {
        self.base.set_offset(pos);
        self.base_clicked.set_offset(pos);
    }
    fn draw(&mut self, buf: &Buffer) {
        if self.clicked {
            self.clicked = false;
            self.base_clicked.draw(buf);
            return;
        }
        self.base.draw(buf);
    }
    #[allow(clippy::needless_return)]
    fn handle_click(&mut self, pos: Offset) {
        let pos = pos - self.base.get_offset();

        if pos.x < 0
            || pos.y < 0
            || pos.x > self.get_size().w as i32
            || pos.y > self.get_size().h as i32
        {
            return;
        }
        self.clicked = true;
        // todo: add button handling!
    }
}

pub fn button<S: AsRef<str>>(label: S) -> Button {
    Button {
        base: label::label(&label),
        base_clicked: label::label(&label).background_color(Rgba::from([128, 128, 128, 255])),
        clicked: false,
    }
}
