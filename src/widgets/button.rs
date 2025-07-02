use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use lite_graphics::color::Rgba;

use crate::themes;

use super::{
    Buffer, IntoWidget, MouseEventFn, Offset, Size, WidgetBase, WidgetExt, WidgetInternal,
};

pub struct Button<W> {
    base: Rc<W>,

    default_bg: Cell<Rgba>,
    hovered_bg: Cell<Rgba>,
    clicked_bg: Cell<Rgba>,

    hovered: Cell<Option<Offset>>,
    clicked: Cell<Option<Offset>>,

    hover_fn: RefCell<Box<MouseEventFn<Self>>>,
    click_fn: RefCell<Box<MouseEventFn<Self>>>,
}

impl<W: WidgetBase> WidgetBase for Button<W> {
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

impl<W: WidgetExt> WidgetExt for Button<W> {
    fn new() -> Rc<Self> {
        let this = Button {
            base: W::new().frame(themes::FrameType::Button),
            default_bg: Cell::new(Rgba::WHITE),
            hovered_bg: Cell::new(Rgba::hex("#808080").unwrap()),
            clicked_bg: Cell::new(Rgba::hex("#a0a0a0").unwrap()),
            hovered: Cell::new(None),
            clicked: Cell::new(None),
            hover_fn: RefCell::new(Box::new(|_, _| {})),
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

impl<W: WidgetBase> WidgetInternal for Button<W> {
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
        if self.clicked.get().is_some() {
            self.base.set_background_color(self.clicked_bg.get());
        } else if self.hovered.get().is_some() {
            self.base.set_background_color(self.hovered_bg.get());
        } else {
            self.base.set_background_color(self.default_bg.get());
        }
        self.base.draw(buf);
    }

    fn handle_button(self: Rc<Self>, pos: Offset, pressed: bool) {
        let pos = pos - self.get_offset();

        if pos.x < 0
            || pos.y < 0
            || pos.x > self.get_size().w as i32
            || pos.y > self.get_size().h as i32
        {
            return;
        }

        if !pressed {
            (self.click_fn.borrow_mut())(&self.clone(), pos)
        };
        self.clicked.set(pressed.then_some(pos));
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
            self.clicked.set(None);
            self.hovered.set(None);
            return is_hovered;
        }

        (self.hover_fn.borrow_mut())(&self.clone(), pos);
        self.hovered.set(Some(pos));

        !is_hovered
    }
}

pub fn button<W: IntoWidget>(base: W) -> Rc<Button<W::W>> {
    let this = Button {
        base: base.into(),
        default_bg: Cell::new(Rgba::WHITE),
        hovered_bg: Cell::new(Rgba::hex("#808080").unwrap()),
        clicked_bg: Cell::new(Rgba::hex("#a0a0a0").unwrap()),
        hovered: Cell::new(None),
        clicked: Cell::new(None),
        hover_fn: RefCell::new(Box::new(|_, _| {})),
        click_fn: RefCell::new(Box::new(|_, _| {})),
    };
    Rc::new(this)
}
