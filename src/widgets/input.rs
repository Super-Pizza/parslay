use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use lite_graphics::{Drawable, color::Rgba};

use crate::{
    app::{CursorType, HoverResult},
    reactive::SignalUpdate,
};
use crate::{event::Key, themes, window::Window};

use super::{
    InputEventFn, MouseEventFn, Offset, Size, WidgetBase, WidgetExt, WidgetInternal, label::Label,
};

pub trait InputBase {
    fn handle_key(&self, key: Key);
}

pub trait InputExt: InputBase {
    fn on_edit<F: FnMut(&Self) + 'static>(self: Rc<Self>, f: F) -> Rc<Self>;
}

pub struct Input {
    base: Label,

    default_bg: Cell<Rgba>,
    hovered_bg: Cell<Rgba>,
    clicked_bg: Cell<Rgba>,

    hovered: Cell<Option<Offset>>,
    clicked: Cell<bool>,

    hover_fn: RefCell<Box<MouseEventFn<Self>>>,
    edit_fn: RefCell<Box<InputEventFn<Self>>>,
    click_fn: RefCell<Box<MouseEventFn<Self>>>,
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
    fn get_text(&self) -> String {
        (&self.base as &dyn WidgetBase).get_text()
    }
}

impl InputBase for Input {
    fn handle_key(&self, key: Key) {
        let text = self.base.get_text();
        let string = key.to_string();
        text.update(|text| match key {
            Key::Backspace => text.remove_back(),
            Key::Delete => text.remove_front(),
            Key::ArrowLeft => text.move_h(-1),
            Key::ArrowRight => text.move_h(1),
            key if !string.is_empty() => text.insert(&key.to_string()[0..1]),
            _ => {}
        });
        (self.edit_fn.borrow_mut())(self)
    }
}
impl InputExt for Input {
    fn on_edit<F: FnMut(&Self) + 'static>(self: Rc<Self>, f: F) -> Rc<Self> {
        *self.edit_fn.borrow_mut() = Box::new(f);
        self
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

            hover_fn: RefCell::new(Box::new(|_, _| {})),
            edit_fn: RefCell::new(Box::new(|_| {})),
            click_fn: RefCell::new(Box::new(|_, _| {})),
        };
        Rc::new(this)
    }

    fn on_hover<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, f: F) -> Rc<Self> {
        *self.hover_fn.borrow_mut() = Box::new(f);
        self
    }
    fn on_click<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, f: F) -> Rc<Self> {
        *self.click_fn.borrow_mut() = Box::new(f);
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
    fn draw_frame(&self, _: &dyn Drawable) {}
    fn draw(&self, buf: &mut dyn Drawable) {
        if self.clicked.get() {
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

        let inside = pos.x >= 0
            && pos.y >= 0
            && pos.x <= self.get_size().w as i32
            && pos.y <= self.get_size().h as i32;

        if let Some(w) = pressed {
            self.clicked.set(inside);
            if inside {
                *w.focus.borrow_mut() = Some(self.clone());
            } else {
                return;
            }
            self.base.get_text().update(|text| {
                text.get_cursor_pos(
                    pos - Offset {
                        x: self.get_padding().3 as i32,
                        y: self.get_padding().0 as i32,
                    },
                )
            });
        } else {
            (self.click_fn.borrow_mut())(&self, pos)
        };

        // todo: add button handling!
    }
    fn handle_hover(self: Rc<Self>, pos: Offset) -> HoverResult {
        let pos = pos - self.get_offset();

        let is_hovered = self.hovered.get().is_some();

        if pos.x < 0
            || pos.y < 0
            || pos.x > self.get_size().w as i32
            || pos.y > self.get_size().h as i32
        {
            self.clicked.set(false);
            self.hovered.set(None);
            return HoverResult {
                redraw: is_hovered,
                cursor: CursorType::Arrow,
            };
        }

        (self.hover_fn.borrow_mut())(&self.clone(), pos);
        self.hovered.set(Some(pos));

        HoverResult {
            redraw: !is_hovered,
            cursor: CursorType::Text,
        }
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

        hover_fn: RefCell::new(Box::new(|_, _| {})),
        edit_fn: RefCell::new(Box::new(|_| {})),
        click_fn: RefCell::new(Box::new(|_, _| {})),
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

        hover_fn: RefCell::new(Box::new(|_, _| {})),
        edit_fn: RefCell::new(Box::new(|_| {})),
        click_fn: RefCell::new(Box::new(|_, _| {})),
    };
    Rc::new(this)
}
