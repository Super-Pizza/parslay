use std::rc::Rc;

use lite_graphics::draw::Buffer;

use super::{wayland, x11};

platform!(
    pub(crate) enum Window {
        linux =>
            X11(Rc<x11::Window>),
            Wayland(Rc<wayland::Window>),
    }
);

impl Window {
    pub(crate) fn new(app: &super::app::App) -> Result<Self, crate::Error> {
        platform!(match app {
            super::app::App::X11(app) if linux => Ok(Self::X11(x11::Window::new(app)?)),
            super::app::App::Wayland(app) if linux => Ok(Self::Wayland(wayland::Window::new(app)?)),
        })
    }
    pub(crate) fn draw(&self, buf: Buffer) -> crate::Result<()> {
        platform!(match self {
            Self::Wayland(window) if linux => window.draw(buf),
            Self::X11(window) if linux => window.draw(buf),
        })
    }
    #[allow(unused)]
    pub(crate) fn id(&self) -> u64 {
        platform!(match self {
            Self::Wayland(window) if linux => window.id(),
            Self::X11(window) if linux => window.id(),
        })
    }
}

impl Clone for Window {
    fn clone(&self) -> Self {
        platform!(match self {
            Self::Wayland(window) if linux => Self::Wayland(window.clone()),
            Self::X11(window) if linux => Self::X11(window.clone()),
        })
    }
}
