use std::io;

use windows::{core::HRESULT, Win32::Foundation::WIN32_ERROR};

impl From<WIN32_ERROR> for crate::Error {
    fn from(value: WIN32_ERROR) -> Self {
        Self::Io(io::Error::from_raw_os_error(value.0 as i32))
    }
}

impl From<HRESULT> for crate::Error {
    fn from(value: HRESULT) -> Self {
        if value.0 as u32 & 0xFFFF_0000u32 == 0x8007_0000u32 {
            Self::Io(io::Error::from_raw_os_error(value.0 & 0xFFFF))
        } else {
            Self::Io(io::Error::other(format!("HRESULT: {}", value.0)))
        }
    }
}

impl From<windows::core::Error> for crate::Error {
    fn from(value: windows::core::Error) -> Self {
        value.code().into()
    }
}
