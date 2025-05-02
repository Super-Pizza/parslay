pub mod app;
pub mod error;
pub mod event;
mod sys;
mod text;
pub mod widgets;
mod window;

use app::App;
pub use error::Error;
pub use error::Result;
pub use lite_graphics::draw::Rgba;
use widgets::IntoWidget;
pub use widgets::{
    button::button,
    label::{dyn_label, label},
    stack::hstack,
    stack::vstack,
    WidgetBase, WidgetExt, WidgetGroup,
};
use window::Window;

pub use floem_reactive as reactive;

pub fn launch<V: IntoWidget + 'static>(view: impl FnOnce() -> V + 'static) -> crate::Result<()> {
    let app = App::new()?;
    let window = Window::new(&app)?;
    window.render(view)?;
    app.run()
}
