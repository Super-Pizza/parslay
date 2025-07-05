use std::{cell::RefCell, rc::Rc};

use lite_graphics::Size;
use x11rb::{
    connection::Connection,
    protocol::{
        xproto::{Atom, AtomEnum, ConnectionExt as _, GetPropertyReply, KeyButMask, Screen},
        Event,
    },
    rust_connection::RustConnection,
};

use crate::{
    event::{Button, Modifiers, RawEvent, WidgetEvent, WindowEvent, WindowState},
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
        fn get_property(
            conn: &RustConnection,
            win: u32,
            atom: Atom,
            ty: AtomEnum,
        ) -> crate::Result<GetPropertyReply> {
            let mut props = conn.get_property(false, win, atom, ty, 0, 1)?.reply()?;
            if props.bytes_after != 0 {
                let len = props.bytes_after / 4 + 1;
                props = conn.get_property(false, win, atom, ty, 0, len)?.reply()?;
            }
            Ok(props)
        }
        let unknown = Ok(Some(RawEvent {
            window: 0,
            event: crate::event::Event::Unknown,
        }));
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
                unknown
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
                .fold(Default::default(), |st, i| Modifiers(i) | st);

                let key = if mods & Modifiers::SHIFT == Modifiers::SHIFT {
                    linux::key_from_xkb(keysym).shift()
                } else {
                    linux::key_from_xkb(keysym)
                };

                Ok(Some(RawEvent {
                    window: event.event as u64,
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
                .fold(Default::default(), |st, i| Modifiers(i) | st);

                let key = if mods & Modifiers::SHIFT == Modifiers::SHIFT {
                    linux::key_from_xkb(keysym).shift()
                } else {
                    linux::key_from_xkb(keysym)
                };

                Ok(Some(RawEvent {
                    window: event.event as u64,
                    event: crate::event::Event::Window(WindowEvent::KeyRelease(mods, key)),
                }))
            }
            Event::ButtonPress(event) => {
                let ev = crate::event::Event::Widget(WidgetEvent::ButtonPress(
                    Button::from_code(event.detail),
                    event.event_x as i32,
                    event.event_y as i32,
                ));
                Ok(Some(RawEvent {
                    window: event.event as u64,
                    event: ev,
                }))
            }
            Event::ButtonRelease(event) => {
                let ev = crate::event::Event::Widget(WidgetEvent::ButtonRelease(
                    Button::from_code(event.detail),
                    event.event_x as i32,
                    event.event_y as i32,
                ));
                Ok(Some(RawEvent {
                    window: event.event as u64,
                    event: ev,
                }))
            }
            Event::MotionNotify(event) => {
                let ev = crate::event::Event::Widget(WidgetEvent::Move(
                    event.event_x as i32,
                    event.event_y as i32,
                ));
                Ok(Some(RawEvent {
                    window: event.event as u64,
                    event: ev,
                }))
            }
            Event::Error(e) => Err(e.into()),
            Event::ConfigureNotify(event) => {
                let windows = self.windows.borrow();
                let curr = windows
                    .iter()
                    .find(|w| w.id() == event.window as _)
                    .unwrap();
                let mut curr_state = curr.state.borrow_mut();

                let mut curr_size = curr.size.borrow_mut();
                let props = get_property(
                    &self.conn,
                    event.window,
                    self.atoms._NET_WM_STATE,
                    AtomEnum::ATOM,
                )?;

                let atoms = props.value32().unwrap().collect::<Vec<_>>();
                let maximized = atoms.contains(&self.atoms._NET_WM_STATE_MAXIMIZED_HORZ)
                    && atoms.contains(&self.atoms._NET_WM_STATE_MAXIMIZED_VERT);
                let fullscreen = atoms.contains(&self.atoms._NET_WM_STATE_FULLSCREEN);
                let activated = atoms.contains(&self.atoms._NET_WM_STATE_FOCUSED);

                let ev = if maximized && *curr_state != WindowState::Maximized {
                    WindowEvent::StateChange(WindowState::Maximized)
                } else if fullscreen && *curr_state != WindowState::Fullscreen {
                    WindowEvent::StateChange(WindowState::Fullscreen)
                } else if activated && *curr_state != WindowState::Activated {
                    WindowEvent::StateChange(WindowState::Activated)
                } else if event.width as u32 != curr_size.w || event.height as u32 != curr_size.h {
                    *curr_size = Size::new(event.width as _, event.height as _);
                    WindowEvent::Resize(event.width as _, event.height as _)
                } else {
                    return unknown;
                };
                if let WindowEvent::StateChange(st) = ev {
                    *curr_state = st
                };

                Ok(Some(RawEvent {
                    window: event.window as _,
                    event: crate::event::Event::Window(ev),
                }))
            }
            _ => unknown,
        }
    }
}
