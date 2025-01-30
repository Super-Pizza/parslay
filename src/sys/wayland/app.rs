#![allow(clippy::collapsible_match)]
// TODO: Remove this when no longer needed.
#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use raw_window_handle::RawWindowHandle;
use wayland_client::protocol::wl_registry;

use super::{window::State, Window};

pub(crate) struct App {
    conn: wayland_client::Connection,
    pub(super) event_queue: RefCell<wayland_client::EventQueue<State>>,
    qh: wayland_client::QueueHandle<State>,
    registry: wl_registry::WlRegistry,
    pub(super) windows: RefCell<HashMap<RawWindowHandle, Rc<Window>>>,
}

impl App {
    pub(crate) fn new() -> crate::Result<Rc<Self>> {
        let conn = wayland_client::Connection::connect_to_env()?;
        let event_queue = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();
        let registry = display.get_registry(&qh, ());

        Ok(Rc::new(Self {
            conn,
            event_queue: RefCell::new(event_queue),
            qh,
            registry,
            windows: RefCell::new(HashMap::new()),
        }))
    }
    pub(crate) fn run(&self) -> crate::Result<()> {
        loop {
            let mut entries = self.windows.borrow_mut();
            let mut result = Ok(());
            entries.retain(|_, win| {
                let mut state = win.state.borrow_mut();
                if state.running {
                    match self.event_queue.borrow_mut().blocking_dispatch(&mut state) {
                        Ok(_) => {}
                        Err(e) => result = Err(e),
                    }
                }
                state.running
            });
            result?;
            if entries.is_empty() {
                break;
            }
        }
        Ok(())
    }
}
