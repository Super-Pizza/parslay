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
}

impl<D: Direction> WidgetBase for Stack<D>
where
    Stack<D>: WidgetInternal,
{
    fn set_size(&mut self, size: Size) {
        self.base.size = size
    }
    fn set_pos(&mut self, pos: Offset) {
        self.base.pos = pos;
    }
    fn set_background_color(&mut self, color: Rgba) {
        self.base.bg_color = color;
    }
    fn set_padding(&mut self, padding: u32) {
        self.base.padding = [padding; 4].into();
    }
    fn set_border_radius(&mut self, radius: u32) {
        self.base.border_radius = radius;
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
        let padding = Size::from((
            self.base.padding.1 + self.base.padding.3,
            self.base.padding.0 + self.base.padding.2,
        ));
        self.base.size = Size::from((
            total_width + self.gap * (self.children.len() as u32 - 1) + padding.w,
            max_height + padding.h,
        ));
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&mut self, pos: Offset) {
        self.base.set_offset(pos);
        let mut offs = Offset::from((self.base.padding.3 as i32, self.base.padding.0 as i32));
        for child in &mut self.children {
            let bounds = child.get_size();
            child.set_offset(offs);
            offs.x += bounds.w as i32 + self.gap as i32;
        }
    }
    fn draw(&mut self, buf: &Buffer) {
        let offset = self.base.pos;
        let offs_buf = buf.with_offset(offset);
        offs_buf.fill_rect(
            Rect::from((self.base.pos, self.base.size)),
            self.base.bg_color,
        );
        for child in &mut self.children {
            child.draw(&offs_buf);
        }
    }
    fn handle_click(&mut self, pos: Offset) {
        let pos = pos - self.base.pos;
        if pos.x < 0
            || pos.y < 0
            || pos.x > self.base.size.w as i32
            || pos.y > self.base.size.h as i32
        {
            return;
        }
        for child in &mut self.children {
            child.handle_click(pos);
        }
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
        let padding = Size::from((
            self.base.padding.1 + self.base.padding.3,
            self.base.padding.0 + self.base.padding.2,
        ));
        self.base.size = Size::from((
            max_width + padding.w,
            total_height + self.gap * (self.children.len() as u32 - 1) + padding.h,
        ));
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&mut self, pos: Offset) {
        self.base.set_offset(pos);
        let mut offs = Offset::from((self.base.padding.3 as i32, self.base.padding.0 as i32));
        for child in &mut self.children {
            let bounds = child.get_size();
            child.set_offset(offs);
            offs.y += bounds.h as i32 + self.gap as i32;
        }
    }
    fn draw(&mut self, buf: &Buffer) {
        let offset = self.base.pos;
        let offs_buf = buf.with_offset(offset);
        offs_buf.fill_round_rect_aa(
            Rect::from((self.base.pos, self.base.size)),
            self.base.border_radius as i32,
            self.base.bg_color,
        );
        for child in &mut self.children {
            child.draw(&offs_buf);
        }
    }
    fn handle_click(&mut self, pos: Offset) {
        let pos = pos - self.base.pos;
        if pos.x < 0
            || pos.y < 0
            || pos.x > self.base.size.w as i32
            || pos.y > self.base.size.h as i32
        {
            return;
        }
        for child in &mut self.children {
            child.handle_click(pos);
        }
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
