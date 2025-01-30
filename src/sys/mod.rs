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
mod wayland;
#[cfg(target_os = "windows")]
mod windows;
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
mod x11;

pub(crate) mod app;
pub(crate) mod window;