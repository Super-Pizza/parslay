use std::{
    cell::{OnceCell, RefCell},
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

use wayland_client::protocol::{wl_buffer, wl_shm, wl_shm_pool, wl_surface};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel};

use super::{App, Shm};

pub(crate) struct Window {
    pub(super) qh: Rc<wayland_client::QueueHandle<super::app::State>>,
    pub(super) id: OnceCell<u64>,
    pub(super) xdg_surface: OnceCell<(xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel)>,
    pub(super) shm: OnceCell<(wl_shm_pool::WlShmPool, Shm)>,
    pub(super) buffer: RefCell<Option<wl_buffer::WlBuffer>>,
    pub(super) base_surface: OnceCell<wl_surface::WlSurface>,
    buffer_data: OnceCell<OwnedFd>,
}

impl Window {
    pub(crate) fn new(app: &Rc<App>) -> crate::Result<Rc<Self>> {
        let window = Rc::new(Window {
            qh: Rc::new(app.qh.clone()),
            id: OnceCell::new(),
            base_surface: OnceCell::new(),
            shm: OnceCell::new(),
            buffer: RefCell::new(None),
            buffer_data: OnceCell::new(),
            xdg_surface: OnceCell::new(),
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
        let _ = window.shm.set((pool, Shm(name.to_string())));
        let _ = window.buffer.borrow_mut().replace(buffer);
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

        let surface = self.base_surface.get().unwrap();
        surface.frame(&self.qh, self.id());
        surface.commit();

        Ok(())
    }
    pub(crate) fn id(&self) -> u64 {
        *self.id.get().unwrap()
    }
}
