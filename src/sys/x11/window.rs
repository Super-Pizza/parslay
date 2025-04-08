use std::{
    borrow::Cow,
    cell::RefCell,
    rc::{Rc, Weak},
};

use lite_graphics::draw::Buffer;
use x11rb::{
    connection::Connection,
    image::{BitsPerPixel, Image, ImageOrder, ScanlinePad},
    protocol::xproto::{
        AtomEnum, ConnectionExt as _, CreateGCAux, CreateWindowAux, EventMask, PropMode,
        WindowClass,
    },
    wrapper::ConnectionExt as _,
    COPY_DEPTH_FROM_PARENT, COPY_FROM_PARENT,
};

use crate::event::WindowState;

use super::App;
pub(crate) struct Window {
    app: Weak<App>,
    pub(super) window: u32,
    pub(super) gc: u32,
    pub(super) state: RefCell<WindowState>,
}

impl Window {
    pub(crate) fn new(app: &Rc<App>) -> crate::Result<Rc<Self>> {
        let conn = &app.conn;
        let screen = &app.screen;
        let win_id = conn.generate_id()?;
        let gc = conn.generate_id()?;
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
                        | EventMask::KEY_RELEASE
                        | EventMask::BUTTON_PRESS
                        | EventMask::BUTTON_RELEASE
                        | EventMask::POINTER_MOTION
                )),
        )?;

        conn.create_gc(gc, win_id, &CreateGCAux::new())?;

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

        let win = Rc::new(Self {
            app: Rc::downgrade(app),
            window: win_id,
            gc,
            state: RefCell::new(WindowState::Suspended),
        });
        app.windows.borrow_mut().push(win.clone());
        Ok(win)
    }
    pub(crate) fn draw(&self, buf: Buffer) -> crate::Result<()> {
        let data = &**buf.data();
        let img = Image::new(
            buf.size().w as _,
            buf.size().h as _,
            ScanlinePad::Pad8,
            24,
            BitsPerPixel::B24,
            ImageOrder::MsbFirst,
            Cow::Borrowed(data),
        )
        .unwrap();
        let app = self.app.upgrade().unwrap();
        let img = img.native(app.conn.setup()).unwrap();
        img.put(&app.conn, self.window, self.gc, 0, 0).unwrap();
        app.conn.flush().unwrap();
        Ok(())
    }
    #[allow(unused)]
    pub(crate) fn id(&self) -> u64 {
        self.window as _
    }
}
