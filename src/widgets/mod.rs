pub mod label;

use lite_graphics::{draw::Buffer, Offset, Size};

use crate::{window::Window, IntoView};

pub trait WidgetBase {
    fn size<S: Into<Size>>(self, size: S) -> Self;
    fn pos<P: Into<Offset>>(self, pos: P) -> Self;
    fn font_size<S: Into<f32>>(self, size: S) -> Self;
    fn label<S: AsRef<str>>(self, label: S) -> Self;
}

pub trait WidgetExt: WidgetBase {
    fn draw(&self, buf: &Buffer);
}

pub struct WidgetView {
    label: String,
    size: Size,
    pos: Offset,
    font_size: f32,
}

impl WidgetView {
    pub(crate) fn new() -> Self {
        Self {
            size: Default::default(),
            pos: Default::default(),
            font_size: 12.0,
            label: String::new(),
        }
    }
}

impl WidgetBase for WidgetView {
    fn label<S: AsRef<str>>(mut self, label: S) -> Self {
        self.label = label.as_ref().to_owned();
        self
    }
    fn size<S: Into<Size>>(mut self, size: S) -> Self {
        self.size = size.into();
        self
    }
    fn pos<P: Into<Offset>>(mut self, pos: P) -> Self {
        self.pos = pos.into();
        self
    }
    fn font_size<S: Into<f32>>(mut self, size: S) -> Self {
        self.font_size = size.into();
        self
    }
}

impl IntoView for WidgetView {
    type Widget = Widget;

    fn create(self, window: Window) -> Self::Widget
    where
        Self::Widget: WidgetExt,
    {
        Widget {
            window,
            label: self.label,
            size: self.size,
            pos: self.pos,
            font_size: self.font_size,
        }
    }
}

impl IntoView for () {
    type Widget = Widget;

    fn create(self, window: Window) -> Self::Widget
    where
        Self::Widget: WidgetExt,
    {
        Widget {
            window,
            label: String::new(),
            size: Default::default(),
            pos: Default::default(),
            font_size: 12.0,
        }
    }
}

pub struct Widget {
    window: Window,
    label: String,
    size: Size,
    pos: Offset,
    font_size: f32,
}

impl WidgetExt for Widget {
    fn draw(&self, _: &Buffer) {}
}

impl WidgetBase for Widget {
    fn label<S: AsRef<str>>(mut self, label: S) -> Self {
        self.label = label.as_ref().to_owned();
        self
    }
    fn size<S: Into<Size>>(mut self, size: S) -> Self {
        self.size = size.into();
        self
    }
    fn pos<P: Into<Offset>>(mut self, pos: P) -> Self {
        self.pos = pos.into();
        self
    }
    fn font_size<S: Into<f32>>(mut self, size: S) -> Self {
        self.font_size = size.into();
        self
    }
}
