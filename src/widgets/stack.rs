use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::Rc,
};

use lite_graphics::color::Rgba;

use crate::{
    app::{CursorType, HoverResult},
    window::Window,
};

use super::{Buffer, Offset, Size, Widget, WidgetBase, WidgetExt, WidgetGroup, WidgetInternal};

pub trait Direction {}

pub struct Horizontal;
pub struct Vertical;

impl Direction for Horizontal {}
impl Direction for Vertical {}

pub struct Stack<D: Direction> {
    base: Widget,
    gap: Cell<u32>,
    children: RefCell<Vec<Rc<dyn WidgetBase>>>,
    _marker: PhantomData<D>,
}

pub type HStack = Stack<Horizontal>;
pub type VStack = Stack<Vertical>;

impl<D: Direction> Stack<D>
where
    Stack<D>: WidgetInternal,
{
    pub fn gap(self, gap: u32) -> Self {
        self.gap.set(gap);
        self
    }
    fn draw_frame(&self, buf: &Buffer) {
        let frame = self.get_frame();
        frame(buf, self.get_size(), self.get_background_color())
    }
    fn draw(&self, buf: &Buffer) {
        let bounds = (self.get_offset(), self.get_size()).into();
        let offs_buf = buf.subregion(bounds);
        self.draw_frame(&offs_buf);
        for child in &*self.children.borrow() {
            child.draw(&offs_buf);
        }
    }
    fn handle_button(self: Rc<Self>, pos: Offset, pressed: Option<Rc<Window>>) {
        let pos = pos - self.get_offset();
        let size = self.get_size();
        if pos.x < 0 || pos.y < 0 || pos.x > size.w as i32 || pos.y > size.h as i32 {
            return;
        }
        for child in &*self.children.borrow() {
            child.clone().handle_button(pos, pressed.clone());
        }
    }
    fn handle_hover(self: Rc<Self>, pos: Offset) -> HoverResult {
        let pos = pos - self.get_offset();
        let mut result = HoverResult {
            redraw: false,
            cursor: CursorType::Arrow,
        };
        for child in &*self.children.borrow() {
            result |= child.clone().handle_hover(pos);
        }
        result
    }
}

impl<D: Direction> WidgetBase for Stack<D>
where
    Stack<D>: WidgetInternal,
{
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
    }
    fn set_padding(&self, padding: u32) {
        self.base.set_padding(padding);
    }
    fn set_border_radius(&self, radius: u32) {
        self.base.set_border_radius(radius);
    }
    // No meaning here
    fn set_color(&self, _color: Rgba) {}
    // No meaning here
    fn set_text(&self, _text: &str) {}
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

impl<D: Direction> WidgetExt for Stack<D>
where
    Stack<D>: WidgetInternal,
{
    fn new() -> Rc<Self> {
        let this = Self {
            base: Widget::new_internal(),
            gap: Cell::new(0),
            children: RefCell::new(vec![]),
            _marker: PhantomData,
        };
        Rc::new(this)
    }

    fn on_hover<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, _f: F) -> Rc<Self> {
        self
    }
    fn on_click<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, _f: F) -> Rc<Self> {
        self
    }
}

impl WidgetInternal for HStack {
    fn compute_size(&self, font: ab_glyph::FontArc) {
        let mut max_height = 0;
        let mut total_width = 0;
        for child in &*self.children.borrow() {
            child.compute_size(font.clone());
            let bounds = child.get_size();
            max_height = max_height.max(bounds.h);
            total_width += bounds.w;
        }
        let padding = self.get_padding();
        let padding_size = Size::from((padding.1 + padding.3, padding.0 + padding.2));
        self.set_size(Size::from((
            total_width
                + self.gap.get() * (self.children.borrow().len() as u32 - 1)
                + padding_size.w,
            max_height + padding_size.h,
        )));
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&self, pos: Offset) {
        self.base.set_offset(pos);
        let padding = self.get_padding();
        let mut offs = Offset::from((padding.3 as i32, padding.0 as i32));
        for child in &*self.children.borrow() {
            let bounds = child.get_size();
            child.set_offset(offs);
            offs.x += bounds.w as i32 + self.gap.get() as i32;
        }
    }
    fn get_frame(&self) -> crate::themes::FrameFn {
        self.base.get_frame()
    }
    fn draw_frame(&self, buf: &Buffer) {
        Stack::draw_frame(self, buf);
    }
    fn draw(&self, buf: &Buffer) {
        Stack::draw(self, buf);
    }
    fn handle_button(self: Rc<Self>, pos: Offset, pressed: Option<Rc<Window>>) {
        Stack::handle_button(self, pos, pressed);
    }
    fn handle_hover(self: Rc<Self>, pos: Offset) -> HoverResult {
        Stack::handle_hover(self, pos)
    }
}

impl WidgetInternal for VStack {
    fn compute_size(&self, font: ab_glyph::FontArc) {
        let mut max_width = 0;
        let mut total_height = 0;
        for child in &*self.children.borrow() {
            child.compute_size(font.clone());
            let bounds = child.get_size();
            max_width = max_width.max(bounds.w);
            total_height += bounds.h;
        }
        let padding = self.get_padding();
        let padding_size = Size::from((padding.1 + padding.3, padding.0 + padding.2));
        self.set_size(Size::from((
            max_width + padding_size.w,
            total_height
                + self.gap.get() * (self.children.borrow().len() as u32 - 1)
                + padding_size.h,
        )));
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&self, pos: Offset) {
        self.base.set_offset(pos);
        let padding = self.get_padding();
        let mut offs = Offset::from((padding.3 as i32, padding.0 as i32));
        for child in &*self.children.borrow() {
            let bounds = child.get_size();
            child.set_offset(offs);
            offs.y += bounds.h as i32 + self.gap.get() as i32;
        }
    }
    fn get_frame(&self) -> crate::themes::FrameFn {
        self.base.get_frame()
    }
    fn draw_frame(&self, buf: &Buffer) {
        Stack::draw_frame(self, buf);
    }
    fn draw(&self, buf: &Buffer) {
        Stack::draw(self, buf);
    }
    fn handle_button(self: Rc<Self>, pos: Offset, pressed: Option<Rc<Window>>) {
        Stack::handle_button(self, pos, pressed);
    }
    fn handle_hover(self: Rc<Self>, pos: Offset) -> HoverResult {
        Stack::handle_hover(self, pos)
    }
}

pub fn hstack<G: WidgetGroup>(gap: u32, widgets: G) -> Rc<HStack> {
    let this = Stack {
        base: Widget::new_internal(),
        gap: Cell::new(gap),
        children: RefCell::new(widgets.create_group()),
        _marker: PhantomData,
    };
    Rc::new(this)
}

pub fn vstack<G: WidgetGroup>(gap: u32, widgets: G) -> Rc<VStack> {
    let this = Stack {
        base: Widget::new_internal(),
        gap: Cell::new(gap),
        children: RefCell::new(widgets.create_group()),
        _marker: PhantomData,
    };
    Rc::new(this)
}
