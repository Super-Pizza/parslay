use crate::sys;

pub struct App {
    pub(crate) inner: sys::app::App,
}

impl App {
    pub fn new() -> crate::Result<Self> {
        let inner = sys::app::App::new()?;
        Ok(Self { inner })
    }
    pub fn run(&mut self) -> crate::Result<()> {
        self.inner.run()
    }
}
