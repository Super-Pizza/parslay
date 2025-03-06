pub mod label;
pub mod stack;

use lite_graphics::{
    draw::{Buffer, Rgba}, Offset, Rect, Size
};

use crate::{window::Window, IntoView};

pub trait WidgetBase {
    fn size<S: Into<Size>>(self, size: S) -> Self;
    fn pos<P: Into<Offset>>(self, pos: P) -> Self;
    fn font_size<S: Into<f32>>(self, size: S) -> Self;
    fn label<S: AsRef<str>>(self, label: S) -> Self;
    fn background_color<C: Into<Rgba>>(self, color: C) -> Self;
}

pub trait WidgetExt {
    fn compute_size(&mut self);
    fn get_size(&self) -> Size;
    fn set_pos(&mut self, pos: Offset);
    fn draw(&self, buf: &Buffer);
}

pub struct WidgetView {
    label: String,
    size: Size,
    pos: Offset,
    font_size: f32,
    background_color: Rgba,
}

impl WidgetView {
    pub(crate) fn new() -> Self {
        Self {
            size: Default::default(),
            pos: Default::default(),
            font_size: 12.0,
            label: String::new(),
            background_color: Rgba::WHITE,
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
    fn background_color<C: Into<Rgba>>(mut self, color: C) -> Self {
        self.background_color = color.into();
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
            background_color: self.background_color,
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
            background_color: Rgba::WHITE,
        }
    }
}

pub struct Widget {
    window: Window,
    label: String,
    size: Size,
    pos: Offset,
    font_size: f32,
    background_color: Rgba,
}

impl WidgetExt for Widget {
    fn compute_size(&mut self) {}
    fn get_size(&self) -> Size {
        self.size
    }
    fn set_pos(&mut self, pos: Offset) {
        self.pos = pos;
    }
    fn draw(&self, buf: &Buffer) {
        buf.fill_rect(Rect::from((self.pos, self.size)), self.background_color);
    }
}

pub trait WidgetGroup {
    fn create_group(self, window: Window) -> Vec<Box<dyn WidgetExt>>;
}

impl<W: WidgetBase + IntoView> WidgetGroup for W
where
    <W as IntoView>::Widget: 'static,
{
    fn create_group(self, window: Window) -> Vec<Box<dyn WidgetExt>> {
        vec![Box::new(self.create(window))]
    }
}

impl<W: WidgetBase + IntoView, const N: usize> WidgetGroup for [W; N]
where
    <W as IntoView>::Widget: 'static,
{
    fn create_group(self, window: Window) -> Vec<Box<dyn WidgetExt>> {
        self.into_iter()
            .map(|w| Box::new(w.create(window.clone())) as Box<dyn WidgetExt>)
            .collect()
    }
}

macro_rules! tupled_group {
    ($($id:tt $name:ident),+) => {
        impl<$($name: WidgetBase + IntoView),+> WidgetGroup for ($($name),+)
        where
            $(<$name as IntoView>::Widget: 'static),+
        {
            fn create_group(self, window: Window) -> Vec<Box<dyn WidgetExt>> {
                vec![
                    $(Box::new(self.$id.create(window.clone()))),+
                ]
            }
        }
    };
}

tupled_group!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9, 9 T10, 10 T11, 11 T12);
tupled_group!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9, 9 T10, 10 T11);
tupled_group!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9, 9 T10);
tupled_group!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8, 8 T9);
tupled_group!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7, 7 T8);
tupled_group!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6, 6 T7);
tupled_group!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5, 5 T6);
tupled_group!(0 T1, 1 T2, 2 T3, 3 T4, 4 T5);
tupled_group!(0 T1, 1 T2, 2 T3, 3 T4);
tupled_group!(0 T1, 1 T2, 2 T3);
tupled_group!(0 T1, 1 T2);
