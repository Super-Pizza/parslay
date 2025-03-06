use std::marker::PhantomData;

use lite_graphics::{draw::Rgba, Rect};

use crate::IntoView;

use super::{Buffer, Offset, Size, Widget, WidgetBase, WidgetExt, WidgetGroup, WidgetView};

pub struct StackView<D: Direction, G: WidgetGroup> {
    base: WidgetView,
    gap: u32,
    children: G,
    _marker: PhantomData<D>,
}

pub trait Direction {}

pub struct Horizontal;
pub struct Vertical;

impl Direction for Horizontal {}
impl Direction for Vertical {}

pub struct Stack<D: Direction> {
    base: Widget,
    gap: u32,
    children: Vec<Box<dyn WidgetExt>>,
    _marker: PhantomData<D>,
}

pub type HStack = Stack<Horizontal>;
pub type VStack = Stack<Vertical>;

impl<D: Direction, G: WidgetGroup> IntoView for StackView<D, G>
where
    Stack<D>: WidgetExt,
{
    type Widget = Stack<D>;
    fn create(self, window: crate::window::Window) -> Self::Widget
    where
        Self::Widget: super::WidgetExt,
    {
        let children = self.children.create_group(window.clone());
        Stack {
            base: self.base.create(window),
            gap: self.gap,
            children,
            _marker: PhantomData,
        }
    }
}

impl<D: Direction, G: WidgetGroup> StackView<D, G> {
    pub fn gap(mut self, gap: u32) -> Self {
        self.gap = gap;
        self
    }
}

impl<D: Direction, G: WidgetGroup> WidgetBase for StackView<D, G> {
    fn label<S: AsRef<str>>(mut self, label: S) -> Self {
        self.base.label = label.as_ref().to_owned();
        self
    }
    fn size<S: Into<Size>>(mut self, size: S) -> Self {
        self.base.size = size.into();
        self
    }
    fn pos<P: Into<Offset>>(mut self, pos: P) -> Self {
        self.base.pos = pos.into();
        self
    }
    fn font_size<S: Into<f32>>(mut self, size: S) -> Self {
        self.base.font_size = size.into();
        self
    }
    fn background_color<C: Into<Rgba>>(mut self, color: C) -> Self {
        self.base.background_color = color.into();
        self
    }
}

impl WidgetExt for HStack {
    fn compute_size(&mut self) {
        let mut max_height = 0;
        let mut total_width = 0;
        for child in &mut self.children {
            child.compute_size();
            let bounds = child.get_size();
            max_height = max_height.max(bounds.h);
            total_width += bounds.w;
        }
        self.base.size = Size::from((
            total_width + self.gap * (self.children.len() as u32 + 1),
            max_height + self.gap * 2,
        ));
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn set_pos(&mut self, pos: Offset) {
        self.base.set_pos(pos);
        let mut offs = Offset::from((self.gap as _, self.gap as _));
        for child in &mut self.children {
            let bounds = child.get_size();
            child.set_pos(offs);
            offs.x += bounds.w as i32 + self.gap as i32;
        }
    }
    fn draw(&self, buf: &Buffer) {
        let offset = self.base.pos;
        let offs_buf = buf.with_offset(offset);
        offs_buf.fill_rect(
            Rect::from((self.base.pos, self.base.size)),
            self.base.background_color,
        );
        for child in &self.children {
            child.draw(&offs_buf);
        }
    }
}

impl WidgetExt for VStack {
    fn compute_size(&mut self) {
        let mut max_width = 0;
        let mut total_height = 0;
        for child in &mut self.children {
            child.compute_size();
            let bounds = child.get_size();
            max_width = max_width.max(bounds.w);
            total_height += bounds.h;
        }
        self.base.size = Size::from((
            max_width + self.gap * 2,
            total_height + self.gap * (self.children.len() as u32 + 1),
        ));
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn set_pos(&mut self, pos: Offset) {
        self.base.set_pos(pos);
        let mut offs = Offset::from((self.gap as _, self.gap as _));
        for child in &mut self.children {
            let bounds = child.get_size();
            child.set_pos(offs);
            offs.y += bounds.h as i32 + self.gap as i32;
        }
    }
    fn draw(&self, buf: &Buffer) {
        let offset = self.base.pos;
        let offs_buf = buf.with_offset(offset);
        offs_buf.fill_rect(
            Rect::from((self.base.pos, self.base.size)),
            self.base.background_color,
        );
        for child in &self.children {
            child.draw(&offs_buf);
        }
    }
}

pub fn hstack<G: WidgetGroup>(gap: u32, widgets: G) -> StackView<Horizontal, G> {
    StackView {
        base: WidgetView::new(),
        gap,
        children: widgets,
        _marker: PhantomData,
    }
}

pub fn vstack<G: WidgetGroup>(gap: u32, widgets: G) -> StackView<Vertical, G> {
    StackView {
        base: WidgetView::new(),
        gap,
        children: widgets,
        _marker: PhantomData,
    }
}
