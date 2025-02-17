#![allow(clippy::collapsible_match)]
// TODO: Remove this when no longer needed.
#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use wayland_client::{
    delegate_noop,
    protocol::{
        wl_buffer, wl_compositor, wl_keyboard, wl_registry, wl_seat, wl_shm, wl_shm_pool,
        wl_surface,
    },
    Dispatch, WEnum,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

use super::Window;

pub(super) struct State {
    pub(super) running: bool,
    pub(super) windows: RefCell<HashMap<u64, Rc<Window>>>,
    pub(super) compositor: Option<wl_compositor::WlCompositor>,
    pub(super) shm: Option<wl_shm::WlShm>,
    pub(super) wm_base: Option<xdg_wm_base::XdgWmBase>,
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
        if let wl_keyboard::Event::Key { key, .. } = event {
            if key == 1 {
                this.running = false;
            }
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

            let windows = this.windows.borrow();
            let window = windows.get(window).unwrap();
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
        if let xdg_toplevel::Event::Close = event {
            this.windows.borrow_mut().remove(window);
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
            windows: RefCell::new(HashMap::new()),
            compositor: None,
            shm: None,
            wm_base: None,
        };

        Ok(Rc::new(Self {
            conn,
            state: RefCell::new(state),
            event_queue: RefCell::new(event_queue),
            qh,
            registry,
        }))
    }
    pub(crate) fn run(&self) -> crate::Result<()> {
        while self.state.borrow().running {
            self.event_queue
                .borrow_mut()
                .blocking_dispatch(&mut self.state.borrow_mut())?;
        }
        Ok(())
    }
}
