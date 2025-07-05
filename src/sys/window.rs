use std::rc::Rc;

use lite_graphics::draw::Buffer;

platform!(
    pub(crate) enum Window {
        linux =>
            X11(Rc<super::x11::Window>),
            Wayland(Rc<super::wayland::Window>);
        windows =>
            Windows(Rc<super::windows::Window>)
    }
);

impl Window {
    pub(crate) fn new(app: &super::app::App) -> Result<Self, crate::Error> {
        platform!(match app {
            super::app::App::X11(app) if linux => Ok(Self::X11(super::x11::Window::new(app)?)),
            super::app::App::Wayland(app) if linux =>
                Ok(Self::Wayland(super::wayland::Window::new(app)?)),
            super::app::App::Windows(app) if windows =>
                Ok(Self::Windows(super::windows::Window::new(app)?)),
        })
    }
    pub(crate) fn draw(&self, buf: Buffer) -> crate::Result<()> {
        platform!(match self {
            Self::Wayland(window) if linux => window.draw(Some(buf)),
            Self::X11(window) if linux => window.draw(buf),
            Self::Windows(window) if windows => window.draw(buf),
        })
    }
    #[allow(unused)]
    pub(crate) fn id(&self) -> u64 {
        platform!(match self {
            Self::Wayland(window) if linux => window.id(),
            Self::X11(window) if linux => window.id(),
            Self::Windows(window) if windows => window.id(),
        })
    }
    pub(crate) fn set_cursor(&self, cursor: crate::app::CursorType) {
        platform!(match self {
            Self::X11(window) if linux => window.set_cursor(cursor),
            Self::Wayland(window) if linux => window.set_cursor(cursor),
            Self::Windows(window) if windows => window.set_cursor(cursor),
        })
    }
}

impl Clone for Window {
    fn clone(&self) -> Self {
        platform!(match self {
            Self::Wayland(window) if linux => Self::Wayland(window.clone()),
            Self::X11(window) if linux => Self::X11(window.clone()),
            Self::Windows(window) if windows => Self::Windows(window.clone()),
        })
    }
}
