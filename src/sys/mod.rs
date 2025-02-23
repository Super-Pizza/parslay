use std::fs;

macro_rules! platform {
    (linux => $(mod $mod:ident;)*) => {
        $(#[cfg(all(
            unix,
            not(any(
                target_os = "redox",
                target_family = "wasm",
                target_os = "android",
                target_os = "ios",
                target_os = "macos"
            ))
        ))]
        mod $mod;)*
    };
    (linux => $($line:expr),*) => {
        $(#[cfg(all(
            unix,
            not(any(
                target_os = "redox",
                target_family = "wasm",
                target_os = "android",
                target_os = "ios",
                target_os = "macos"
            ))
        ))]
        $line),*
    };
    ($vis:vis enum $name:ident {linux => $($line:ident $block:tt),*$(,)? }) => {
        $vis enum $name {
            $(#[cfg(all(
                unix,
                not(any(
                    target_os = "redox",
                    target_family = "wasm",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "macos"
                ))
            ))]
            $line $block),*
        }
    };
    (match $name:ident { $($pat:pat if linux => $block:expr),*$(,)? }) => {
        match $name {
            $(#[cfg(all(
                unix,
                not(any(
                    target_os = "redox",
                    target_family = "wasm",
                    target_os = "android",
                    target_os = "ios",
                    target_os = "macos"
                ))
            ))]$pat=> $block),*
        }
    }
}

platform!(
    linux =>
        mod linux;
        mod x11;
        mod wayland;

);

pub(crate) mod app;
pub(crate) mod window;

pub(crate) fn get_font(name: Option<String>) -> crate::Result<(fs::File, u8)> {
    platform!(linux => linux::get_font(name))
}
