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
        wl_buffer, wl_callback, wl_compositor, wl_keyboard, wl_pointer, wl_registry, wl_seat,
        wl_shm, wl_shm_pool, wl_surface,
    },
    Dispatch, Proxy, WEnum,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use xkbcommon_rs::xkb_state::StateComponent;

use crate::{
    event::{Button, Event, Modifiers, RawEvent, WidgetEvent, WindowEvent, WindowState},
    sys::{linux, wayland::window::TITLEBAR_HEIGHT},
};
use lite_graphics::Offset;

use super::Window;

pub(super) struct State {
    pub(super) running: bool,
    pub(super) windows: HashMap<u64, Rc<Window>>,
    pub(super) compositor: Option<wl_compositor::WlCompositor>,
    pub(super) shm: Option<wl_shm::WlShm>,
    pub(super) wm_base: Option<xdg_wm_base::XdgWmBase>,
    pub(super) events: VecDeque<crate::event::RawEvent>,
    pub(super) keymap_state: Option<xkbcommon_rs::State>,
    pub(super) mouse_event: RawEvent,
    is_framed_pointer: bool,
    pub(super) last_move: Offset,
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
        this: &mut Self,
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
            if capabilities.contains(wl_seat::Capability::Pointer) {
                let ptr = seat.get_pointer(qh, ());
                if ptr.version() < 5 {
                    this.is_framed_pointer = false;
                }
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

impl Dispatch<wl_pointer::WlPointer, ()> for State {
    fn event(
        this: &mut Self,
        _: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        fn button_from_ev(ev: u32) -> Button {
            match ev {
                0x110 => Button::Left,
                0x111 => Button::Right,
                0x112 => Button::Middle,
                0x113 => Button::Forward,
                0x114 => Button::Back,
                _ => Button::Other,
            }
        }

        match event {
            wl_pointer::Event::Enter { surface, .. } => {
                this.mouse_event.window = this
                    .windows
                    .values()
                    .find(|w| *w.base_surface.get().unwrap() == surface)
                    .unwrap()
                    .id()
            }
            wl_pointer::Event::Leave { .. } => this.mouse_event.window = 0,
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                if surface_y > super::window::TITLEBAR_HEIGHT as f64
                    && this.last_move.y > super::window::TITLEBAR_HEIGHT as i32
                {
                    let window = this.windows.get(&this.mouse_event.window).unwrap();
                    window.titlebar(Offset::new(surface_x as i32, surface_y as i32), false);
                    window.draw(None).unwrap();
                }
                this.last_move = Offset::new(surface_x as _, surface_y as _);
                if this.last_move.y < super::window::TITLEBAR_HEIGHT as i32 {
                    let window = this.windows.get(&this.mouse_event.window).unwrap();
                    window.titlebar(this.last_move, false);
                    window.draw(None).unwrap();
                }
                this.mouse_event.event = Event::Widget(WidgetEvent::Move(
                    surface_x as i32,
                    surface_y as i32 - TITLEBAR_HEIGHT as i32,
                ));
                if !this.is_framed_pointer {
                    this.events.push_back(this.mouse_event);
                }
            }
            wl_pointer::Event::Button { button, state, .. } => {
                match state {
                    WEnum::Value(wl_pointer::ButtonState::Pressed) => {
                        if this.last_move.y < super::window::TITLEBAR_HEIGHT as i32 {
                            let window = this.windows.get(&this.mouse_event.window).unwrap();
                            window.titlebar(this.last_move, true);
                            window.draw(None).unwrap();
                        }
                        this.mouse_event.event = Event::Widget(WidgetEvent::ButtonPress(
                            button_from_ev(button),
                            this.last_move.x,
                            this.last_move.y - super::window::TITLEBAR_HEIGHT as i32,
                        ))
                    }
                    WEnum::Value(wl_pointer::ButtonState::Released) => {
                        if this.last_move.y < super::window::TITLEBAR_HEIGHT as i32 {
                            let window = this.windows.get(&this.mouse_event.window).unwrap();
                            let pos = this.last_move;
                            let width = super::window::WIDTH;
                            if pos.x < width as i32 - 4 && pos.x > width as i32 - 28 {
                                window.base_surface.get().unwrap().destroy();
                                this.windows.remove(&this.mouse_event.window).unwrap();
                                if this.windows.is_empty() {
                                    this.running = false;
                                }
                            } else if pos.x < width as i32 - 36 && pos.x > width as i32 - 60 {
                                window.xdg_surface.get().unwrap().1.set_minimized();
                            } else if pos.x < width as i32 - 68 && pos.x > width as i32 - 92 {
                                //window.xdg_surface.get().unwrap().1.set_maximized();
                            } else {
                                window.draw(None).unwrap();
                            }
                        }
                        this.mouse_event.event = Event::Widget(WidgetEvent::ButtonRelease(
                            button_from_ev(button),
                            this.last_move.x,
                            this.last_move.y - super::window::TITLEBAR_HEIGHT as i32,
                        ))
                    }
                    _ => {}
                }
                if !this.is_framed_pointer {
                    this.events.push_back(this.mouse_event);
                }
            }
            wl_pointer::Event::Frame => {
                if this.is_framed_pointer {
                    this.events.push_back(this.mouse_event);
                    this.mouse_event = RawEvent {
                        window: 0,
                        event: Event::Unknown,
                    }
                }
            }
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

impl Dispatch<wl_callback::WlCallback, u64> for State {
    fn event(
        this: &mut Self,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        window: &u64,
        _: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            let win = &this.windows[window];
            win.buffer.borrow_mut().take().unwrap().destroy();
            let buffer = win.shm.get().unwrap().0.create_buffer(
                0,
                super::window::WIDTH as i32,
                super::window::HEIGHT as i32 + super::window::TITLEBAR_HEIGHT as i32,
                super::window::WIDTH as i32 * 4,
                wl_shm::Format::Argb8888,
                qh,
                win.id(),
            );
            let surface = win.base_surface.get().unwrap();
            surface.attach(Some(&buffer), 0, 0);
            win.buffer.borrow_mut().replace(buffer);
            surface.damage(0, 0, i32::MAX, i32::MAX);
            surface.commit();
        }
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
            surface.attach(window.buffer.borrow().as_ref(), 0, 0);
            surface.commit();
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, u64> for State {
    fn event(
        this: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        win_id: &u64,
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            xdg_toplevel::Event::Configure { states, .. } => {
                let maximized = *states.first().unwrap_or(&0) > 0;
                let fullscreen = *states.get(1).unwrap_or(&0) > 0;
                let activated = *states.get(3).unwrap_or(&0) > 0;
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
                this.events.push_back(RawEvent {
                    window: *win_id,
                    event,
                });
            }
            xdg_toplevel::Event::Close => {
                this.windows.remove(win_id);
            }
            _ => {}
        }
    }
}

pub(crate) struct App {
    pub(super) state: RefCell<State>,
    pub(super) event_queue: RefCell<wayland_client::EventQueue<State>>,
    pub(super) qh: wayland_client::QueueHandle<State>,
}

impl App {
    pub(crate) fn new() -> crate::Result<Rc<Self>> {
        let conn = wayland_client::Connection::connect_to_env()?;
        let event_queue = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();
        let _registry = display.get_registry(&qh, ());

        let state = State {
            running: true,
            windows: HashMap::new(),
            compositor: None,
            shm: None,
            wm_base: None,
            events: VecDeque::new(),
            keymap_state: None,
            mouse_event: RawEvent {
                window: 0,
                event: Event::Unknown,
            },
            is_framed_pointer: true,
            last_move: Offset::default(),
        };

        Ok(Rc::new(Self {
            state: RefCell::new(state),
            event_queue: RefCell::new(event_queue),
            qh,
        }))
    }
    pub(crate) fn get_event(&self) -> crate::Result<Option<crate::event::RawEvent>> {
        let mut state = self.state.borrow_mut();
        if state.running {
            if state.events.is_empty() {
                self.event_queue
                    .borrow_mut()
                    .blocking_dispatch(&mut state)?;
            }
            Ok(Some(state.events.pop_front().unwrap_or(RawEvent {
                window: 0,
                event: Event::Unknown,
            })))
        } else {
            Ok(None)
        }
    }
}
