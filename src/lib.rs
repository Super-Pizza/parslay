pub mod app;
pub mod error;
mod sys;
mod window;

use app::App;
pub use error::Error;
pub use error::Result;
use window::Window;

pub fn launch<V: IntoView + 'static>(_view: impl Fn() -> V + 'static) -> crate::Result<()> {
    let mut app = App::new()?;
    let _window = Window::new(&mut app)?;
    app.run()
}

pub trait IntoView {}

/// Empty window
impl IntoView for () {}
