use std::rc::Rc;

use lite_graphics::draw::Buffer;

use super::{wayland, x11};

pub(crate) enum Window {
    #[cfg(all(
        unix,
        not(any(
            target_os = "redox",
            target_family = "wasm",
            target_os = "android",
            target_os = "ios",
            target_os = "macos"
        ))
    ))]
    X11(Rc<x11::Window>),
    #[cfg(all(
        unix,
        not(any(
            target_os = "redox",
            target_family = "wasm",
            target_os = "android",
            target_os = "ios",
            target_os = "macos"
        ))
    ))]
    Wayland(Rc<wayland::Window>),
    #[cfg(target_os = "windows")]
    Windows,
}

impl Window {
    pub(crate) fn new(app: &super::app::App) -> Result<Self, crate::Error> {
        match app {
            #[cfg(all(
                unix,
                not(any(
                    target_os = "redox",
                    target_family = "wasm",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "macos"
                ))
            ))]
            super::app::App::X11(app) => Ok(Self::X11(x11::Window::new(app)?)),
            #[cfg(all(
                unix,
                not(any(
                    target_os = "redox",
                    target_family = "wasm",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "macos"
                ))
            ))]
            super::app::App::Wayland(app) => Ok(Self::Wayland(wayland::Window::new(app)?)),
        }
    }
    pub(crate) fn draw(&self, buf: Buffer) -> crate::Result<()> {
        match self {
            #[cfg(all(
                unix,
                not(any(
                    target_os = "redox",
                    target_family = "wasm",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "macos"
                ))
            ))]
            Self::Wayland(window) => window.draw(buf),
            #[cfg(all(
                unix,
                not(any(
                    target_os = "redox",
                    target_family = "wasm",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "macos"
                ))
            ))]
            Self::X11(window) => window.draw(buf),
        }
    }
    #[allow(unused)]
    pub(crate) fn id(&self) -> u64 {
        match self {
            #[cfg(all(
                unix,
                not(any(
                    target_os = "redox",
                    target_family = "wasm",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "macos"
                ))
            ))]
            Self::Wayland(window) => window.id(),
            #[cfg(all(
                unix,
                not(any(
                    target_os = "redox",
                    target_family = "wasm",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "macos"
                ))
            ))]
            Self::X11(window) => window.id(),
        }
    }
}

impl Clone for Window {
    fn clone(&self) -> Self {
        match self {
            #[cfg(all(
                unix,
                not(any(
                    target_os = "redox",
                    target_family = "wasm",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "macos"
                ))
            ))]
            Self::Wayland(window) => Self::Wayland(window.clone()),
            #[cfg(all(
                unix,
                not(any(
                    target_os = "redox",
                    target_family = "wasm",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "macos"
                ))
            ))]
            Self::X11(window) => Self::X11(window.clone()),
        }
    }
}
