use std::rc::Rc;

use raw_window_handle::{RawWindowHandle, XlibWindowHandle};
use x11rb::{
    connection::Connection,
    protocol::xproto::{
        AtomEnum, ConnectionExt as _, CreateWindowAux, EventMask, PropMode, WindowClass,
    },
    wrapper::ConnectionExt as _,
    COPY_DEPTH_FROM_PARENT, COPY_FROM_PARENT,
};

use super::App;
pub(crate) struct Window(pub(super) u32);

impl Window {
    pub(crate) fn new(app: &Rc<App>) -> crate::Result<Rc<Self>> {
        let conn = &app.conn;
        let screen = &app.screen;
        let win_id = conn.generate_id()?;
        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            win_id,
            screen.root,
            0,
            0,
            800,
            600,
            0,
            WindowClass::COPY_FROM_PARENT,
            COPY_FROM_PARENT,
            &CreateWindowAux::new()
                .background_pixel(Some(0xffffffff))
                .event_mask(Some(
                    EventMask::EXPOSURE
                        | EventMask::STRUCTURE_NOTIFY
                        | EventMask::KEY_PRESS
                        | EventMask::KEY_RELEASE,
                )),
        )?;

        let title = "Simple Window";
        conn.change_property8(
            PropMode::REPLACE,
            win_id,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            title.as_bytes(),
        )?;
        conn.change_property8(
            PropMode::REPLACE,
            win_id,
            app.atoms._NET_WM_NAME,
            app.atoms.UTF8_STRING,
            title.as_bytes(),
        )?;

        conn.change_property32(
            PropMode::REPLACE,
            win_id,
            app.atoms.WM_PROTOCOLS,
            AtomEnum::ATOM,
            &[app.atoms.WM_DELETE_WINDOW],
        )?;
        conn.change_property8(
            PropMode::REPLACE,
            win_id,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            b"simple_gui window\0",
        )?;

        conn.map_window(win_id)?;
        conn.flush()?;

        let win = Rc::new(Self(win_id));
        app.windows.borrow_mut().push(win.clone());
        Ok(win)
    }
    #[allow(unused)]
    pub(crate) fn id(&self) -> RawWindowHandle {
        RawWindowHandle::Xlib(XlibWindowHandle::new(self.0 as _))
    }
}
