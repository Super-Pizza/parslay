use std::fs;

macro_rules! platform {
    (linux => $($lvis:vis mod $lmod:ident);*, windows => $wvis:vis mod $wmod:ident $(,)?) => {
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
        $lvis mod $lmod;)*
        #[cfg(target_os = "windows")] $wvis mod $wmod;
    };
    (linux => $lline:expr; windows => $wline:expr $(;)?) => {
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
        return $lline;
        #[cfg(target_os = "windows")] return $wline;
    };
    ($vis:vis enum $name:ident {linux => $($lline:ident $lblock:tt),*; windows => $wline:ident $wblock:tt $(,)? }) => {
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
            $lline $lblock,)*
            #[cfg(target_os = "windows")]
            $wline $wblock

        }
    };
    (match $name:ident { $($pat:pat if $platform:ident => $block:expr,)* }) => {
        platform!(@match $name {
            $($platform: $pat=> $block,)*
        })
    };
    (@match $name:ident { $(linux: $lpat:pat => $lblock:expr,)* windows: $wpat:pat => $wblock:expr $(,)? }) => {
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
            ))]
            $lpat => $lblock,)*
            #[cfg(target_os = "windows")]
            $wpat => $wblock
        }
    }
}

platform!(
    linux =>
        pub(crate) mod linux;
        mod x11;
        mod wayland,
    windows => pub(crate) mod windows

);

pub(crate) mod app;
pub(crate) mod window;

pub(crate) fn get_font(name: Option<String>) -> crate::Result<(fs::File, u8)> {
    platform!(
        linux => linux::get_font(name);
        windows => windows::get_font(name);
    );
}

pub(crate) fn get_default_font() -> crate::Result<ab_glyph::FontArc> {
    platform!(
        linux => linux::get_default_font();
        windows => windows::get_default_font();
    );
}
