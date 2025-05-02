use std::{cell::RefCell, rc::Rc};

use lite_graphics::draw::Rgba;

use super::{Buffer, Offset, Size, WidgetBase, WidgetExt, WidgetInternal};
use super::{IntoWidget, MouseEventFn};

#[derive(Clone)]
pub struct Button<W: WidgetBase> {
    base: Box<W>,

    hovered: Option<(Box<W>, Offset)>,
    clicked: Option<(Box<W>, Offset)>,
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
    fn set_background_color(&mut self, color: Rgba) {
        self.base.set_background_color(color);
    }
    fn set_padding(&mut self, padding: u32) {
        self.base.set_padding(padding);
    }
    fn set_border_radius(&mut self, radius: u32) {
        self.base.set_border_radius(radius);
    }
    fn set_text(&mut self, text: &str) {
        self.base.set_text(text);
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

impl<W: WidgetExt + Clone> WidgetExt for Button<W> {
    fn new() -> Self {
        Self {
            base: Box::new(W::new()),
            hover_fn: Rc::new(RefCell::new(|button: &mut Button<W>, _| {
                button.set_background_color(Rgba::hex("#808080").unwrap())
            })),
            click_fn: Rc::new(RefCell::new(|button: &mut Button<W>, _| {
                button.set_background_color(Rgba::hex("#a0a0a0").unwrap())
            })),
            hovered: None,
            clicked: None,
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
        if let Some((base, _)) = self.clicked.as_mut() {
            base.compute_size(font);
        } else if let Some((base, _)) = self.hovered.as_mut() {
            base.compute_size(font);
        } else {
            self.base.compute_size(font);
        }
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
        if let Some((base, _)) = self.clicked.as_mut() {
            base.draw(buf);
        } else if let Some((base, _)) = self.hovered.as_mut() {
            base.draw(buf);
        } else {
            self.base.draw(buf);
        }
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
        let mut clicked_state = self.clone();
        (self.click_fn.borrow_mut())(&mut clicked_state, pos);
        self.clicked = Some((clicked_state.base, pos));
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
            self.hovered = None;
            return is_hovered;
        }

        let mut hovered_state = self.clone();
        (self.hover_fn.borrow_mut())(&mut hovered_state, pos);
        self.hovered = Some((hovered_state.base, pos));

        !is_hovered
    }
}

pub fn button<W: IntoWidget>(base: W) -> Button<W::W>
where
    W::W: Clone,
{
    Button {
        base: Box::new(base.into()),
        hover_fn: Rc::new(RefCell::new(|button: &mut Button<W::W>, _| {
            button.set_background_color(Rgba::hex("#808080").unwrap())
        })),
        click_fn: Rc::new(RefCell::new(|button: &mut Button<W::W>, _| {
            button.set_background_color(Rgba::hex("#a0a0a0").unwrap())
        })),
        hovered: None,
        clicked: None,
    }
}
