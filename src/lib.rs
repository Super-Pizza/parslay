pub mod app;
pub mod error;
pub mod event;
mod sys;
mod text;
mod themes;
pub mod widgets;
mod window;

use app::App;
pub use error::Error;
pub use error::Result;
pub use lite_graphics::color::{Color, Rgba};
pub use themes::FrameType;
use widgets::IntoWidget;

use lite_graphics::Size as ComputedSize;
pub use widgets::{
    WidgetBase, WidgetExt, WidgetGroup,
    button::button,
    drop_down::drop_down,
    input::{dyn_input, input},
    label::{dyn_label, label},
    stack::hstack,
    stack::vstack,
};
use window::Window;

pub use floem_reactive as reactive;

pub mod prelude {
    pub use super::reactive::RwSignal;
    pub use super::{Color, Rgba};
    pub use super::{
        FrameType, Sizing::*, WidgetBase, WidgetExt, WidgetGroup, button, drop_down, dyn_input,
        dyn_label, hstack, input, label, launch, vstack,
    };
}

pub fn launch<V: IntoWidget + 'static>(view: impl FnOnce() -> V + 'static) -> crate::Result<()> {
    let app = App::new()?;
    let window = Window::new(&app)?;
    window.render(view)?;
    app.run()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sizing {
    /// Size shouldn't change. May change if the size is too small.
    Fixed(u32),
    /// Stretch with weight proportional to the size. Set to 0 to fit without stretch.
    Stretch(u32),
    /// Stretch as much as possible. Similar to `Stretch` with infinite weight. Ignores the given size.
    Fill,
}

impl Default for Sizing {
    fn default() -> Self {
        Self::Stretch(0)
    }
}

#[derive(Clone, Copy, Default)]
pub struct Size {
    pub w: Sizing,
    pub h: Sizing,
}

impl Size {
    pub fn new(w: Sizing, h: Sizing) -> Self {
        Self { w, h }
    }
    pub fn fixed(w: u32, h: u32) -> Self {
        Self {
            w: Sizing::Fixed(w),
            h: Sizing::Fixed(h),
        }
    }
    pub fn stretch(w_fac: u32, h_fac: u32) -> Self {
        Self {
            w: Sizing::Stretch(w_fac),
            h: Sizing::Stretch(h_fac),
        }
    }
    pub fn fill() -> Self {
        Self {
            w: Sizing::Fill,
            h: Sizing::Fill,
        }
    }
}

impl From<lite_graphics::Size> for Size {
    fn from(value: lite_graphics::Size) -> Self {
        Self::new(Sizing::Fixed(value.w), Sizing::Fixed(value.h))
    }
}
