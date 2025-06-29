use std::{cell::RefCell, rc::Rc};

use lite_graphics::color::Rgba;

use crate::themes;

use super::{
    Buffer, IntoWidget, MouseEventFn, Offset, Size, WidgetBase, WidgetExt, WidgetInternal,
};

#[derive(Clone)]
pub struct Button<W: WidgetBase> {
    base: Box<W>,

    default_bg: Rgba,
    hovered_bg: Rgba,
    clicked_bg: Rgba,

    hovered: Option<Offset>,
    clicked: Option<Offset>,

    hover_fn: Rc<RefCell<MouseEventFn<Self>>>,
    click_fn: Rc<RefCell<MouseEventFn<Self>>>,
}

impl<W: WidgetBase + Clone> WidgetBase for Button<W> {
    fn set_size(&mut self, size: Size) {
        self.base.set_size(size);
    }
    fn set_pos(&mut self, pos: Offset) {
        self.base.set_pos(pos);
    }
    fn set_frame(&mut self, frame: String) {
        self.base.set_frame(frame);
    }
    fn set_background_color(&mut self, color: Rgba) {
        self.base.set_background_color(color);
        self.default_bg = color;
    }
    fn set_padding(&mut self, padding: u32) {
        self.base.set_padding(padding);
    }
    fn set_border_radius(&mut self, radius: u32) {
        self.base.set_border_radius(radius);
    }
    fn set_color(&mut self, color: Rgba) {
        self.base.set_color(color);
    }
    fn set_text(&mut self, text: &str) {
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

impl<W: WidgetExt + Clone> WidgetExt for Button<W> {
    fn new() -> Self {
        Self {
            base: Box::new(W::new().frame(themes::FrameType::Button)),
            default_bg: Rgba::WHITE,
            hovered_bg: Rgba::hex("#808080").unwrap(),
            clicked_bg: Rgba::hex("#a0a0a0").unwrap(),
            hovered: None,
            clicked: None,
            hover_fn: Rc::new(RefCell::new(|_button: &mut Button<W>, _| {})),
            click_fn: Rc::new(RefCell::new(|_button: &mut Button<W>, _| {})),
        }
    }

    fn on_hover<F: FnMut(&mut Self, Offset) + 'static>(mut self, f: F) -> Self {
        self.hover_fn = Rc::new(RefCell::new(f));
        self
    }
    fn on_click<F: FnMut(&mut Self, Offset) + 'static>(mut self, f: F) -> Self {
        self.click_fn = Rc::new(RefCell::new(f));
        self
    }
}

impl<W: WidgetBase + Clone> WidgetInternal for Button<W> {
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
    fn get_frame(&self) -> themes::FrameFn {
        self.base.get_frame()
    }
    fn draw_frame(&mut self, _: &Buffer) {}
    fn draw(&mut self, buf: &Buffer) {
        if self.clicked.is_some() {
            self.base.set_background_color(self.clicked_bg);
        } else if self.hovered.is_some() {
            self.base.set_background_color(self.hovered_bg);
        } else {
            self.base.set_background_color(self.default_bg);
        }
        self.base.draw(buf);
    }

    fn handle_button(&mut self, pos: Offset, pressed: bool) {
        let pos = pos - self.get_offset();

        if pos.x < 0
            || pos.y < 0
            || pos.x > self.get_size().w as i32
            || pos.y > self.get_size().h as i32
        {
            return;
        }

        if !pressed {
            (self.click_fn.borrow_mut())(&mut self.clone(), pos)
        };
        self.clicked = pressed.then_some(pos);
        // todo: add button handling!
    }
    fn handle_hover(&mut self, pos: Offset) -> bool {
        let pos = pos - self.get_offset();

        let is_hovered = self.hovered.is_some();
        if pos.x < 0
            || pos.y < 0
            || pos.x > self.get_size().w as i32
            || pos.y > self.get_size().h as i32
        {
            self.clicked = None;
            self.hovered = None;
            return is_hovered;
        }

        (self.hover_fn.borrow_mut())(&mut self.clone(), pos);
        self.hovered = Some(pos);

        !is_hovered
    }
}

pub fn button<W: IntoWidget>(base: W) -> Button<W::W>
where
    W::W: Clone,
{
    Button {
        base: Box::new(base.into()),
        default_bg: Rgba::WHITE,
        hovered_bg: Rgba::hex("#808080").unwrap(),
        clicked_bg: Rgba::hex("#a0a0a0").unwrap(),
        hovered: None,
        clicked: None,
        hover_fn: Rc::new(RefCell::new(|_button: &mut Button<W::W>, _| {})),
        click_fn: Rc::new(RefCell::new(|_button: &mut Button<W::W>, _| {})),
    }
}
