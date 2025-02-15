use std::{fs, process::Command};

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
