use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    ffi::CStr,
    num::NonZero,
    rc::Rc,
};

use lite_graphics::{Offset, Size};
use nix::sys::mman::{MapFlags, ProtFlags, mmap};
use wayland_client::{
    Dispatch, Proxy, WEnum, delegate_noop,
    protocol::{
        wl_buffer, wl_callback, wl_compositor, wl_keyboard, wl_pointer, wl_registry, wl_seat,
        wl_shm, wl_shm_pool, wl_surface,
    },
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};
use xkbcommon_rs::xkb_state::StateComponent;

use crate::{
    app::CursorType,
    event::{Button, Event, Modifiers, RawEvent, WidgetEvent, WindowEvent, WindowState},
    sys::{linux, wayland::window::TITLEBAR_HEIGHT},
};

use super::Window;

pub(super) struct State {
    pub(super) cursor: Option<super::cursor::Cursor>,
    pub(super) windows: HashMap<u64, Rc<Window>>,
    pub(super) wm_base: Option<xdg_wm_base::XdgWmBase>,
    pub(super) keyboard: Option<wl_keyboard::WlKeyboard>,
    pub(super) pointer: Option<wl_pointer::WlPointer>,
    pub(super) seat: Option<wl_seat::WlSeat>,
    pub(super) shm: Option<wl_shm::WlShm>,
    pub(super) compositor: Option<wl_compositor::WlCompositor>,
    pub(super) running: bool,
    pub(super) events: VecDeque<crate::event::RawEvent>,
    pub(super) keymap_state: Option<xkbcommon_rs::State>,
    pub(super) mouse_event: RawEvent,
    pub(super) buttons_held: [bool; 6],
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
        conn: &wayland_client::Connection,
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
                    if state.shm.is_some() {
                        state.cursor = Some(
                            super::cursor::Cursor::new(
                                state.compositor.as_ref().unwrap(),
                                conn.new_event_queue().handle(),
                                state.shm.as_ref().unwrap(),
                            )
                            .unwrap(),
                        );
                    }
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, 1, qh, ());
                    state.shm = Some(shm);
                    if state.compositor.is_some() {
                        state.cursor = Some(
                            super::cursor::Cursor::new(
                                state.compositor.as_ref().unwrap(),
                                conn.new_event_queue().handle(),
                                state.shm.as_ref().unwrap(),
                            )
                            .unwrap(),
                        );
                    }
                }
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                    state.seat = Some(seat);
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
                let keyboard = seat.get_keyboard(qh, ());
                this.keyboard = Some(keyboard);
            }
            if capabilities.contains(wl_seat::Capability::Pointer) {
                let pointer = seat.get_pointer(qh, ());
                if pointer.version() < 5 {
                    this.is_framed_pointer = false;
                }
                this.pointer = Some(pointer);
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
        pointer: &wl_pointer::WlPointer,
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

        fn check_cursor(
            this: &mut State,
            surface_x: u32,
            surface_y: u32,
            pointer: &wl_pointer::WlPointer,
        ) {
            let window = this.windows.get(&this.mouse_event.window).unwrap();
            let size = window.size.borrow();
            let top_rsz = (surface_y) < 5;
            let left_rsz = (surface_x) < 5;
            let right_rsz = (surface_x) > size.w - 5;
            let bottom_rsz = (surface_y) > size.h + super::window::TITLEBAR_HEIGHT as u32 - 5;
            let ns = if top_rsz {
                "n"
            } else if bottom_rsz {
                "s"
            } else {
                ""
            };
            let we = if left_rsz {
                "w"
            } else if right_rsz {
                "e"
            } else {
                ""
            };
            let ty = match (ns, we) {
                ("n", "") => CursorType::NResize,
                ("s", "") => CursorType::SResize,
                ("", "e") => CursorType::EResize,
                ("", "w") => CursorType::WResize,
                ("n", "e") => CursorType::NEResize,
                ("n", "w") => CursorType::NWResize,
                ("s", "e") => CursorType::SEResize,
                ("s", "w") => CursorType::SWResize,
                _ => CursorType::Arrow,
            };
            let cursor = this.cursor.as_mut().unwrap();
            if cursor.current_cursor != ty {
                let hot = cursor.set_cursor(ty).unwrap();
                pointer.set_cursor(cursor.last_serial, Some(&cursor.surface), hot.x, hot.y);
            }
        }

        fn handle_motion(
            this: &mut State,
            surface_x: u32,
            surface_y: u32,
            pointer: &wl_pointer::WlPointer,
        ) {
            check_cursor(this, surface_x, surface_y, pointer);

            if this.last_move.y < super::window::TITLEBAR_HEIGHT as i32 {
                let window = this.windows.get(&this.mouse_event.window).unwrap();
                window.titlebar(Offset::new(surface_x as _, surface_y as _), false);
                window.draw(None).unwrap();
            }
            this.last_move = Offset::new(surface_x as _, surface_y as _);
        }

        fn handle_press(this: &mut State, pointer: &wl_pointer::WlPointer, serial: u32) {
            let window = this.windows.get(&this.mouse_event.window).unwrap();
            let cursor = this.cursor.as_mut().unwrap().current_cursor;
            if !matches!(
                cursor,
                CursorType::Arrow
                    | CursorType::Pointer
                    | CursorType::Text
                    | CursorType::Move
                    | CursorType::Unknown
            ) {
                window.xdg_surface.get().unwrap().1.resize(
                    this.seat.as_ref().unwrap(),
                    serial,
                    match cursor {
                        CursorType::NResize => xdg_toplevel::ResizeEdge::Top,
                        CursorType::NEResize => xdg_toplevel::ResizeEdge::TopRight,
                        CursorType::EResize => xdg_toplevel::ResizeEdge::Right,
                        CursorType::SEResize => xdg_toplevel::ResizeEdge::BottomRight,
                        CursorType::SResize => xdg_toplevel::ResizeEdge::Bottom,
                        CursorType::SWResize => xdg_toplevel::ResizeEdge::BottomLeft,
                        CursorType::WResize => xdg_toplevel::ResizeEdge::Left,
                        CursorType::NWResize => xdg_toplevel::ResizeEdge::TopLeft,
                        _ => xdg_toplevel::ResizeEdge::None,
                    },
                );
            }
            if this.last_move.y < super::window::TITLEBAR_HEIGHT as i32 {
                window.titlebar(this.last_move, true);
                window.draw(None).unwrap();
                if this.last_move.x < window.size.borrow().w as i32 - 92 {
                    let cursor = this.cursor.as_mut().unwrap();
                    let hot = cursor.set_cursor(CursorType::Move).unwrap();
                    pointer.set_cursor(cursor.last_serial, Some(&cursor.surface), hot.x, hot.y);
                    window
                        .xdg_surface
                        .get()
                        .unwrap()
                        .1
                        ._move(this.seat.as_ref().unwrap(), serial);
                }
            }
        }

        fn handle_release(this: &mut State, pointer: &wl_pointer::WlPointer) {
            let window = this.windows.get(&this.mouse_event.window).unwrap();
            if this.last_move.x < window.size.borrow().w as i32 - 92 {
                let cursor = this.cursor.as_mut().unwrap();
                let hot = cursor.set_cursor(CursorType::Arrow).unwrap();
                pointer.set_cursor(cursor.last_serial, Some(&cursor.surface), hot.x, hot.y);
            }
            if this.last_move.y < super::window::TITLEBAR_HEIGHT as i32 - 4 && this.last_move.y > 4
            {
                let pos = this.last_move;
                let width = window.size.borrow().w;
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
        }

        match event {
            wl_pointer::Event::Enter {
                surface, serial, ..
            } => {
                this.mouse_event.window = this
                    .windows
                    .values()
                    .find(|w| *w.base_surface.get().unwrap() == surface)
                    .unwrap()
                    .id();
                this.cursor.as_mut().unwrap().last_serial = serial
            }
            wl_pointer::Event::Leave { .. } => {
                this.mouse_event.window = 0;
                this.buttons_held = [false; 6];
                this.cursor
                    .as_mut()
                    .unwrap()
                    .set_cursor(CursorType::Arrow)
                    .unwrap();
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                handle_motion(this, surface_x as u32, surface_y as u32, pointer);
                this.mouse_event.event = Event::Widget(WidgetEvent::Move(
                    surface_x as i32,
                    surface_y as i32 - TITLEBAR_HEIGHT as i32,
                ));
                if !this.is_framed_pointer {
                    this.events.push_back(this.mouse_event);
                }
            }
            wl_pointer::Event::Button {
                button,
                state,
                serial,
                ..
            } => {
                match state {
                    WEnum::Value(wl_pointer::ButtonState::Pressed) => {
                        this.buttons_held[button_from_ev(button) as usize] = true;
                        handle_press(this, pointer, serial);
                        this.mouse_event.event = Event::Widget(WidgetEvent::ButtonPress(
                            button_from_ev(button),
                            this.last_move.x,
                            this.last_move.y - super::window::TITLEBAR_HEIGHT as i32,
                        ))
                    }
                    WEnum::Value(wl_pointer::ButtonState::Released) => {
                        this.buttons_held[button_from_ev(button) as usize] = false;
                        handle_release(this, pointer);
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
            win.buffer.borrow_mut().take();
            let size = win.size.borrow();
            let buffer = win.shm.borrow().as_ref().unwrap().0.create_buffer(
                0,
                size.w as i32,
                size.h as i32 + super::window::TITLEBAR_HEIGHT as i32,
                size.w as i32 * 4,
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
            xdg_toplevel::Event::Configure {
                states,
                width,
                height,
            } => {
                let mut maximized = false;
                let mut fullscreen = false;
                let mut activated = false;
                for state in states.chunks_exact(4) {
                    if state[0] == 1 {
                        maximized = true;
                    } else if state[0] == 2 {
                        fullscreen = true;
                    } else if state[0] == 3 {
                        if width == 0 || height == 0 {
                            return;
                        }
                        let win = this.windows.get(win_id).unwrap();
                        win.resize(this, Size::new(width as u32, height as u32))
                            .unwrap();
                        let event = Event::Window(WindowEvent::Resize(
                            width as _,
                            height as u32 - TITLEBAR_HEIGHT as u32,
                        ));
                        if let Event::Window(WindowEvent::Resize(_, _)) = this
                            .events
                            .back()
                            .unwrap_or(&RawEvent {
                                window: 0,
                                event: Event::Unknown,
                            })
                            .event
                        {
                            this.events.pop_back();
                        }
                        this.events.push_back(RawEvent {
                            window: *win_id,
                            event,
                        });
                        return;
                    } else if state[0] == 4 {
                        activated = true;
                    }
                }
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
        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();
        let _registry = display.get_registry(&qh, ());

        let mut state = State {
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
            pointer: None,
            keyboard: None,
            seat: None,
            buttons_held: [false; 6],
            is_framed_pointer: true,
            last_move: Offset::default(),
            cursor: None,
        };

        event_queue.roundtrip(&mut state)?;

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
    pub(crate) fn destroy_window(&self, window_id: u64) {
        let mut state = self.state.borrow_mut();

        let window = state.windows.get(&window_id).unwrap();
        window.base_surface.get().unwrap().destroy();
        state.windows.remove(&window_id).unwrap();
        if state.windows.is_empty() {
            state.running = false;
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        self.keyboard.take().unwrap().release();
        self.pointer.take().unwrap().release();
        self.seat.take().unwrap().release();
        self.wm_base.take().unwrap().destroy();
        self.shm.take().unwrap().release();
    }
}
