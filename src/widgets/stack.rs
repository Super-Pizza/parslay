use std::marker::PhantomData;

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
}

impl WidgetExt for HStack {
    fn bounds(&self) -> Size {
        let mut max_height = 0;
        let mut widths = vec![];
        for child in &self.children {
            let bounds = child.bounds();
            max_height = max_height.max(bounds.h);
            widths.push(bounds.w);
        }
        Size::from((
            widths.iter().sum::<u32>()
                + self.gap * (widths.len() as u32 - 1)
                + self.base.pos.x as u32,
            max_height + self.base.pos.y as u32,
        ))
    }
    fn draw(&self, buf: &Buffer) {
        let mut max_height = 0;
        let mut widths = vec![];
        for child in &self.children {
            let bounds = child.bounds();
            max_height = max_height.max(bounds.h);
            widths.push(bounds.w);
        }
        let mut offset = self.base.pos;
        for (idx, child) in self.children.iter().enumerate() {
            let offs_buf = buf.with_offset(offset);
            child.draw(&offs_buf);
            offset = offset + Offset::from((widths[idx - 1] as i32 + self.gap as i32, 0))
        }
    }
}
impl WidgetExt for VStack {
    fn bounds(&self) -> Size {
        let mut max_width = 0;
        let mut heights = vec![];
        for child in &self.children {
            let bounds = child.bounds();
            max_width = max_width.max(bounds.w);
            heights.push(bounds.h);
        }
        Size::from((
            max_width + self.base.pos.y as u32,
            heights.iter().sum::<u32>()
                + self.gap * (heights.len() as u32 - 1)
                + self.base.pos.x as u32,
        ))
    }
    fn draw(&self, buf: &Buffer) {
        let mut max_width = 0;
        let mut heights = vec![];
        for child in &self.children {
            let bounds = child.bounds();
            max_width = max_width.max(bounds.w);
            heights.push(bounds.h);
        }
        let mut offset = self.base.pos;
        for (idx, child) in self.children.iter().enumerate() {
            let offs_buf = buf.with_offset(offset);
            child.draw(&offs_buf);
            offset = offset + Offset::from((0, heights[idx] as i32 + self.gap as i32))
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
