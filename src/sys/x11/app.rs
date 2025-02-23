use std::{cell::RefCell, rc::Rc};

use x11rb::{
    connection::Connection,
    protocol::{xproto::Screen, Event},
    rust_connection::RustConnection,
};

use crate::event::RawEvent;

use super::Window;

pub(crate) struct App {
    pub(super) conn: RustConnection,
    pub(super) screen: Screen,
    pub(super) atoms: Atoms,
    pub(super) windows: RefCell<Vec<Rc<Window>>>,
}

x11rb::atom_manager! {
    pub Atoms: AtomsCookie {
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
        _NET_WM_NAME,
        UTF8_STRING,
    }
}

impl App {
    pub(crate) fn new() -> crate::Result<Rc<Self>> {
        let (conn, dpy) = x11rb::connect(None)?;
        let conn = conn;
        let screen = conn.setup().roots[dpy].clone();
        let atoms_cookie = Atoms::new(&conn)?;
        let atoms = atoms_cookie.reply()?;

        Ok(Rc::new(Self {
            conn,
            screen,
            atoms,
            windows: RefCell::new(vec![]),
        }))
    }
    pub(crate) fn get_event(self: &Rc<Self>) -> crate::Result<Option<crate::event::RawEvent>> {
        let event = self.conn.wait_for_event()?;
        match event {
            Event::ClientMessage(event) => {
                let data = event.data.as_data32();
                if event.format == 32 && data[0] == self.atoms.WM_DELETE_WINDOW {
                    let mut windows = self.windows.borrow_mut();
                    windows.retain(|w| w.id() != event.window as _);
                    if windows.is_empty() {
                        return Ok(None);
                    }
                }
                Ok(Some(RawEvent {
                    window: event.window as _,
                    event: crate::event::Event::Unknown,
                }))
            }
            Event::Error(e) => Err(e.into()),
            _ => Ok(Some(RawEvent {
                window: 0,
                event: crate::event::Event::Unknown,
            })),
        }
    }
}
