pub mod app;
pub mod error;
mod sys;
mod window;

use app::App;
pub use error::Error;
pub use error::Result;
use lite_graphics::draw::Buffer;
use lite_graphics::draw::Rgba;
use window::Window;

pub fn launch<V: IntoView + 'static>(view: impl Fn() -> V + 'static) -> crate::Result<()> {
    let mut app = App::new()?;
    let mut window = Window::new(&mut app)?;
    window.render(view)?;
    app.run()
}

pub trait IntoView {
    fn render(&self) -> Buffer;
}

/// Empty window
impl IntoView for () {
    fn render(&self) -> Buffer {
        let buf = Buffer::new(800, 600);
        buf.fill_rect(buf.size().into(), Rgba::WHITE);
        buf
    }
}
