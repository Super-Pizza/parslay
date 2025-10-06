use std::{collections::HashMap, rc::Rc};

use lite_graphics::{Drawable, Size, color::Rgba};

pub type FrameFn = Rc<dyn Fn(&dyn Drawable, Size, Rgba)>;

pub(crate) fn get_default_theme() -> HashMap<String, FrameFn> {
    let mut map = HashMap::<String, FrameFn>::new();

    map.insert(
        "Box".to_string(),
        Rc::new(|buf, size, color| {
            buf.fill_rect(size.into(), color.into());
        }),
    );

    map.insert(
        "Button".to_string(),
        Rc::new(|buf, size, color| {
            buf.fill_round_rect_aa(size.into(), 4, color.into());
        }),
    );

    map.insert(
        "Frame".to_string(),
        Rc::new(|buf, size, color| {
            buf.fill_round_rect_aa(size.into(), 8, color.into());
        }),
    );

    map.insert(
        "InputFrame".to_string(),
        Rc::new(|buf, size, color| {
            let border_color = if color.intensity() >= 128 {
                Rgba::from([color.r - 32, color.r - 32, color.r - 32, color.a])
            } else {
                Rgba::from([color.r + 32, color.r + 32, color.r + 32, color.a])
            };
            buf.fill_round_rect_aa(size.into(), 8, color.into());
            buf.round_rect_aa(size.into(), 8, border_color.into());
        }),
    );

    map
}

thread_local! {
pub(crate) static NONE_FN: FrameFn = Rc::new(|_, _, _| {});
}

pub enum FrameType {
    None,
    Box,
    Button,
    Frame,
    InputFrame,
    Custom(String),
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for FrameType {
    fn to_string(&self) -> String {
        match self {
            Self::None => String::new(),
            Self::Box => "Box".to_string(),
            Self::Button => "Button".to_string(),
            Self::Frame => "Frame".to_string(),
            Self::InputFrame => "InputFrame".to_string(),
            Self::Custom(s) => s.clone(),
        }
    }
}
