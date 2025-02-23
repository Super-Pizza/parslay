use std::rc::Rc;

use super::{wayland, x11};

platform!(
    pub(crate) enum App {
        linux =>
            X11(Rc<x11::App>),
            Wayland(Rc<wayland::App>),
    }
);

impl App {
    pub(crate) fn new() -> Result<Self, crate::Error> {
        platform!(
            linux => if super::wayland::has_wayland() {
                Ok(Self::Wayland(wayland::App::new()?))
            } else {
                Ok(Self::X11(x11::App::new()?))
            }
        )
    }
    pub(crate) fn get_events(&self) -> Result<Option<crate::event::RawEvent>, crate::Error> {
        platform!(match self {
            Self::X11(app) if linux => app.get_event(),
            Self::Wayland(app) if linux => app.get_event(),
        })
    }
}
