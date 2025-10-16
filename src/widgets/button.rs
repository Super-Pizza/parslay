use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use lite_graphics::{Drawable, color::Rgba};

use crate::{
    app::{CursorType, HoverResult},
    themes,
    window::Window,
};

use super::{IntoWidget, MouseEventFn, Offset, Size, WidgetBase, WidgetExt, WidgetInternal};

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

impl<W: WidgetBase> Button<W> {
    pub(crate) fn new_internal(label: Rc<W>) -> Rc<Self> {
        let this = Button {
            base: label,
            default_bg: Cell::new(Rgba::WHITE),
            hovered_bg: Cell::new(Rgba::hex("#808080").unwrap()),
            clicked_bg: Cell::new(Rgba::hex("#a0a0a0").unwrap()),
            hovered: Cell::new(None),
            clicked: Cell::new(None),
            hover_fn: RefCell::new(Box::new(|_, _| {})),
            click_fn: RefCell::new(Box::new(|_, _| {})),
        };
        this.base.set_frame(themes::FrameType::Button.to_string());
        this.base.set_padding(4);
        Rc::new(this)
    }
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
        self.base.set_color(color);
    }
    fn set_text(&self, text: &str) {
        self.base.set_text(text);
    }
    fn get_text(&self) -> String {
        self.base.get_text()
    }
    fn set_disabled(&self, disable: bool) {
        self.base.set_disabled(disable);
    }
    fn is_disabled(&self) -> bool {
        self.base.is_disabled()
    }
}

impl<W: WidgetExt> WidgetExt for Button<W> {
    fn new() -> Rc<Self> {
        let this = Button {
            base: W::new().frame(themes::FrameType::Button).padding(4),
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
    fn draw_frame(&self, _: &dyn Drawable) {}
    fn draw(&self, buf: &mut dyn Drawable) {
        if self.is_disabled() {
            self.base
                .set_background_color(Rgba::hex("#d0d0d0").unwrap());
        } else if self.clicked.get().is_some() {
            self.base.set_background_color(self.clicked_bg.get());
        } else if self.hovered.get().is_some() {
            self.base.set_background_color(self.hovered_bg.get());
        } else {
            self.base.set_background_color(self.default_bg.get());
        }
        self.base.draw(buf);
    }

    fn handle_button(self: Rc<Self>, pos: Offset, pressed: Option<Rc<Window>>) {
        if self.is_disabled() {
            return;
        }

        let pos = pos - self.get_offset();
        if pos.x < 0
            || pos.y < 0
            || pos.x > self.get_size().w as i32
            || pos.y > self.get_size().h as i32
        {
            return;
        }

        if pressed.is_none() {
            (self.click_fn.borrow_mut())(&self.clone(), pos)
        };
        self.clicked.set(pressed.map(|_| pos));
        // todo: add button handling!
    }
    fn handle_hover(self: Rc<Self>, pos: Offset) -> HoverResult {
        let is_hovered = self.hovered.get().is_some();
        if self.is_disabled() {
            self.hovered.set(None);
            return HoverResult {
                redraw: is_hovered,
                cursor: CursorType::Arrow,
            };
        }

        let pos = pos - self.get_offset();
        if pos.x < 0
            || pos.y < 0
            || pos.x > self.get_size().w as i32
            || pos.y > self.get_size().h as i32
        {
            self.clicked.set(None);
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
            cursor: CursorType::Pointer,
        }
    }
}

pub fn button<W: IntoWidget>(base: W) -> Rc<Button<W::W>> {
    let this = Button {
        base: base.into_widget(),
        default_bg: Cell::new(Rgba::WHITE),
        hovered_bg: Cell::new(Rgba::hex("#808080").unwrap()),
        clicked_bg: Cell::new(Rgba::hex("#a0a0a0").unwrap()),
        hovered: Cell::new(None),
        clicked: Cell::new(None),
        hover_fn: RefCell::new(Box::new(|_, _| {})),
        click_fn: RefCell::new(Box::new(|_, _| {})),
    };
    this.base.set_frame(themes::FrameType::Button.to_string());
    this.base.set_padding(4);
    Rc::new(this)
}
