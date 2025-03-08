#![allow(clippy::collapsible_match)]
// TODO: Remove this when no longer needed.
#![allow(dead_code)]
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    ffi::CStr,
    num::NonZero,
    rc::Rc,
};

use nix::sys::mman::{mmap, MapFlags, ProtFlags};
use wayland_client::{
    delegate_noop,
    protocol::{
        wl_buffer, wl_compositor, wl_keyboard, wl_registry, wl_seat, wl_shm, wl_shm_pool,
        wl_surface,
    },
    Dispatch, WEnum,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use xkbcommon_rs::xkb_state::StateComponent;

use crate::{
    event::{Event, Modifiers, RawEvent, WindowEvent, WindowState},
    sys::linux,
};

use super::Window;

pub(super) struct State {
    pub(super) running: bool,
    pub(super) windows: HashMap<u64, Rc<Window>>,
    pub(super) compositor: Option<wl_compositor::WlCompositor>,
    pub(super) shm: Option<wl_shm::WlShm>,
    pub(super) wm_base: Option<xdg_wm_base::XdgWmBase>,
    pub(super) events: VecDeque<crate::event::RawEvent>,
    pub(super) keymap_state: Option<xkbcommon_rs::State>,
}

delegate_noop!(State: ignore wl_compositor::WlCompositor);
delegate_noop!(State: ignore wl_shm::WlShm);

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match &interface[..] {
                "wl_compositor" => {
                    let compositor =
                        registry.bind::<wl_compositor::WlCompositor, _, _>(name, 1, qh, ());
                    state.compositor = Some(compositor);
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, 1, qh, ());
                    state.shm = Some(shm);
                }
                "wl_seat" => {
                    registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                }
                "xdg_wm_base" => {
                    let wm_base = registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, 1, qh, ());
                    state.wm_base = Some(wm_base);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        seat: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(capabilities),
        } = event
        {
            if capabilities.contains(wl_seat::Capability::Keyboard) {
                seat.get_keyboard(qh, ());
            }
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(
        this: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Key { key, state, .. } => {
                let keymap = this.keymap_state.as_ref().unwrap();
                let mods = ["Shift", "Control", "Mod1", "Mod4"]
                    .iter()
                    .enumerate()
                    .map(|(idx, name)| {
                        let active =
                            keymap.mod_name_is_active(name, StateComponent::MODS_EFFECTIVE);
                        (active.unwrap() as u8) << idx
                    })
                    .fold(Default::default(), |st, id| st | Modifiers(id));
                let sym = keymap
                    .key_get_one_sym(8 + key) // `key` is evdev code, but xkb codes are 8 over.
                    .unwrap();
                let key = linux::key_from_xkb(sym.raw());
                let win_evt = match state {
                    WEnum::Value(wl_keyboard::KeyState::Pressed) => {
                        WindowEvent::KeyPress(mods, key)
                    }
                    WEnum::Value(wl_keyboard::KeyState::Released) => {
                        WindowEvent::KeyRelease(mods, key)
                    }
                    _ => return,
                };
                this.events.push_back(RawEvent {
                    window: 0,
                    event: Event::Window(win_evt),
                });
            }
            wl_keyboard::Event::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
                this.keymap_state.as_mut().unwrap().update_mask(
                    mods_depressed,
                    mods_latched,
                    mods_locked,
                    group as _,
                    0,
                    0,
                );
            }
            wl_keyboard::Event::Keymap { fd, size, .. } => unsafe {
                let addr = mmap(
                    None,
                    NonZero::new(size as usize).unwrap(),
                    ProtFlags::PROT_READ,
                    MapFlags::MAP_PRIVATE,
                    fd,
                    0,
                )
                .unwrap();
                let data = CStr::from_ptr(addr.as_ptr() as *const _).to_string_lossy();
                let keymap = xkbcommon_rs::Keymap::new_from_string(
                    xkbcommon_rs::Context::new(0).unwrap(),
                    &data,
                    xkbcommon_rs::KeymapFormat::TextV1,
                    0,
                )
                .unwrap();
                let state = xkbcommon_rs::State::new(keymap);
                this.keymap_state = Some(state);
            },
            _ => {}
        }
    }
}

impl Dispatch<xdg_wm_base::XdgWmBase, ()> for State {
    fn event(
        _: &mut Self,
        wm_base: &xdg_wm_base::XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            wm_base.pong(serial);
        }
    }
}

impl Dispatch<wl_surface::WlSurface, u64> for State {
    fn event(
        _: &mut Self,
        _: &wl_surface::WlSurface,
        _: wl_surface::Event,
        _: &u64,
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, u64> for State {
    fn event(
        _: &mut Self,
        _: &wl_shm_pool::WlShmPool,
        _: wl_shm_pool::Event,
        _: &u64,
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, u64> for State {
    fn event(
        _: &mut Self,
        _: &wl_buffer::WlBuffer,
        _: wl_buffer::Event,
        _: &u64,
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<xdg_surface::XdgSurface, u64> for State {
    fn event(
        this: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        window: &u64,
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            xdg_surface.ack_configure(serial);

            let window = this.windows.get(window).unwrap();
            let surface = window.base_surface.get().unwrap();
            if let Some((ref buffer, _)) = window.buffer.get() {
                surface.attach(Some(buffer), 0, 0);
                surface.commit();
            }
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, u64> for State {
    fn event(
        this: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        window: &u64,
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            xdg_toplevel::Event::Configure { states, .. } => {
                let window = this.windows.get(window).unwrap();
                let maximized = states[0] > 0;
                let fullscreen = states[1] > 0;
                let activated = states[3] > 0;
                let st = if maximized {
                    WindowState::Maximized
                } else if fullscreen {
                    WindowState::Fullscreen
                } else if activated {
                    WindowState::Activated
                } else {
                    return;
                };
                let event = Event::Window(WindowEvent::StateChange(st));
                window.events.borrow_mut().push_back(event);
            }
            xdg_toplevel::Event::Close => {
                this.windows.remove(window);
            }
            _ => {}
        }
    }
}

pub(crate) struct App {
    pub(super) state: RefCell<State>,
    pub(super) event_queue: RefCell<wayland_client::EventQueue<State>>,
    pub(super) qh: wayland_client::QueueHandle<State>,
    registry: wl_registry::WlRegistry,
    conn: wayland_client::Connection,
}

impl App {
    pub(crate) fn new() -> crate::Result<Rc<Self>> {
        let conn = wayland_client::Connection::connect_to_env()?;
        let event_queue = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();
        let registry = display.get_registry(&qh, ());

        let state = State {
            running: true,
            windows: HashMap::new(),
            compositor: None,
            shm: None,
            wm_base: None,
            events: VecDeque::new(),
            keymap_state: None,
        };

        Ok(Rc::new(Self {
            conn,
            state: RefCell::new(state),
            event_queue: RefCell::new(event_queue),
            qh,
            registry,
        }))
    }
    pub(crate) fn get_event(&self) -> crate::Result<Option<crate::event::RawEvent>> {
        let mut state = self.state.borrow_mut();
        if state.running {
            if state.events.is_empty() {
                self.event_queue
                    .borrow_mut()
                    .blocking_dispatch(&mut state)?;
                let mut events = VecDeque::new();
                for window in state.windows.values() {
                    events.append(&mut window.get_events()?);
                }
                state.events.append(&mut events);
            }
            Ok(state.events.pop_front())
        } else {
            Ok(None)
        }
    }
}
