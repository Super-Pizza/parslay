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
        FrameType, Rgba, WidgetBase, WidgetExt, WidgetGroup, button, drop_down, dyn_input,
        dyn_label, hstack, input, label, launch, vstack,
    };
}

pub fn launch<V: IntoWidget + 'static>(view: impl FnOnce() -> V + 'static) -> crate::Result<()> {
    let app = App::new()?;
    let window = Window::new(&app)?;
    window.render(view)?;
    app.run()
}
