use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    #[cfg(all(
        unix,
        not(any(
            target_os = "redox",
            target_family = "wasm",
            target_os = "android",
            target_os = "ios",
            target_os = "macos"
        ))
    ))]
    X11(x11rb::errors::ReplyError),
    #[cfg(all(
        unix,
        not(any(
            target_os = "redox",
            target_family = "wasm",
            target_os = "android",
            target_os = "ios",
            target_os = "macos"
        ))
    ))]
    WaylandError(wayland_client::backend::protocol::ProtocolError),
    #[cfg(all(
        unix,
        not(any(
            target_os = "redox",
            target_family = "wasm",
            target_os = "android",
            target_os = "ios",
            target_os = "macos"
        ))
    ))]
    WaylandConnect(wayland_client::ConnectError),
    #[cfg(target_os = "windows")]
    Windows(),
}
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(all(
    unix,
    not(any(
        target_os = "redox",
        target_family = "wasm",
        target_os = "android",
        target_os = "ios",
        target_os = "macos"
    ))
))]
impl From<nix::Error> for Error {
    fn from(value: nix::Error) -> Self {
        Self::Io(io::Error::from(value))
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
