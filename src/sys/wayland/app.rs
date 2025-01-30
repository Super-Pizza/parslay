#![allow(clippy::collapsible_match)]
// TODO: Remove this when no longer needed.
#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, num::NonZeroUsize, os::fd::AsFd, rc::Rc};

use nix::{
    fcntl::OFlag,
    sys::{
        mman::{MapFlags, ProtFlags},
        stat::Mode,
    },
};
use raw_window_handle::RawWindowHandle;
use wayland_client::{
    delegate_noop,
    protocol::{
        wl_buffer, wl_compositor, wl_keyboard, wl_registry, wl_seat, wl_shm, wl_shm_pool,
        wl_surface,
    },
    Dispatch, WEnum,
};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel, xdg_wm_base};

use super::{Shm, Window};

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

pub(super) struct State {
    pub(super) running: bool,
    pub(super) base_surface: Option<wl_surface::WlSurface>,
    pub(super) buffer: Option<(wl_buffer::WlBuffer, Shm)>,
    pub(super) wm_base: Option<xdg_wm_base::XdgWmBase>,
    pub(super) xdg_surface: Option<(xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel)>,
    pub(super) configured: bool,
}

delegate_noop!(State: ignore wl_compositor::WlCompositor);
delegate_noop!(State: ignore wl_surface::WlSurface);
delegate_noop!(State: ignore wl_shm::WlShm);
delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
delegate_noop!(State: ignore wl_buffer::WlBuffer);

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
                    let surface = compositor.create_surface(qh, ());
                    state.base_surface = Some(surface);

                    if state.wm_base.is_some() && state.xdg_surface.is_none() {
                        state.init_xdg_surface(qh);
                    }
                }
                "wl_shm" => {
                    let shm = registry.bind::<wl_shm::WlShm, _, _>(name, 1, qh, ());
                    let name = "lite_graphics_wayland";
                    let file = nix::sys::mman::shm_open(
                        name,
                        OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR,
                        Mode::S_IRUSR | Mode::S_IWUSR,
                    )
                    .unwrap();
                    nix::unistd::ftruncate(file.as_fd(), 800 * 600 * 4).unwrap();
                    let addr = unsafe {
                        let ptr = nix::sys::mman::mmap(
                            None,
                            NonZeroUsize::new(800 * 600 * 4).unwrap(),
                            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                            MapFlags::MAP_SHARED,
                            file.as_fd(),
                            0,
                        )
                        .unwrap();
                        std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, 800 * 600 * 4)
                    };
                    for i in addr {
                        *i = 255;
                    }

                    let pool = shm.create_pool(file.as_fd(), 800 * 600 * 4, qh, ());
                    let buffer =
                        pool.create_buffer(0, 800, 600, 800 * 4, wl_shm::Format::Argb8888, qh, ());
                    state.buffer = Some((buffer.clone(), Shm(name.to_string())));
                    if state.configured {
                        let surface = state.base_surface.as_ref().unwrap();
                        surface.attach(Some(&buffer), 0, 0);
                        surface.commit();
                    }
                }
                "wl_seat" => {
                    registry.bind::<wl_seat::WlSeat, _, _>(name, 1, qh, ());
                }
                "xdg_wm_base" => {
                    let wm_base = registry.bind::<xdg_wm_base::XdgWmBase, _, _>(name, 1, qh, ());
                    state.wm_base = Some(wm_base);

                    if state.base_surface.is_some() && state.xdg_surface.is_none() {
                        state.init_xdg_surface(qh);
                    }
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

impl Dispatch<xdg_surface::XdgSurface, ()> for State {
    fn event(
        this: &mut Self,
        xdg_surface: &xdg_surface::XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            xdg_surface.ack_configure(serial);
            this.configured = true;

            let surface = this.base_surface.as_ref().unwrap();
            if let Some((ref buffer, _)) = this.buffer {
                surface.attach(Some(buffer), 0, 0);
                surface.commit();
            }
        }
    }
}

impl Dispatch<xdg_toplevel::XdgToplevel, ()> for State {
    fn event(
        this: &mut Self,
        _: &xdg_toplevel::XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        if let xdg_toplevel::Event::Close = event {
            this.running = false
        }
    }
}

impl State {
    fn init_xdg_surface(&mut self, qh: &wayland_client::QueueHandle<Self>) {
        let wm_base = self.wm_base.as_ref().unwrap();
        let base_surface = self.base_surface.as_ref().unwrap();

        let xdg_surface = wm_base.get_xdg_surface(base_surface, qh, ());
        let toplevel = xdg_surface.get_toplevel(qh, ());
        toplevel.set_title("Example window".into());

        base_surface.commit();

        self.xdg_surface = Some((xdg_surface, toplevel));
    }
}
