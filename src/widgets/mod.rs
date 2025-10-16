pub mod button;
pub mod drop_down;
pub mod input;
pub mod label;
pub mod stack;
pub mod widget;

pub use widget::Widget;

use std::{any::Any, rc::Rc};

use lite_graphics::{Buffer, Drawable, Offset, Size, color::Rgba};

use crate::{
    app::{CursorType, HoverResult},
    themes,
    window::Window,
};

type MouseEventFn<T> = dyn FnMut(&T, Offset);
type InputEventFn<T> = dyn FnMut(&T);

pub trait IntoWidget {
    type W: WidgetBase;
    fn into_widget(self) -> Rc<Self::W>;
}

impl IntoWidget for String {
    type W = label::Label;
    fn into_widget(self) -> Rc<Self::W> {
        label::Label::new().text(self)
    }
}

impl IntoWidget for &str {
    type W = label::Label;
    fn into_widget(self) -> Rc<Self::W> {
        label::Label::new().text(self)
    }
}

impl IntoWidget for Box<dyn Fn() -> String> {
    type W = label::Label;
    fn into_widget(self) -> Rc<Self::W> {
        label::dyn_label(self)
    }
}

impl IntoWidget for Box<dyn Fn() -> &'static str> {
    type W = label::Label;
    fn into_widget(self) -> Rc<Self::W> {
        label::dyn_label(self)
    }
}

impl<W: WidgetBase> IntoWidget for Rc<W> {
    type W = W;
    fn into_widget(self) -> Rc<Self::W> {
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
    fn get_text(&self) -> String;
    fn set_disabled(&self, disable: bool);
    fn is_disabled(&self) -> bool;
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
pub trait WidgetInternal: Any {
    fn compute_size(&self, font: ab_glyph::FontArc);
    fn get_size(&self) -> Size;
    fn get_offset(&self) -> Offset;
    fn set_offset(&self, pos: Offset);
    fn get_frame(&self) -> themes::FrameFn;
    fn draw_frame(&self, buf: &dyn Drawable);
    fn draw(&self, buf: &mut dyn Drawable);
    fn draw_overlays(&self, _buf: &mut Buffer) {}

    fn handle_button(self: Rc<Self>, pos: Offset, pressed: Option<Rc<Window>>);
    /// Return: If Should Redraw
    fn handle_hover(self: Rc<Self>, pos: Offset) -> HoverResult;
    fn handle_overlay_button(self: Rc<Self>, _pos: Offset, _pressed: Option<Rc<Window>>) -> bool {
        false
    }
    fn handle_overlay_hover(self: Rc<Self>, _pos: Offset) -> HoverResult {
        HoverResult {
            redraw: false,
            cursor: CursorType::Arrow,
        }
    }
}

pub trait WidgetGroup {
    fn create_group(self) -> Vec<Rc<dyn WidgetBase>>;
    fn map<F: Fn(Rc<dyn WidgetBase>) -> Rc<dyn WidgetBase>>(self, f: F) -> Vec<Rc<dyn WidgetBase>>;
}

impl<W: IntoWidget + 'static> WidgetGroup for W {
    fn create_group(self) -> Vec<Rc<dyn WidgetBase>> {
        vec![self.into_widget()]
    }
    fn map<F: Fn(Rc<dyn WidgetBase>) -> Rc<dyn WidgetBase>>(self, f: F) -> Vec<Rc<dyn WidgetBase>> {
        vec![f(self.into_widget())]
    }
}

impl<W: IntoWidget + 'static, const N: usize> WidgetGroup for [W; N] {
    fn create_group(self) -> Vec<Rc<dyn WidgetBase>> {
        self.into_iter()
            .map::<Rc<dyn WidgetBase>, _>(|w| w.into_widget())
            .collect()
    }
    fn map<F: Fn(Rc<dyn WidgetBase>) -> Rc<dyn WidgetBase>>(self, f: F) -> Vec<Rc<dyn WidgetBase>> {
        self.into_iter()
            .map::<Rc<dyn WidgetBase>, _>(|w| f(w.into_widget()))
            .collect()
    }
}

impl<W: IntoWidget + 'static> WidgetGroup for Vec<W> {
    fn create_group(self) -> Vec<Rc<dyn WidgetBase>> {
        self.into_iter()
            .map::<Rc<dyn WidgetBase>, _>(|w| w.into_widget())
            .collect()
    }
    fn map<F: Fn(Rc<dyn WidgetBase>) -> Rc<dyn WidgetBase>>(self, f: F) -> Vec<Rc<dyn WidgetBase>> {
        self.into_iter()
            .map::<Rc<dyn WidgetBase>, _>(|w| f(w.into_widget()))
            .collect()
    }
}

impl<W: IntoWidget + 'static, G: WidgetGroup + 'static> WidgetGroup for (W, G) {
    fn create_group(self) -> Vec<Rc<dyn WidgetBase>> {
        let mut result: Vec<Rc<dyn WidgetBase>> = vec![self.0.into_widget()];
        result.append(&mut self.1.create_group());
        result
    }
    fn map<F: Fn(Rc<dyn WidgetBase>) -> Rc<dyn WidgetBase>>(self, f: F) -> Vec<Rc<dyn WidgetBase>> {
        let mut result: Vec<Rc<dyn WidgetBase>> = vec![f(self.0.into_widget())];
        result.append(&mut self.1.map(f));
        result
    }
}

macro_rules! tupled_group {
    ($($id:tt $name:ident),+) => {
        impl<$($name: IntoWidget + 'static),+> WidgetGroup for ($($name),+)
        {
            fn create_group(self) -> Vec<Rc<dyn WidgetBase>> {
                vec![
                    $(self.$id.into_widget()),+
                ]
            }
            fn map<F: Fn(Rc<dyn WidgetBase>) -> Rc<dyn WidgetBase>>(self, f: F) -> Vec<Rc<dyn WidgetBase>> {
                vec![
                    $(f(self.$id.into_widget())),+
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
