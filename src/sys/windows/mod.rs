mod app;
mod error;
mod window;

use std::{
    ffi::OsString,
    fs,
    io::{self, Read as _},
    mem,
    os::windows::ffi::OsStringExt,
};

pub(crate) use app::App;
use heck::AsPascalCase;
pub(crate) use window::Window;
use windows::Win32::{
    Foundation::GetLastError,
    UI::WindowsAndMessaging::{SystemParametersInfoW, NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS},
};

pub(crate) fn get_font(name: Option<String>) -> crate::Result<(fs::File, u8)> {
    let get_metrics = || {
        let mut ncm = NONCLIENTMETRICSW {
            cbSize: mem::size_of::<NONCLIENTMETRICSW>() as u32,
            ..Default::default()
        };
        unsafe {
            SystemParametersInfoW(
                SPI_GETNONCLIENTMETRICS,
                mem::size_of::<NONCLIENTMETRICSW>() as u32,
                Some((&raw mut ncm).cast()),
                Default::default(),
            )
        }
        .ok()?;

        let face_name_raw = ncm
            .lfCaptionFont
            .lfFaceName
            .split(|c| *c == 0)
            .next()
            .unwrap();
        let face_name = OsString::from_wide(face_name_raw)
            .to_string_lossy()
            .into_owned();
        Some((face_name, ncm.lfMenuFont.lfWidth))
    };
    let face_name = name
        .zip(Some(12))
        .or_else(get_metrics)
        .ok_or_else(|| unsafe { GetLastError() })?;

    for font in fs::read_dir("C:/Windows/Fonts/")? {
        let Ok(font) = font else {
            continue;
        };
        let filename_os = font.file_name();
        let filename = filename_os.to_string_lossy();
        let name = filename.rsplit_once('.').unwrap_or((&filename, "")).0;
        if AsPascalCase(name).to_string().to_lowercase()
            == AsPascalCase(&face_name.0).to_string().to_lowercase()
        {
            return Ok((fs::File::open(font.path())?, face_name.1 as u8));
        }
    }
    Ok((fs::File::open("NUL")?, 0))
}

pub(crate) fn get_default_font() -> crate::Result<ab_glyph::FontArc> {
    let mut font = get_font(None)?.0;
    let mut buf = vec![];
    font.read_to_end(&mut buf)?;
    ab_glyph::FontArc::try_from_vec(buf)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Font Used").into())
}
