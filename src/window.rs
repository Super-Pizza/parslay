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
    pub fn render<V: super::IntoView + 'static>(
        &mut self,
        f: impl Fn() -> V + 'static,
    ) -> crate::Result<()> {
        let view = f();
        let buffer = view.render();
        self.inner.draw(buffer)
    }
}
