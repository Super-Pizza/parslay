use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use lite_graphics::color::Rgba;

use crate::reactive::{SignalRead, SignalWrite};
use crate::{event::Key, themes, window::Window};

use super::{
    label::Label, Buffer, MouseEventFn, Offset, Size, WidgetBase, WidgetExt, WidgetInternal,
};

pub trait InputBase {
    fn handle_key(&self, key: Key);
}

pub struct Input {
    base: Label,

    default_bg: Cell<Rgba>,
    hovered_bg: Cell<Rgba>,
    clicked_bg: Cell<Rgba>,

    hovered: Cell<Option<Offset>>,
    clicked: Cell<bool>,
    cursor: Cell<Option<usize>>,

    hovered_fn: RefCell<Box<MouseEventFn<Self>>>,
    clicked_fn: RefCell<Box<MouseEventFn<Self>>>,
}

impl WidgetBase for Input {
    fn set_size(&self, size: Size) {
        self.base.set_size(size);
    }
    fn set_pos(&self, pos: Offset) {
        self.base.set_pos(pos);
    }
    fn set_frame(&self, frame: String) {
        self.base.set_frame(frame);
    }
    fn set_background_color(&self, color: Rgba) {
        self.base.set_background_color(color);
        self.default_bg.set(color);
    }
    fn set_padding(&self, padding: u32) {
        self.base.set_padding(padding);
    }
    fn set_border_radius(&self, radius: u32) {
        self.base.set_border_radius(radius);
    }
    fn set_color(&self, color: Rgba) {
        self.base.set_color(color);
    }
    fn set_text(&self, text: &str) {
        self.base.set_text(text);
    }
    fn get_background_color(&self) -> Rgba {
        self.base.get_background_color()
    }
    fn get_padding(&self) -> (u32, u32, u32, u32) {
        self.base.get_padding()
    }
    fn get_border_radius(&self) -> u32 {
        self.base.get_border_radius()
    }
}

impl InputBase for Input {
    fn handle_key(&self, key: Key) {
        let text = self.base.get_text();
        match key.to_string().as_str() {
            "\x08" => {
                text.write_only()
                    .write()
                    .borrow_mut()
                    .remove(self.cursor.get().unwrap() - 1);
                self.cursor.set(self.cursor.get().map(|v| v - 1))
            }
            "\x7f" => text
                .write_only()
                .write()
                .borrow_mut()
                .remove(self.cursor.get().unwrap()),
            s => {
                text.write_only()
                    .write()
                    .borrow_mut()
                    .insert(s, self.cursor.get().unwrap());
                self.cursor.set(self.cursor.get().map(|v| v + 1));
            }
        }
    }
}

impl WidgetExt for Input {
    fn new() -> Rc<Self> {
        let base = Label::new_internal();
        base.set_frame(themes::FrameType::InputFrame.to_string());
        let this = Input {
            base,
            default_bg: Cell::new(Rgba::WHITE),
            hovered_bg: Cell::new(Rgba::hex("#808080").unwrap()),
            clicked_bg: Cell::new(Rgba::hex("#a0a0a0").unwrap()),

            hovered: Cell::new(None),
            clicked: Cell::new(false),
            cursor: Cell::new(None),

            hovered_fn: RefCell::new(Box::new(|_, _| {})),
            clicked_fn: RefCell::new(Box::new(|_, _| {})),
        };
        Rc::new(this)
    }

    fn on_hover<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, f: F) -> Rc<Self> {
        *self.hovered_fn.borrow_mut() = Box::new(f);
        self
    }
    fn on_click<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, f: F) -> Rc<Self> {
        *self.clicked_fn.borrow_mut() = Box::new(f);
        self
    }
}

impl WidgetInternal for Input {
    fn compute_size(&self, font: ab_glyph::FontArc) {
        self.base.compute_size(font);
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&self, pos: Offset) {
        self.base.set_offset(pos);
    }
    fn get_frame(&self) -> themes::FrameFn {
        self.base.get_frame()
    }
    fn draw_frame(&self, _: &Buffer) {}
    fn draw(&self, buf: &Buffer) {
        if self.cursor.get().is_some() {
            self.base.set_background_color(self.clicked_bg.get());
        } else if self.hovered.get().is_some() {
            self.base.set_background_color(self.hovered_bg.get());
        } else {
            self.base.set_background_color(self.default_bg.get());
        }
        self.base.draw(buf);
    }

    fn handle_button(self: Rc<Self>, pos: Offset, pressed: Option<Rc<Window>>) {
        let pos = pos - self.get_offset();

        if pos.x < 0
            || pos.y < 0
            || pos.x > self.get_size().w as i32
            || pos.y > self.get_size().h as i32
        {
            return;
        }

        if let Some(w) = pressed {
            *w.focus.borrow_mut() = Some(self.clone());
            self.cursor.set(Some(
                self.base.get_text().read().borrow().get_cursor_pos(pos),
            ));
        } else {
            (self.clicked_fn.borrow_mut())(&self, pos)
        };

        // todo: add button handling!
    }
    fn handle_hover(self: Rc<Self>, pos: Offset) -> bool {
        let pos = pos - self.get_offset();

        let is_hovered = self.hovered.get().is_some();
        if pos.x < 0
            || pos.y < 0
            || pos.x > self.get_size().w as i32
            || pos.y > self.get_size().h as i32
        {
            self.clicked.set(false);
            self.hovered.set(None);
            return is_hovered;
        }

        (self.hovered_fn.borrow_mut())(&self.clone(), pos);
        self.hovered.set(Some(pos));

        !is_hovered
    }
}

pub fn input() -> Rc<Input> {
    let base = Label::new_internal();
    base.set_frame(themes::FrameType::InputFrame.to_string());
    let this = Input {
        base,

        default_bg: Cell::new(Rgba::WHITE),
        hovered_bg: Cell::new(Rgba::hex("#808080").unwrap()),
        clicked_bg: Cell::new(Rgba::hex("#a0a0a0").unwrap()),

        hovered: Cell::new(None),
        clicked: Cell::new(false),
        cursor: Cell::new(None),

        hovered_fn: RefCell::new(Box::new(|_, _| {})),
        clicked_fn: RefCell::new(Box::new(|_, _| {})),
    };
    Rc::new(this)
}

pub fn dyn_input<S: AsRef<str> + 'static>(label: impl Fn() -> S + 'static) -> Rc<Input> {
    let base = Label::new_dyn_internal(label);
    base.set_frame(themes::FrameType::InputFrame.to_string());
    let this = Input {
        base,

        default_bg: Cell::new(Rgba::WHITE),
        hovered_bg: Cell::new(Rgba::hex("#808080").unwrap()),
        clicked_bg: Cell::new(Rgba::hex("#a0a0a0").unwrap()),

        hovered: Cell::new(None),
        clicked: Cell::new(false),
        cursor: Cell::new(None),

        hovered_fn: RefCell::new(Box::new(|_, _| {})),
        clicked_fn: RefCell::new(Box::new(|_, _| {})),
    };
    Rc::new(this)
}
