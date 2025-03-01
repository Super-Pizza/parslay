use std::{cell::RefCell, rc::Rc};

use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{AtomEnum, ConnectionExt, KeyButMask, Screen},
        Event,
    },
    rust_connection::RustConnection,
};

use crate::{
    event::{Modifiers, RawEvent, WindowEvent, WindowState},
    sys::linux,
};

use super::Window;

pub(crate) struct App {
    pub(super) conn: RustConnection,
    pub(super) screen: Screen,
    pub(super) atoms: Atoms,
    pub(super) windows: RefCell<Vec<Rc<Window>>>,
    keymap: Vec<u32>,
}

x11rb::atom_manager! {
    pub Atoms: AtomsCookie {
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
        _NET_WM_NAME,
        _NET_WM_STATE,
        _NET_WM_STATE_MAXIMIZED_VERT,
        _NET_WM_STATE_MAXIMIZED_HORZ,
        _NET_WM_STATE_FULLSCREEN,
        _NET_WM_STATE_FOCUSED,
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

        let mapping_reply = conn.get_keyboard_mapping(8, 247)?.reply()?;
        let keymap = mapping_reply
            .keysyms
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                (idx % mapping_reply.keysyms_per_keycode as usize == 0).then_some(*item)
            })
            .collect();

        Ok(Rc::new(Self {
            conn,
            screen,
            atoms,
            windows: RefCell::new(vec![]),
            keymap,
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
            Event::KeyPress(event) => {
                let keysym = self.keymap[event.detail as usize - 8];
                let mods = [
                    KeyButMask::SHIFT,
                    KeyButMask::CONTROL,
                    KeyButMask::MOD1,
                    KeyButMask::MOD4,
                ]
                .iter()
                .enumerate()
                .map(|(idx, &name)| ((event.state & name == name) as u8) << idx)
                .map(Modifiers)
                .fold(Default::default(), |st, i| i | st);

                let key = if mods & Modifiers::SHIFT == Modifiers::SHIFT {
                    linux::key_from_xkb(keysym).shift()
                } else {
                    linux::key_from_xkb(keysym)
                };

                Ok(Some(RawEvent {
                    window: event.root as u64,
                    event: crate::event::Event::Window(WindowEvent::KeyPress(mods, key)),
                }))
            }
            Event::KeyRelease(event) => {
                let keysym = self.keymap[event.detail as usize - 8];
                let mods = [
                    KeyButMask::SHIFT,
                    KeyButMask::CONTROL,
                    KeyButMask::MOD1,
                    KeyButMask::MOD4,
                ]
                .iter()
                .enumerate()
                .map(|(idx, &name)| ((event.state & name == name) as u8) << idx)
                .map(Modifiers)
                .fold(Default::default(), |st, i| i | st);

                let key = if mods & Modifiers::SHIFT == Modifiers::SHIFT {
                    linux::key_from_xkb(keysym).shift()
                } else {
                    linux::key_from_xkb(keysym)
                };

                Ok(Some(RawEvent {
                    window: event.root as u64,
                    event: crate::event::Event::Window(WindowEvent::KeyRelease(mods, key)),
                }))
            }
            Event::Error(e) => Err(e.into()),
            Event::ConfigureNotify(event) => {
                let windows = self.windows.borrow();
                let mut curr = windows
                    .iter()
                    .find(|w| w.id() == event.window as _)
                    .unwrap()
                    .state
                    .borrow_mut();
                let mut props = self
                    .conn
                    .get_property(
                        false,
                        event.window,
                        self.atoms._NET_WM_STATE,
                        AtomEnum::ATOM,
                        0,
                        1,
                    )?
                    .reply()?;
                if props.bytes_after != 0 {
                    props = self
                        .conn
                        .get_property(
                            false,
                            event.window,
                            self.atoms._NET_WM_STATE,
                            AtomEnum::ATOM,
                            0,
                            props.bytes_after / 4 + 1,
                        )?
                        .reply()?;
                }
                let mut maximized_h = false;
                let mut maximized_v = false;
                let mut fullscreen = false;
                let mut activated = false;
                for atom in props.value32().unwrap() {
                    if atom == self.atoms._NET_WM_STATE_MAXIMIZED_HORZ {
                        maximized_h = true;
                    } else if atom == self.atoms._NET_WM_STATE_MAXIMIZED_VERT {
                        maximized_v = true;
                    } else if atom == self.atoms._NET_WM_STATE_FULLSCREEN {
                        fullscreen = true;
                    } else if atom == self.atoms._NET_WM_STATE_FOCUSED {
                        activated = true;
                    }
                }
                let st = if maximized_h && maximized_v {
                    WindowState::Maximized
                } else if fullscreen {
                    WindowState::Fullscreen
                } else if activated {
                    WindowState::Activated
                } else {
                    return Ok(Some(RawEvent {
                        window: 0,
                        event: crate::event::Event::Unknown,
                    }));
                };
                if st != *curr {
                    *curr = st;
                    Ok(Some(RawEvent {
                        window: event.window as _,
                        event: crate::event::Event::Window(WindowEvent::StateChange(st)),
                    }))
                } else {
                    Ok(Some(RawEvent {
                        window: 0,
                        event: crate::event::Event::Unknown,
                    }))
                }
            }
            _ => Ok(Some(RawEvent {
                window: 0,
                event: crate::event::Event::Unknown,
            })),
        }
    }
}
