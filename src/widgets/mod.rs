pub mod button;
pub mod label;
pub mod stack;

use lite_graphics::{
    draw::{Buffer, Rgba},
    Offset, Rect, Size,
};

pub trait WidgetBase: WidgetInternal {
    fn set_size(&mut self, size: Size);
    fn set_pos(&mut self, pos: Offset);
    fn set_font_size(&mut self, size: f32);
    fn set_label(&mut self, label: &str);
    fn set_background_color(&mut self, color: Rgba);
    fn set_padding(&mut self, padding: u32);
    fn set_border_radius(&mut self, radius: u32);
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
    fn font_size<S: Into<f32>>(mut self, size: S) -> Self
    where
        Self: Sized,
    {
        self.set_font_size(size.into());
        self
    }
    fn label<S: AsRef<str>>(mut self, label: S) -> Self
    where
        Self: Sized,
    {
        self.set_label(label.as_ref());
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
}

/// Internal functions
pub trait WidgetInternal {
    fn compute_size(&mut self, font: ab_glyph::FontArc);
    fn get_size(&self) -> Size;
    fn get_offset(&self) -> Offset;
    fn set_offset(&mut self, pos: Offset);
    fn draw(&mut self, font: ab_glyph::FontArc, buf: &Buffer);
    fn handle_click(&mut self, pos: Offset);
}

pub struct Widget {
    label: String,
    size: Size,
    pos: Offset,
    padding: (u32, u32, u32, u32),
    font_size: f32,
    background_color: Rgba,
    border_radius: u32,
}

impl WidgetBase for Widget {
    fn set_label(&mut self, label: &str) {
        self.label = label.to_owned();
    }
    fn set_size(&mut self, size: Size) {
        self.size = size;
    }
    fn set_pos(&mut self, pos: Offset) {
        self.pos = pos;
    }
    fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }
    fn set_background_color(&mut self, color: Rgba) {
        self.background_color = color;
    }
    fn set_padding(&mut self, padding: u32) {
        self.padding = [padding; 4].into();
    }
    fn set_border_radius(&mut self, radius: u32) {
        self.border_radius = radius;
    }
}

impl WidgetExt for Widget {
    fn new() -> Self {
        Self {
            size: Default::default(),
            pos: Default::default(),
            padding: (0, 0, 0, 0),
            font_size: 12.0,
            label: String::new(),
            background_color: Rgba::WHITE,
            border_radius: 0,
        }
    }
}

impl WidgetInternal for Widget {
    fn compute_size(&mut self, _: ab_glyph::FontArc) {}
    fn get_size(&self) -> Size {
        self.size
    }
    fn get_offset(&self) -> Offset {
        self.pos
    }
    fn set_offset(&mut self, pos: Offset) {
        self.pos = pos;
    }
    fn draw(&mut self, _: ab_glyph::FontArc, buf: &Buffer) {
        buf.fill_round_rect_aa(
            Rect::from((self.pos, self.size)),
            self.border_radius as i32,
            self.background_color,
        );
    }
    fn handle_click(&mut self, _: Offset) {}
}

pub trait WidgetGroup {
    fn create_group(self) -> Vec<Box<dyn WidgetBase>>;
}

impl<W: WidgetExt + 'static> WidgetGroup for W {
    fn create_group(self) -> Vec<Box<dyn WidgetBase>> {
        vec![Box::new(self)]
    }
}

impl<W: WidgetExt + 'static, const N: usize> WidgetGroup for [W; N] {
    fn create_group(self) -> Vec<Box<dyn WidgetBase>> {
        self.into_iter()
            .map(|w| Box::new(w) as Box<dyn WidgetBase>)
            .collect()
    }
}

macro_rules! tupled_group {
    ($($id:tt $name:ident),+) => {
        impl<$($name: WidgetExt + 'static),+> WidgetGroup for ($($name),+)
        {
            fn create_group(self) -> Vec<Box<dyn WidgetBase>> {
                vec![
                    $(Box::new(self.$id)),+
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
