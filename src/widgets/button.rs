use std::{cell::RefCell, rc::Rc};

use lite_graphics::draw::Rgba;

use super::{label, MosueEventFn};
use super::{Buffer, Offset, Size, WidgetBase, WidgetExt, WidgetInternal};

#[derive(Clone)]
pub struct Button {
    base: label::Label,

    clicked: Option<Offset>,
    hover_fn: Rc<RefCell<MosueEventFn<Self>>>,
    click_fn: Rc<RefCell<MosueEventFn<Self>>>,
}

impl WidgetBase for Button {
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

impl WidgetExt for Button {
    fn new() -> Self {
        Self {
            base: label::Label::new(),
            hover_fn: Rc::new(RefCell::new(|button: &mut Button, _| {
                button.set_background_color(Rgba::hex("#808080").unwrap())
            })),
            click_fn: Rc::new(RefCell::new(|button: &mut Button, _| {
                button.set_background_color(Rgba::hex("#a0a0a0").unwrap())
            })),
            clicked: None,
        }
    }

    fn on_hover<F: FnMut(&mut Self, Offset) + 'static>(&mut self, f: F) {
        self.hover_fn = Rc::new(RefCell::new(f));
    }
    fn on_click<F: FnMut(&mut Self, Offset) + 'static>(&mut self, f: F) {
        self.click_fn = Rc::new(RefCell::new(f));
    }
}

impl WidgetInternal for Button {
    fn compute_size(&mut self, font: ab_glyph::FontArc) {
        self.base.compute_size(font);
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
        if self.clicked.is_some() {
            let mut clicked_state = self.clone();
            (self.click_fn.borrow_mut())(&mut clicked_state, self.clicked.take().unwrap());
            // Prevents stack overflow
            clicked_state.clicked = None;
            clicked_state.draw(buf);
            return;
        }
        self.base.draw(buf);
    }
    #[allow(clippy::needless_return)]
    fn handle_click(&mut self, pos: Offset) {
        let pos = pos - self.get_offset();

        if pos.x < 0
            || pos.y < 0
            || pos.x > self.get_size().w as i32
            || pos.y > self.get_size().h as i32
        {
            return;
        }
        self.clicked = Some(pos);
        // todo: add button handling!
    }
}

pub fn button<S: AsRef<str>>(label: S) -> Button {
    Button {
        base: label::label(&label),
        hover_fn: Rc::new(RefCell::new(|button: &mut Button, _| {
            button.set_background_color(Rgba::hex("#808080").unwrap())
        })),
        click_fn: Rc::new(RefCell::new(|button: &mut Button, _| {
            button.set_background_color(Rgba::hex("#a0a0a0").unwrap())
        })),
        clicked: None,
    }
}
