use std::{
    collections::HashMap,
    fs, io,
    num::NonZeroUsize,
    os::fd::{AsFd as _, OwnedFd},
};

use lite_graphics::Offset;
use nix::{
    fcntl::OFlag,
    sys::{
        mman::{MapFlags, ProtFlags},
        stat::Mode,
    },
};
use wayland_client::{
    QueueHandle, delegate_noop,
    protocol::{
        wl_buffer, wl_callback, wl_compositor, wl_registry, wl_shm, wl_shm_pool, wl_surface,
    },
};

use crate::app::CursorType;

use super::Shm;

pub(super) struct Cursor {
    pub(super) last_serial: u32,
    theme: xcursor::CursorTheme,
    qh: QueueHandle<Self>,
    pub(super) surface: wl_surface::WlSurface,
    pool: (wl_shm_pool::WlShmPool, Shm),
    buffers: HashMap<CursorType, (usize, wl_buffer::WlBuffer, Offset)>,
    buffer_data: OwnedFd,
    pub(super) current_cursor: CursorType,
}

const WIDTH: usize = 24;
const SIZE: usize = WIDTH * WIDTH * 4;
const NUM_CURSORS: usize = POINTERS.len();
const POINTERS: &[CursorType] = &[
    CursorType::Arrow,
    CursorType::Pointer,
    CursorType::Text,
    CursorType::NResize,
    CursorType::SResize,
    CursorType::EResize,
    CursorType::WResize,
    CursorType::NEResize,
    CursorType::NWResize,
    CursorType::SEResize,
    CursorType::SWResize,
];

impl Cursor {
    fn load_cursor(&mut self, name: String, index: usize) -> crate::Result<Offset> {
        let data = fs::read(self.theme.load_icon(&name).unwrap())?;

        let cursors = xcursor::parser::parse_xcursor(&data).ok_or(crate::Error::Io(
            io::Error::new(io::ErrorKind::InvalidData, "Cursor file is invalid"),
        ))?;

        for cursor in cursors {
            if cursor.size != 24 {
                continue;
            }

            unsafe {
                let page_size = nix::unistd::sysconf(nix::unistd::SysconfVar::PAGE_SIZE)?.unwrap();
                let ptr = nix::sys::mman::mmap(
                    None,
                    NonZeroUsize::new(SIZE).unwrap(),
                    ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                    MapFlags::MAP_SHARED,
                    self.buffer_data.as_fd(),
                    index as i64 * page_size.max(SIZE as i64),
                )
                .unwrap();
                let addr = std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut u8, SIZE);
                for i in 0..(cursor.width * cursor.height) as usize {
                    addr[i * 4] = cursor.pixels_rgba[i * 4];
                    addr[i * 4 + 1] = cursor.pixels_rgba[i * 4 + 1];
                    addr[i * 4 + 2] = cursor.pixels_rgba[i * 4 + 2];
                    addr[i * 4 + 3] = cursor.pixels_rgba[i * 4 + 3];
                }
                nix::sys::mman::munmap(ptr, SIZE).unwrap();
            };
            return Ok(Offset::new(cursor.xhot as i32, cursor.yhot as i32));
        }
        Err(crate::Error::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "No matching cursor found",
        )))
    }
    pub(super) fn new(
        comp: &wl_compositor::WlCompositor,
        qh: QueueHandle<Cursor>,
        shm: &wl_shm::WlShm,
    ) -> crate::Result<Self> {
        let theme = xcursor::CursorTheme::load("");
        let surface = comp.create_surface(&qh, ());

        let name = env!("CARGO_PKG_NAME").to_string() + "_cursor";
        let _ = nix::sys::mman::shm_unlink(&*name);
        let file = nix::sys::mman::shm_open(
            &*name,
            OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR,
            Mode::S_IRUSR | Mode::S_IWUSR,
        )?;

        let page_size = nix::unistd::sysconf(nix::unistd::SysconfVar::PAGE_SIZE)?.unwrap();

        nix::unistd::ftruncate(
            file.as_fd(),
            page_size.max(SIZE as i64) * NUM_CURSORS as i64,
        )
        .unwrap();

        let pool = shm.create_pool(
            file.as_fd(),
            page_size.max(SIZE as i64) as i32 * NUM_CURSORS as i32,
            &qh,
            (),
        );

        let buffers = HashMap::new();

        let mut this = Self {
            last_serial: 0,
            theme,
            qh,
            surface,
            pool: (pool, Shm(name.to_string())),
            buffers,
            buffer_data: file,
            current_cursor: CursorType::Unknown,
        };

        for (i, &ty) in POINTERS.iter().enumerate() {
            let hot = this.load_cursor(ty.to_string(), i)?;
            let buffer = this.pool.0.create_buffer(
                i as i32 * page_size.max(SIZE as i64) as i32,
                WIDTH as i32,
                WIDTH as i32,
                WIDTH as i32 * 4,
                wl_shm::Format::Argb8888,
                &this.qh,
                (),
            );
            this.buffers.insert(ty, (i, buffer, hot));
        }

        this.set_cursor(CursorType::Arrow)?;

        Ok(this)
    }
    pub(super) fn set_cursor(&mut self, ty: crate::app::CursorType) -> crate::Result<Offset> {
        let buffer = self.buffers.get(&ty).ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "The requested cursor isn't loaded",
        ))?;

        self.current_cursor = ty;
        self.surface.frame(&self.qh, ());
        self.surface.attach(Some(&buffer.1), buffer.2.x, buffer.2.y);
        self.surface.damage(0, 0, i32::MAX, i32::MAX);
        self.surface.commit();
        Ok(buffer.2)
    }
}

delegate_noop!(Cursor: ignore wl_registry::WlRegistry);
delegate_noop!(Cursor: ignore wl_surface::WlSurface);
delegate_noop!(Cursor: ignore wl_shm_pool::WlShmPool);
delegate_noop!(Cursor: ignore wl_buffer::WlBuffer);
delegate_noop!(Cursor: ignore wl_callback::WlCallback);

impl Drop for Cursor {
    fn drop(&mut self) {
        self.surface.destroy();
        for buffer in self.buffers.values() {
            buffer.1.destroy();
        }
        self.pool.0.destroy();
    }
}
