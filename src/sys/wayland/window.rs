use std::{
    cell::{OnceCell, RefCell},
    fmt::Alignment,
    num::NonZeroUsize,
    os::fd::{AsFd as _, OwnedFd},
    rc::{Rc, Weak},
};

use lite_graphics::{Offset, Rect, Size, color::Rgba, draw::Buffer};
use nix::{
    fcntl::OFlag,
    sys::{
        mman::{MapFlags, ProtFlags},
        stat::Mode,
    },
};

use wayland_client::protocol::{wl_buffer, wl_shm, wl_shm_pool, wl_surface};
use wayland_protocols::xdg::shell::client::{xdg_surface, xdg_toplevel};

use crate::{sys, text::Text};

use super::{App, Shm};

pub(crate) struct Window {
    pub(super) app: Weak<super::app::App>,
    pub(super) xdg_surface: OnceCell<(xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel)>,
    pub(super) base_surface: OnceCell<wl_surface::WlSurface>,
    pub(super) buffer: RefCell<Option<wl_buffer::WlBuffer>>,
    pub(super) shm: OnceCell<(wl_shm_pool::WlShmPool, Shm)>,
    pub(super) qh: wayland_client::QueueHandle<super::app::State>,
    pub(super) id: OnceCell<u64>,
    buffer_data: OnceCell<OwnedFd>,
    titlebar_buf: RefCell<Buffer>,
    text: RefCell<Text>,
    pub(super) size: RefCell<Size>,
}

pub(super) const TITLEBAR_HEIGHT: usize = 32;
pub(super) const fn data_size(size: Size) -> usize {
    size.w as usize * (TITLEBAR_HEIGHT + size.h as usize) * 4
}

impl Window {
    pub(crate) fn new(app: &Rc<App>) -> crate::Result<Rc<Self>> {
        let window = Rc::new(Window {
            app: Rc::downgrade(app),
            qh: app.qh.clone(),
            id: OnceCell::new(),
            base_surface: OnceCell::new(),
            shm: OnceCell::new(),
            buffer: RefCell::new(None),
            buffer_data: OnceCell::new(),
            xdg_surface: OnceCell::new(),
            titlebar_buf: RefCell::new(Buffer::new(800, TITLEBAR_HEIGHT)),
            text: RefCell::new(Text::new("Hello, World!", 12.0)),
            size: RefCell::new(Size::new(800, 600)),
        });

        let size = *window.size.borrow();

        window.text.borrow_mut().set_font(sys::get_default_font()?);

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

        let name = env!("CARGO_PKG_NAME").to_string() + "_wayland";
        let _ = nix::sys::mman::shm_unlink(&*name);
        let file = nix::sys::mman::shm_open(
            &*name,
            OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR,
            Mode::S_IRUSR | Mode::S_IWUSR,
        )?;

        nix::unistd::ftruncate(file.as_fd(), data_size(size) as i64).unwrap();
        unsafe {
            let ptr = nix::sys::mman::mmap(
                None,
                NonZeroUsize::new(data_size(size)).unwrap(),
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                file.as_fd(),
                0,
            )
            .unwrap();
            let addr = std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, data_size(size));
            for i in addr {
                *i = 255;
            }
            nix::sys::mman::munmap(ptr, data_size(size)).unwrap();
        };
        let pool = app_st.shm.as_ref().unwrap().create_pool(
            file.as_fd(),
            data_size(size) as i32,
            &app.qh,
            id,
        );
        let buffer = pool.create_buffer(
            0,
            size.w as i32,
            size.h as i32 + TITLEBAR_HEIGHT as i32,
            size.w as i32 * 4,
            wl_shm::Format::Argb8888,
            &app.qh,
            id,
        );
        let _ = window.shm.set((pool, Shm(name.to_string())));
        let _ = window.buffer.borrow_mut().replace(buffer);
        let _ = window.buffer_data.set(file);

        window.titlebar(Offset::default(), false);

        app_st.windows.insert(id, window.clone());

        Ok(window)
    }
    pub(crate) fn titlebar(&self, pos: Offset, pressed: bool) {
        let titlebar_buf = self.titlebar_buf.borrow();
        let mut text = self.text.borrow_mut();
        let size = self.size.borrow();
        titlebar_buf.fill_rect(
            Rect::from((0, 0, size.w, TITLEBAR_HEIGHT as u32)),
            Rgba::hex("#333").unwrap(),
        );
        text.set_color(Rgba::WHITE);
        text.set_align(Alignment::Center);
        text.draw(
            &titlebar_buf,
            Rect::from((0, 8, size.w, TITLEBAR_HEIGHT as u32 - 8)),
            Rgba::hex("#333").unwrap(),
        )
        .unwrap_or_default();

        let color = if pressed {
            Rgba::hex("#777").unwrap()
        } else {
            Rgba::hex("#555").unwrap()
        };

        if pos.y > 4 && pos.y < 28 {
            if pos.x < size.w as i32 - 4 && pos.x > size.w as i32 - 28 {
                titlebar_buf.fill_circle_aa(Offset::new(size.w as i32 - 16, 16), 12, color);
            }
            if pos.x < size.w as i32 - 36 && pos.x > size.w as i32 - 60 {
                titlebar_buf.fill_circle_aa(Offset::new(size.w as i32 - 48, 16), 12, color);
            }
            if pos.x < size.w as i32 - 68 && pos.x > size.w as i32 - 92 {
                titlebar_buf.fill_circle_aa(Offset::new(size.w as i32 - 80, 16), 12, color);
            }
        }

        // Close
        titlebar_buf.line_aa(
            Offset::new(size.w as i32 - 20, 12),
            Offset::new(size.w as i32 - 12, 20),
            Rgba::WHITE,
        );
        titlebar_buf.line_aa(
            Offset::new(size.w as i32 - 20, 20),
            Offset::new(size.w as i32 - 12, 12),
            Rgba::WHITE,
        );

        // Minimize
        titlebar_buf.line_h(Offset::new(size.w as i32 - 52, 20), 8, Rgba::WHITE);

        // Maximize
        titlebar_buf.rect(Rect::from((size.w as i32 - 84, 12, 8, 8)), Rgba::WHITE);
    }
    pub(crate) fn draw(&self, buf: Option<Buffer>) -> crate::Result<()> {
        let file = self.buffer_data.get().unwrap();
        let size = *self.size.borrow();
        unsafe {
            let ptr = nix::sys::mman::mmap(
                None,
                NonZeroUsize::new(data_size(size)).unwrap(),
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                file.as_fd(),
                0,
            )?;
            let addr = std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, data_size(size));
            let titlebar = self.titlebar_buf.borrow();
            let titlebar_src = &**titlebar.data();

            for i in 0..size.w as usize * TITLEBAR_HEIGHT {
                addr[i * 4] = titlebar_src[i * 3];
                addr[i * 4 + 1] = titlebar_src[i * 3 + 1];
                addr[i * 4 + 2] = titlebar_src[i * 3 + 2];
            }

            if let Some(buf) = buf {
                let src = &**buf.data();
                for i in 0..(size.w * size.h) as usize {
                    let base = size.w as usize * TITLEBAR_HEIGHT;
                    addr[(base + i) * 4] = src[i * 3];
                    addr[(base + i) * 4 + 1] = src[i * 3 + 1];
                    addr[(base + i) * 4 + 2] = src[i * 3 + 2];
                }
            }
            nix::sys::mman::munmap(ptr, data_size(size))?;
        };

        let surface = self.base_surface.get().unwrap();
        surface.frame(&self.qh, self.id());
        surface.commit();

        Ok(())
    }
    pub(crate) fn id(&self) -> u64 {
        *self.id.get().unwrap()
    }
    pub(crate) fn set_cursor(&self, cursor_ty: crate::app::CursorType) {
        let app = self.app.upgrade().unwrap();
        let mut state = app.state.borrow_mut();
        let pointer = state.pointer.clone().unwrap();
        let cursor = state.cursor.as_mut().unwrap();
        if cursor.current_cursor == cursor_ty
            || cursor.current_cursor.to_string().contains("resize")
        {
            return;
        }
        let hot = cursor.set_cursor(cursor_ty).unwrap();
        pointer.set_cursor(cursor.last_serial, Some(&cursor.surface), hot.x, hot.y);
        self.base_surface.get().unwrap().commit();
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let xdg_surface = self.xdg_surface.take().unwrap();
        xdg_surface.1.destroy();
        xdg_surface.0.destroy();
        self.base_surface.take().unwrap().destroy();
        self.buffer.take().unwrap().destroy();
        self.shm.take().unwrap().0.destroy();
    }
}
