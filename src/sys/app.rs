use std::rc::Rc;

platform!(
    pub(crate) enum App {
        linux =>
            X11(Rc<super::x11::App>),
            Wayland(Rc<super::wayland::App>);
        windows => Windows(Rc<super::windows::App>)
    }
);

impl App {
    pub(crate) fn new() -> Result<Self, crate::Error> {
        platform!(
            linux => if super::wayland::has_wayland() {
                Ok(Self::Wayland(super::wayland::App::new()?))
            } else {
                Ok(Self::X11(super::x11::App::new()?))
            };
            windows => Ok(Self::Windows(super::windows::App::new()?))
        );
    }
    pub(crate) fn get_events(&self) -> Result<Option<crate::event::RawEvent>, crate::Error> {
        platform!(match self {
            Self::X11(app) if linux => app.get_event(),
            Self::Wayland(app) if linux => app.get_event(),
            Self::Windows(app) if windows => app.get_event(),
        })
    }
}
