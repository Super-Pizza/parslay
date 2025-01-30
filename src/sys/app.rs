use std::rc::Rc;

use super::{wayland, x11};

pub(crate) enum App {
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
    X11(Rc<x11::App>),
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
    Wayland(Rc<wayland::App>),
    #[cfg(target_os = "windows")]
    Windows,
}

impl App {
    pub(crate) fn new() -> Result<Self, crate::Error> {
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
        if super::wayland::has_wayland() {
            Ok(Self::Wayland(wayland::App::new()?))
        } else {
            Ok(Self::X11(x11::App::new()?))
        }
    }
    pub(crate) fn run(&self) -> Result<(), crate::Error> {
        match self {
            Self::X11(app) => app.run(),
            Self::Wayland(app) => app.run(),
        }
    }
}
