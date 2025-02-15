use std::fs;

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
mod linux;
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

pub(crate) fn get_font(name: Option<String>) -> crate::Result<(fs::File, u8)> {
    if cfg!(all(
        unix,
        not(any(
            target_os = "redox",
            target_family = "wasm",
            target_os = "android",
            target_os = "ios",
            target_os = "macos"
        ))
    )) {
        linux::get_font(name)
    } else {
        panic!("We support Linux only for now, sorry!")
    }
}
