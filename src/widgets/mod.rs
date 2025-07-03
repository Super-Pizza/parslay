pub mod button;
pub mod input;
pub mod label;
pub mod stack;
pub mod widget;

pub use widget::Widget;

use std::rc::Rc;

use lite_graphics::{color::Rgba, draw::Buffer, Offset, Size};

use crate::{themes, window::Window};

type MouseEventFn<T> = dyn FnMut(&T, Offset);

pub trait IntoWidget {
    type W: WidgetBase;
    fn into(self) -> Rc<Self::W>;
}

impl IntoWidget for String {
    type W = label::Label;
    fn into(self) -> Rc<Self::W> {
        label::Label::new().text(self)
    }
}

impl IntoWidget for &str {
    type W = label::Label;
    fn into(self) -> Rc<Self::W> {
        label::Label::new().text(self)
    }
}

impl IntoWidget for Box<dyn Fn() -> String> {
    type W = label::Label;
    fn into(self) -> Rc<Self::W> {
        label::dyn_label(self)
    }
}

impl IntoWidget for Box<dyn Fn() -> &'static str> {
    type W = label::Label;
    fn into(self) -> Rc<Self::W> {
        label::dyn_label(self)
    }
}

impl<W: WidgetBase> IntoWidget for Rc<W> {
    type W = W;
    fn into(self) -> Rc<Self::W> {
        self
    }
}

pub trait WidgetBase: WidgetInternal {
    fn set_size(&self, size: Size);
    fn set_pos(&self, pos: Offset);
    fn set_frame(&self, frame: String);
    fn set_background_color(&self, color: Rgba);
    fn set_padding(&self, padding: u32);
    fn set_border_radius(&self, radius: u32);
    fn set_color(&self, color: Rgba);
    fn set_text(&self, text: &str);
    fn get_background_color(&self) -> Rgba;
    fn get_padding(&self) -> (u32, u32, u32, u32);
    fn get_border_radius(&self) -> u32;
}

pub trait WidgetExt: WidgetBase {
    fn new() -> Rc<Self>;
    fn size<S: Into<Size>>(self: Rc<Self>, size: S) -> Rc<Self>
    where
        Self: Sized,
    {
        self.set_size(size.into());
        self
    }
    fn pos<P: Into<Offset>>(self: Rc<Self>, pos: P) -> Rc<Self>
    where
        Self: Sized,
    {
        self.set_pos(pos.into());
        self
    }
    fn frame(self: Rc<Self>, frame: themes::FrameType) -> Rc<Self>
    where
        Self: Sized,
    {
        self.set_frame(frame.to_string());
        self
    }
    fn background_color<C: Into<Rgba>>(self: Rc<Self>, color: C) -> Rc<Self>
    where
        Self: Sized,
    {
        self.set_background_color(color.into());
        self
    }
    fn padding(self: Rc<Self>, padding: u32) -> Rc<Self>
    where
        Self: Sized,
    {
        self.set_padding(padding);
        self
    }
    fn border_radius(self: Rc<Self>, radius: u32) -> Rc<Self>
    where
        Self: Sized,
    {
        self.set_border_radius(radius);
        self
    }
    fn color<C: Into<Rgba>>(self: Rc<Self>, color: C) -> Rc<Self>
    where
        Self: Sized,
    {
        self.set_color(color.into());
        self
    }
    fn text<S: AsRef<str>>(self: Rc<Self>, text: S) -> Rc<Self>
    where
        Self: Sized,
    {
        self.set_text(text.as_ref());
        self
    }

    fn on_hover<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, f: F) -> Rc<Self>
    where
        Self: Sized;
    fn on_click<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, f: F) -> Rc<Self>
    where
        Self: Sized;
}

/// Internal functions
pub trait WidgetInternal {
    fn compute_size(&self, font: ab_glyph::FontArc);
    fn get_size(&self) -> Size;
    fn get_offset(&self) -> Offset;
    fn set_offset(&self, pos: Offset);
    fn get_frame(&self) -> themes::FrameFn;
    fn draw_frame(&self, buf: &Buffer);
    fn draw(&self, buf: &Buffer);
    fn handle_button(self: Rc<Self>, pos: Offset, pressed: Option<Rc<Window>>);
    /// Return: If Should Redraw
    fn handle_hover(self: Rc<Self>, pos: Offset) -> bool;
}

pub trait WidgetGroup {
    fn create_group(self) -> Vec<Rc<dyn WidgetBase>>;
}

impl<W: IntoWidget + 'static> WidgetGroup for W {
    fn create_group(self) -> Vec<Rc<dyn WidgetBase>> {
        vec![self.into()]
    }
}

impl<W: IntoWidget + 'static, const N: usize> WidgetGroup for [W; N] {
    fn create_group(self) -> Vec<Rc<dyn WidgetBase>> {
        self.into_iter()
            .map::<Rc<dyn WidgetBase>, _>(|w| w.into())
            .collect()
    }
}

macro_rules! tupled_group {
    ($($id:tt $name:ident),+) => {
        impl<$($name: IntoWidget + 'static),+> WidgetGroup for ($($name),+)
        {
            fn create_group(self) -> Vec<Rc<dyn WidgetBase>> {
                vec![
                    $(self.$id.into()),+
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
