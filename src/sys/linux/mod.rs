use std::{
    fs,
    io::{self, Read},
    process::Command,
};

use crate::event::Key;

pub(crate) fn get_font(name: Option<String>) -> crate::Result<(fs::File, u8)> {
    let default_font_vec = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "font-name"])
        .output()?
        .stdout;
    let mut default_font = String::from_utf8_lossy(&default_font_vec).into_owned();
    default_font = default_font.trim().trim_matches('\'').to_owned();
    let split = default_font.rfind(' ').unwrap();
    let size = default_font.split_off(split).trim().parse::<u8>().unwrap();
    let path = fontconfig::Fontconfig::new()
        .unwrap()
        .find(&name.unwrap_or(default_font), Some("Regular"))
        .unwrap()
        .path;
    fs::File::open(path).map_err(Into::into).map(|f| (f, size))
}

pub(crate) fn get_default_font() -> crate::Result<ab_glyph::FontArc> {
    let mut font = get_font(None)?.0;
    let mut buf = vec![];
    font.read_to_end(&mut buf)?;
    ab_glyph::FontArc::try_from_vec(buf)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid Font Used").into())
}

pub(crate) fn key_from_xkb(sym: u32) -> Key {
    match sym {
        // ASCII Keys
        0x20 => Key::Space,
        0x21 => Key::Exclam,
        0x22 => Key::Quote,
        0x23 => Key::NumberSign,
        0x24 => Key::Dollar,
        0x25 => Key::Percent,
        0x26 => Key::Amp,
        0x27 => Key::Apostrophe,
        0x28 => Key::LeftParen,
        0x29 => Key::RightParen,
        0x2a => Key::Asterisk,
        0x2b => Key::Plus,
        0x2c => Key::Comma,
        0x2d => Key::Minus,
        0x2e => Key::Period,
        0x2f => Key::Slash,
        0x30 => Key::Key0,
        0x31 => Key::Key1,
        0x32 => Key::Key2,
        0x33 => Key::Key3,
        0x34 => Key::Key4,
        0x35 => Key::Key5,
        0x36 => Key::Key6,
        0x37 => Key::Key7,
        0x38 => Key::Key8,
        0x39 => Key::Key9,
        0x3a => Key::Colon,
        0x3b => Key::Semicolon,
        0x3c => Key::Less,
        0x3d => Key::Equals,
        0x3e => Key::Greater,
        0x3f => Key::Question,
        0x40 => Key::At,
        0x41 => Key::A,
        0x42 => Key::B,
        0x43 => Key::C,
        0x44 => Key::D,
        0x45 => Key::E,
        0x46 => Key::F,
        0x47 => Key::G,
        0x48 => Key::H,
        0x49 => Key::I,
        0x4a => Key::J,
        0x4b => Key::K,
        0x4c => Key::L,
        0x4d => Key::M,
        0x4e => Key::N,
        0x4f => Key::O,
        0x50 => Key::P,
        0x51 => Key::Q,
        0x52 => Key::R,
        0x53 => Key::S,
        0x54 => Key::T,
        0x55 => Key::U,
        0x56 => Key::V,
        0x57 => Key::W,
        0x58 => Key::X,
        0x59 => Key::Y,
        0x5a => Key::Z,
        0x5b => Key::LeftBracket,
        0x5c => Key::Backslash,
        0x5d => Key::RightBracket,
        0x5e => Key::Circum,
        0x5f => Key::Underscore,
        0x60 => Key::Backtick,
        0x61 => Key::a,
        0x62 => Key::b,
        0x63 => Key::c,
        0x64 => Key::d,
        0x65 => Key::e,
        0x66 => Key::f,
        0x67 => Key::g,
        0x68 => Key::h,
        0x69 => Key::i,
        0x6a => Key::j,
        0x6b => Key::k,
        0x6c => Key::l,
        0x6d => Key::m,
        0x6e => Key::n,
        0x6f => Key::o,
        0x70 => Key::p,
        0x71 => Key::q,
        0x72 => Key::r,
        0x73 => Key::s,
        0x74 => Key::t,
        0x75 => Key::u,
        0x76 => Key::v,
        0x77 => Key::w,
        0x78 => Key::x,
        0x79 => Key::y,
        0x7a => Key::z,
        0x7b => Key::LeftBrace,
        0x7c => Key::Bar,
        0x7d => Key::RightBrace,
        0x7e => Key::Tilde,
        // Others
        0xff08 => Key::Backspace,
        0xff09 => Key::Tab,
        0xff0d => Key::Enter,
        0xff13 => Key::Pause,
        0xff1b => Key::Escape,
        0xffff => Key::Delete,
        0xff51 => Key::ArrowLeft,
        0xff52 => Key::ArrowUp,
        0xff53 => Key::ArrowRight,
        0xff54 => Key::ArrowDown,
        0xff55 => Key::PgUp,
        0xff56 => Key::PgDn,
        0xff57 => Key::End,
        0xff58 => Key::Home,
        0xff63 => Key::Insert,
        0xff67 => Key::Menu,
        0xff6b => Key::Break,
        0xffaa => Key::NumMul,
        0xffab => Key::NumAdd,
        0xffad => Key::NumSub,
        0xffae => Key::NumDecimal,
        0xffaf => Key::NumDiv,
        0xffb0 => Key::Num0,
        0xffb1 => Key::Num1,
        0xffb2 => Key::Num2,
        0xffb3 => Key::Num3,
        0xffb4 => Key::Num4,
        0xffb5 => Key::Num5,
        0xffb6 => Key::Num6,
        0xffb7 => Key::Num7,
        0xffb8 => Key::Num8,
        0xffb9 => Key::Num9,
        0xffbe => Key::F1,
        0xffbf => Key::F2,
        0xffc0 => Key::F3,
        0xffc1 => Key::F4,
        0xffc2 => Key::F5,
        0xffc3 => Key::F6,
        0xffc4 => Key::F7,
        0xffc5 => Key::F8,
        0xffc6 => Key::F9,
        0xffc7 => Key::F10,
        0xffc8 => Key::F11,
        0xffc9 => Key::F12,
        _ => Key::Other,
    }
}
