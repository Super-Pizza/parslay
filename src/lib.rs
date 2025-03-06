pub mod app;
pub mod error;
pub mod event;
mod sys;
pub mod widgets;
mod window;

use app::App;
pub use error::Error;
pub use error::Result;
pub use lite_graphics::draw::Rgba;
pub use widgets::{label::label, stack::hstack, stack::vstack, WidgetBase, WidgetExt, WidgetGroup};
use window::Window;

pub fn launch<V: IntoView + 'static>(view: impl Fn() -> V + 'static) -> crate::Result<()> {
    let app = App::new()?;
    let window = Window::new(&app)?;
    window.render(view)?;
    app.run()
}

pub trait IntoView {
    type Widget: WidgetExt;
    fn create(self, window: Window) -> Self::Widget
    where
        Self::Widget: WidgetExt;
}
