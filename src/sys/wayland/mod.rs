mod app;
mod error;
mod window;

pub(crate) use app::App;
pub(crate) use window::Window;

pub(crate) fn has_wayland() -> bool {
    !std::env::var("WAYLAND_DISPLAY")
        .unwrap_or_default()
        .is_empty()
}

struct Shm(String);
impl Drop for Shm {
    fn drop(&mut self) {
        nix::sys::mman::shm_unlink(self.0.as_str()).unwrap();
    }
}
