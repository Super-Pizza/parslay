use std::{
    collections::HashMap,
    fs, io,
    num::NonZeroUsize,
    os::fd::{AsFd as _, OwnedFd},
    path::{Path, PathBuf},
    process::Command,
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
    delegate_noop,
    protocol::{
        wl_buffer, wl_callback, wl_compositor, wl_registry, wl_shm, wl_shm_pool, wl_surface,
    },
    QueueHandle,
};

pub(super) struct Cursor {
    pub(super) last_serial: u32,
    base_path: PathBuf,
    qh: QueueHandle<Self>,
    pub(super) surface: wl_surface::WlSurface,
    pool: wl_shm_pool::WlShmPool,
    buffers: HashMap<String, (usize, wl_buffer::WlBuffer, Offset)>,
    buffer_data: OwnedFd,
    current_cursor: String,
}

const WIDTH: usize = 24;
const SIZE: usize = WIDTH * WIDTH * 4;
const NUM_CURSORS: usize = POINTERS.len();
const POINTERS: &[&str] = &[
    "default",
    "pointer",
    "n-resize",
    "s-resize",
    "e-resize",
    "w-resize",
    "ne-resize",
    "nw-resize",
    "se-resize",
    "sw-resize",
];

impl Cursor {
    fn get_path() -> crate::Result<PathBuf> {
        let default_cursor_vec = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "cursor-theme"])
            .output()?
            .stdout;
        let mut default_cursor = String::from_utf8_lossy(&default_cursor_vec).into_owned();
        default_cursor = default_cursor.trim().trim_matches('\'').to_owned();

        let mut default_path = Path::new("/usr/share/icons").join(default_cursor);
        if !default_path.join("cursors").exists() {
            // Backup for 'default'
            let index = fs::read(default_path.join("index.theme"))?;
            let index_string = String::from_utf8_lossy(&index);
            let line = index_string
                .split('\n')
                .find(|line| line.starts_with("Inherits"))
                .ok_or(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Cannot find inherit",
                ))?;
            let real = line.split_once('=').unwrap().1.trim();
            default_path = default_path.parent().unwrap().join(real);
        }
        assert!(default_path.join("cursors").exists());
        Ok(default_path.join("cursors"))
    }
    fn load_cursor(&mut self, name: String, index: usize) -> crate::Result<Offset> {
        let data = fs::read(self.base_path.join(name))?;

        if &data[0..4] != b"Xcur" {
            return Err(crate::Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not an X Cursor!",
            )));
        }

        let ntoc = u32_(&data[12..16]) as usize;
        for toc_entry in 0..ntoc {
            let typ = u32_(&data[16 + toc_entry * 12..20 + toc_entry * 12]);
            let pos = u32_(&data[24 + toc_entry * 12..28 + toc_entry * 12]) as usize;
            if typ != 0xfffd0002 {
                continue;
            }

            let width = u32_(&data[pos + 16..pos + 20]);
            let height = u32_(&data[pos + 20..pos + 24]);
            let xhot = u32_(&data[pos + 24..pos + 28]);
            let yhot = u32_(&data[pos + 28..pos + 32]);
            let pixels = &data[pos + 36..pos + 36 + (width * height * 4) as usize];
            if height as usize != WIDTH || width as usize != WIDTH {
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
                for i in 0..(width * height) as usize {
                    addr[i * 4] = pixels[i * 4];
                    addr[i * 4 + 1] = pixels[i * 4 + 1];
                    addr[i * 4 + 2] = pixels[i * 4 + 2];
                    addr[i * 4 + 3] = pixels[i * 4 + 3];
                }
                nix::sys::mman::munmap(ptr, SIZE).unwrap();
            };
            return Ok(Offset::new(xhot as i32, yhot as i32));
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
        let base_path = Self::get_path()?;
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
            base_path,
            qh,
            surface,
            pool,
            buffers,
            buffer_data: file,
            current_cursor: "default".to_owned(),
        };

        for (i, &name) in POINTERS.iter().enumerate() {
            let hot = this.load_cursor(name.to_owned(), i)?;
            let buffer = this.pool.create_buffer(
                i as i32 * page_size.max(SIZE as i64) as i32,
                WIDTH as i32,
                WIDTH as i32,
                WIDTH as i32 * 4,
                wl_shm::Format::Argb8888,
                &this.qh,
                (),
            );
            this.buffers.insert(name.to_owned(), (i, buffer, hot));
        }

        this.set_cursor("default")?;

        Ok(this)
    }
    pub(super) fn set_cursor(&mut self, name: &str) -> crate::Result<Offset> {
        let buffer = self.buffers.get(name).ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "The requested cursor isn't loaded",
        ))?;
        if name == self.current_cursor {
            // Small optimization
            return Ok(buffer.2);
        }

        self.current_cursor = name.to_owned();
        self.surface.frame(&self.qh, ());
        self.surface.attach(Some(&buffer.1), buffer.2.x, buffer.2.y);
        self.surface.damage(0, 0, i32::MAX, i32::MAX);
        self.surface.commit();
        Ok(buffer.2)
    }
}

fn u32_(arr: &[u8]) -> u32 {
    u32::from_le_bytes([arr[0], arr[1], arr[2], arr[3]])
}

delegate_noop!(Cursor: ignore wl_registry::WlRegistry);
delegate_noop!(Cursor: ignore wl_surface::WlSurface);
delegate_noop!(Cursor: ignore wl_shm_pool::WlShmPool);
delegate_noop!(Cursor: ignore wl_buffer::WlBuffer);
delegate_noop!(Cursor: ignore wl_callback::WlCallback);
