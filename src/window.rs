use crate::sys;

pub struct Window {
    pub(crate) inner: sys::window::Window,
}

impl Window {
    pub fn new(app: &mut crate::app::App) -> crate::Result<Self> {
        let inner = sys::window::Window::new(&app.inner)?;
        let this = Self { inner };
        Ok(this)
    }
}
