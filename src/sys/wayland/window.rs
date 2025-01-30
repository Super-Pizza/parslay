use raw_window_handle::{RawWindowHandle, WaylandWindowHandle};
use wayland_client::Proxy;

use super::{app::State, App};
use std::{cell::RefCell, ptr::NonNull, rc::Rc};

pub(crate) struct Window {
    pub(super) state: RefCell<State>,
}

impl Window {
    pub(crate) fn new(app: &Rc<App>) -> crate::Result<Rc<Self>> {
        let state = State {
            running: true,
            base_surface: None,
            buffer: None,
            wm_base: None,
            xdg_surface: None,
            configured: false,
        };

        let window = Rc::new(Window {
            state: RefCell::new(state),
        });

        app.event_queue
            .borrow_mut()
            .roundtrip(&mut window.state.borrow_mut())
            .unwrap();

        app.windows
            .borrow_mut()
            .insert(window.clone().id(), window.clone());

        Ok(window)
    }
    pub(crate) fn id(&self) -> RawWindowHandle {
        let ptr = &mut self.state.borrow().base_surface.as_ref().unwrap().id();
        RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(ptr as *mut _ as *mut _).unwrap(),
        ))
    }
}
