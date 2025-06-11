pub mod button;
pub mod label;
pub mod stack;
pub mod widget;

pub use widget::Widget;

use lite_graphics::{
    draw::{Buffer, Rgba},
    Offset, Size,
};

type MouseEventFn<T> = dyn FnMut(&mut T, Offset);

pub trait IntoWidget {
    type W: WidgetBase;
    fn into(self) -> Self::W;
}

impl IntoWidget for String {
    type W = label::Label;
    fn into(self) -> Self::W {
        label::Label::new().text(self)
    }
}

impl IntoWidget for &str {
    type W = label::Label;
    fn into(self) -> Self::W {
        label::Label::new().text(self)
    }
}

impl IntoWidget for Box<dyn Fn() -> String> {
    type W = label::Label;
    fn into(self) -> Self::W {
        label::dyn_label(self)
    }
}

impl IntoWidget for Box<dyn Fn() -> &'static str> {
    type W = label::Label;
    fn into(self) -> Self::W {
        label::dyn_label(self)
    }
}

impl<W: WidgetBase> IntoWidget for W {
    type W = W;
    fn into(self) -> Self::W {
        self
    }
}

pub trait WidgetBase: WidgetInternal {
    fn set_size(&mut self, size: Size);
    fn set_pos(&mut self, pos: Offset);
    fn set_background_color(&mut self, color: Rgba);
    fn set_padding(&mut self, padding: u32);
    fn set_border_radius(&mut self, radius: u32);
    fn set_color(&mut self, color: Rgba);
    fn set_text(&mut self, text: &str);
    fn get_backgounr_color(&self) -> Rgba;
    fn get_padding(&self) -> (u32, u32, u32, u32);
    fn get_border_radius(&self) -> u32;
}

pub trait WidgetExt: WidgetBase {
    fn new() -> Self;
    fn size<S: Into<Size>>(mut self, size: S) -> Self
    where
        Self: Sized,
    {
        self.set_size(size.into());
        self
    }
    fn pos<P: Into<Offset>>(mut self, pos: P) -> Self
    where
        Self: Sized,
    {
        self.set_pos(pos.into());
        self
    }
    fn background_color<C: Into<Rgba>>(mut self, color: C) -> Self
    where
        Self: Sized,
    {
        self.set_background_color(color.into());
        self
    }
    fn padding(mut self, padding: u32) -> Self
    where
        Self: Sized,
    {
        self.set_padding(padding);
        self
    }
    fn border_radius(mut self, radius: u32) -> Self
    where
        Self: Sized,
    {
        self.set_border_radius(radius);
        self
    }
    fn color<C: Into<Rgba>>(mut self, color: C) -> Self
    where
        Self: Sized,
    {
        self.set_color(color.into());
        self
    }
    fn text<S: AsRef<str>>(mut self, text: S) -> Self
    where
        Self: Sized,
    {
        self.set_text(text.as_ref());
        self
    }

    fn on_hover<F: FnMut(&mut Self, Offset) + 'static>(self, f: F) -> Self
    where
        Self: Sized;
    fn on_click<F: FnMut(&mut Self, Offset) + 'static>(self, f: F) -> Self
    where
        Self: Sized;
}

/// Internal functions
pub trait WidgetInternal {
    fn compute_size(&mut self, font: ab_glyph::FontArc);
    fn get_size(&self) -> Size;
    fn get_offset(&self) -> Offset;
    fn set_offset(&mut self, pos: Offset);
    fn draw(&mut self, buf: &Buffer);
    fn handle_click(&mut self, pos: Offset);
    fn handle_hover(&mut self, pos: Offset) -> bool;
}

pub trait WidgetGroup {
    fn create_group(self) -> Vec<Box<dyn WidgetBase>>;
}

impl<W: IntoWidget + 'static> WidgetGroup for W {
    fn create_group(self) -> Vec<Box<dyn WidgetBase>> {
        vec![Box::new(self.into())]
    }
}

impl<W: IntoWidget + 'static, const N: usize> WidgetGroup for [W; N] {
    fn create_group(self) -> Vec<Box<dyn WidgetBase>> {
        self.into_iter()
            .map::<Box<dyn WidgetBase>, _>(|w| Box::new(w.into()))
            .collect()
    }
}

macro_rules! tupled_group {
    ($($id:tt $name:ident),+) => {
        impl<$($name: IntoWidget + 'static),+> WidgetGroup for ($($name),+)
        {
            fn create_group(self) -> Vec<Box<dyn WidgetBase>> {
                vec![
                    $(Box::new(self.$id.into())),+
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
