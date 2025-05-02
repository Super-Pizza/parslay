use std::marker::PhantomData;

use lite_graphics::{draw::Rgba, Rect};

use super::{Buffer, Offset, Size, Widget, WidgetBase, WidgetExt, WidgetGroup, WidgetInternal};

pub trait Direction {}

pub struct Horizontal;
pub struct Vertical;

impl Direction for Horizontal {}
impl Direction for Vertical {}

pub struct Stack<D: Direction> {
    base: Widget,
    gap: u32,
    children: Vec<Box<dyn WidgetBase>>,
    _marker: PhantomData<D>,
}

pub type HStack = Stack<Horizontal>;
pub type VStack = Stack<Vertical>;

impl<D: Direction> Stack<D> {
    pub fn gap(mut self, gap: u32) -> Self {
        self.gap = gap;
        self
    }
}

impl<D: Direction> WidgetBase for Stack<D>
where
    Stack<D>: WidgetInternal,
{
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
    // No meaning here
    fn set_text(&mut self, _text: &str) {}
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

impl<D: Direction> WidgetExt for Stack<D>
where
    Stack<D>: WidgetInternal,
{
    fn new() -> Self {
        Self {
            base: Widget::new(),
            gap: 0,
            children: vec![],
            _marker: PhantomData,
        }
    }

    fn on_hover<F: FnMut(&mut Self, Offset) + 'static>(self, _f: F) -> Self {
        self
    }
    fn on_click<F: FnMut(&mut Self, Offset) + 'static>(self, _f: F) -> Self {
        self
    }
}

impl WidgetInternal for HStack {
    fn compute_size(&mut self, font: ab_glyph::FontArc) {
        let mut max_height = 0;
        let mut total_width = 0;
        for child in &mut self.children {
            child.compute_size(font.clone());
            let bounds = child.get_size();
            max_height = max_height.max(bounds.h);
            total_width += bounds.w;
        }
        let padding = self.get_padding();
        let padding_size = Size::from((padding.1 + padding.3, padding.0 + padding.2));
        self.set_size(Size::from((
            total_width + self.gap * (self.children.len() as u32 - 1) + padding_size.w,
            max_height + padding_size.h,
        )));
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&mut self, pos: Offset) {
        self.base.set_offset(pos);
        let padding = self.get_padding();
        let mut offs = Offset::from((padding.3 as i32, padding.0 as i32));
        for child in &mut self.children {
            let bounds = child.get_size();
            child.set_offset(offs);
            offs.x += bounds.w as i32 + self.gap as i32;
        }
    }
    fn draw(&mut self, buf: &Buffer) {
        let offset = self.get_offset();
        buf.fill_round_rect_aa(
            Rect::from((offset, self.get_size())),
            self.get_border_radius(),
            self.get_backgounr_color(),
        );
        let offs_buf = buf.with_offset(offset);
        for child in &mut self.children {
            child.draw(&offs_buf);
        }
    }
    fn handle_click(&mut self, pos: Offset) {
        let pos = pos - self.get_offset();
        let size = self.get_size();
        if pos.x < 0 || pos.y < 0 || pos.x > size.w as i32 || pos.y > size.h as i32 {
            return;
        }
        for child in &mut self.children {
            child.handle_click(pos);
        }
    }
    fn handle_hover(&mut self, pos: Offset) -> bool {
        let pos = pos - self.get_offset();
        let mut redraw = false;
        for child in &mut self.children {
            redraw |= child.handle_hover(pos);
        }
        redraw
    }
}

impl WidgetInternal for VStack {
    fn compute_size(&mut self, font: ab_glyph::FontArc) {
        let mut max_width = 0;
        let mut total_height = 0;
        for child in &mut self.children {
            child.compute_size(font.clone());
            let bounds = child.get_size();
            max_width = max_width.max(bounds.w);
            total_height += bounds.h;
        }
        let padding = self.get_padding();
        let padding_size = Size::from((padding.1 + padding.3, padding.0 + padding.2));
        self.set_size(Size::from((
            max_width + padding_size.w,
            total_height + self.gap * (self.children.len() as u32 - 1) + padding_size.h,
        )));
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&mut self, pos: Offset) {
        self.base.set_offset(pos);
        let padding = self.get_padding();
        let mut offs = Offset::from((padding.3 as i32, padding.0 as i32));
        for child in &mut self.children {
            let bounds = child.get_size();
            child.set_offset(offs);
            offs.y += bounds.h as i32 + self.gap as i32;
        }
    }
    fn draw(&mut self, buf: &Buffer) {
        let offset = self.get_offset();
        buf.fill_round_rect_aa(
            Rect::from((offset, self.get_size())),
            self.get_border_radius(),
            self.get_backgounr_color(),
        );
        let offs_buf = buf.with_offset(offset);
        for child in &mut self.children {
            child.draw(&offs_buf);
        }
    }
    fn handle_click(&mut self, pos: Offset) {
        let pos = pos - self.get_offset();
        let size = self.get_size();
        if pos.x < 0 || pos.y < 0 || pos.x > size.w as i32 || pos.y > size.h as i32 {
            return;
        }
        for child in &mut self.children {
            child.handle_click(pos);
        }
    }
    fn handle_hover(&mut self, pos: Offset) -> bool {
        let pos = pos - self.get_offset();
        let mut redraw = false;
        for child in &mut self.children {
            redraw |= child.handle_hover(pos);
        }
        redraw
    }
}

pub fn hstack<G: WidgetGroup>(gap: u32, widgets: G) -> HStack {
    Stack {
        base: Widget::new(),
        gap,
        children: widgets.create_group(),
        _marker: PhantomData,
    }
}

pub fn vstack<G: WidgetGroup>(gap: u32, widgets: G) -> VStack {
    Stack {
        base: Widget::new(),
        gap,
        children: widgets.create_group(),
        _marker: PhantomData,
    }
}
