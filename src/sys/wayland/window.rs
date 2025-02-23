use std::{
    cell::{OnceCell, RefCell},
    collections::VecDeque,
    num::NonZeroUsize,
    os::fd::{AsFd, OwnedFd},
    rc::Rc,
};

use lite_graphics::draw::Buffer;
use nix::{
    fcntl::OFlag,
    sys::{
        mman::{MapFlags, ProtFlags},
        stat::Mode,
    },
};

use wayland_client::protocol::{wl_buffer, wl_shm, wl_surface};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel};

use super::{App, Shm};

pub(crate) struct Window {
    pub(super) id: OnceCell<u64>,
    pub(super) xdg_surface: OnceCell<(xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel)>,
    pub(super) buffer: OnceCell<(wl_buffer::WlBuffer, Shm)>,
    pub(super) base_surface: OnceCell<wl_surface::WlSurface>,
    buffer_data: OnceCell<OwnedFd>,
    pub(super) events: RefCell<VecDeque<crate::event::Event>>,
}

impl Window {
    pub(crate) fn new(app: &Rc<App>) -> crate::Result<Rc<Self>> {
        let window = Rc::new(Window {
            id: OnceCell::new(),
            base_surface: OnceCell::new(),
            buffer: OnceCell::new(),
            buffer_data: OnceCell::new(),
            xdg_surface: OnceCell::new(),
            events: RefCell::new(VecDeque::new()),
        });

        let mut app_st = app.state.borrow_mut();

        app.event_queue.borrow_mut().roundtrip(&mut app_st)?;

        let id = app_st.windows.keys().max().unwrap_or(&0) + 1;
        let _ = window.id.set(id);

        let surface = app_st
            .compositor
            .as_ref()
            .unwrap()
            .create_surface(&app.qh, id);
        let _ = window.base_surface.set(surface);

        let xdg_surface = app_st.wm_base.as_ref().unwrap().get_xdg_surface(
            window.base_surface.get().unwrap(),
            &app.qh,
            id,
        );
        let toplevel = xdg_surface.get_toplevel(&app.qh, id);
        toplevel.set_title("Example window".into());

        window.base_surface.get().unwrap().commit();

        let _ = window.xdg_surface.set((xdg_surface, toplevel));

        let name = "lite_graphics_wayland";
        let file = nix::sys::mman::shm_open(
            name,
            OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR,
            Mode::S_IRUSR | Mode::S_IWUSR,
        )
        .unwrap();
        nix::unistd::ftruncate(file.as_fd(), 800 * 600 * 4).unwrap();
        unsafe {
            let ptr = nix::sys::mman::mmap(
                None,
                NonZeroUsize::new(800 * 600 * 4).unwrap(),
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                file.as_fd(),
                0,
            )
            .unwrap();
            let addr = std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, 800 * 600 * 4);
            for i in addr {
                *i = 255;
            }
            nix::sys::mman::munmap(ptr, 800 * 600 * 4).unwrap();
        };
        let pool =
            app_st
                .shm
                .as_ref()
                .unwrap()
                .create_pool(file.as_fd(), 800 * 600 * 4, &app.qh, id);
        let buffer =
            pool.create_buffer(0, 800, 600, 800 * 4, wl_shm::Format::Argb8888, &app.qh, id);
        let _ = window.buffer.set((buffer.clone(), Shm(name.to_string())));
        let _ = window.buffer_data.set(file);

        app_st.windows.insert(id, window.clone());

        Ok(window)
    }
    pub(crate) fn draw(&self, buf: Buffer) -> crate::Result<()> {
        let file = self.buffer_data.get().unwrap();

        unsafe {
            let ptr = nix::sys::mman::mmap(
                None,
                NonZeroUsize::new(800 * 600 * 4).unwrap(),
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                file.as_fd(),
                0,
            )?;
            let addr = std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, 800 * 600 * 4);
            let src = &**buf.data();

            for i in 0..800 * 600 {
                addr[i * 4] = src[i * 3];
                addr[i * 4 + 1] = src[i * 3 + 1];
                addr[i * 4 + 2] = src[i * 3 + 2];
            }
            nix::sys::mman::munmap(ptr, 800 * 600 * 4)?;
        };

        self.base_surface.get().unwrap().commit();

        Ok(())
    }
    pub(crate) fn get_events(&self) -> crate::Result<VecDeque<crate::event::RawEvent>> {
        let id = self.id();
        let mut events = self.events.borrow_mut();
        if events.is_empty() {
            events.push_front(crate::event::Event::Unknown);
        }
        Ok(events
            .drain(..)
            .map(|event| crate::event::RawEvent { window: id, event })
            .collect::<VecDeque<_>>())
    }
    pub(crate) fn id(&self) -> u64 {
        *self.id.get().unwrap()
    }
}
