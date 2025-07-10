use std::{borrow::Cow, collections::HashMap, fs, io, rc::Rc};

use lite_graphics::Offset;
use x11rb::{
    connection::Connection as _,
    image::{BitsPerPixel, Image, ImageOrder, ScanlinePad},
    protocol::xproto::{
        ConnectionExt as _, CreateGCAux, Cursor as XCursor, GcontextWrapper, PixmapWrapper,
    },
    rust_connection::RustConnection,
};

use crate::app::CursorType;

pub(super) struct Cursor {
    gc: GcontextWrapper<Rc<RustConnection>>,
    theme: xcursor::CursorTheme,
    buffers: HashMap<CursorType, (XCursor, Offset)>,
    pub(super) current_cursor: CursorType,
}

const WIDTH: u32 = 24;
const POINTERS: &[CursorType] = &[CursorType::Arrow, CursorType::Pointer, CursorType::Text];

impl Cursor {
    fn load_cursor(
        &mut self,
        conn: Rc<RustConnection>,
        root: u32,
        name: String,
    ) -> crate::Result<(XCursor, Offset)> {
        let data = fs::read(self.theme.load_icon(&name).unwrap())?;

        let cursors = xcursor::parser::parse_xcursor(&data).ok_or(crate::Error::Io(
            io::Error::new(io::ErrorKind::InvalidData, "Cursor file is invalid"),
        ))?;

        for cursor in cursors {
            if cursor.size != WIDTH {
                continue;
            }

            let source = PixmapWrapper::create_pixmap(
                conn.clone(),
                1,
                root,
                cursor.width as u16,
                cursor.height as u16,
            )?;
            let mask = PixmapWrapper::create_pixmap(
                conn.clone(),
                1,
                root,
                cursor.width as u16,
                cursor.height as u16,
            )?;

            let data = &cursor
                .pixels_rgba
                .iter()
                .map(|i| i / 128)
                .collect::<Vec<u8>>();

            let img = Image::new(
                cursor.width as _,
                cursor.height as _,
                ScanlinePad::Pad8,
                1,
                BitsPerPixel::B32,
                ImageOrder::LsbFirst,
                Cow::Borrowed(data),
            )
            .unwrap();
            img.put(&conn, source.pixmap(), self.gc.gcontext(), 0, 0)?;

            let img = Image::new(
                cursor.width as _,
                cursor.height as _,
                ScanlinePad::Pad8,
                1,
                BitsPerPixel::B32,
                ImageOrder::MsbFirst,
                Cow::Borrowed(data),
            )
            .unwrap();
            img.put(&conn, mask.pixmap(), self.gc.gcontext(), 0, 0)?;

            let cur = conn.generate_id()?;
            conn.create_cursor(
                cur,
                source.pixmap(),
                mask.pixmap(),
                u16::MAX,
                u16::MAX,
                u16::MAX,
                0,
                0,
                0,
                cursor.xhot as u16,
                cursor.yhot as u16,
            )?;

            conn.flush()?;

            let hot = Offset::new(cursor.xhot as i32, cursor.yhot as i32);

            return Ok((cur, hot));
        }
        Err(crate::Error::Io(io::Error::new(
            io::ErrorKind::NotFound,
            "No matching cursor found",
        )))
    }
    pub(super) fn new(conn: Rc<RustConnection>, root: u32) -> crate::Result<Self> {
        let theme = xcursor::CursorTheme::load("");
        let pix = PixmapWrapper::create_pixmap(conn.clone(), 1, root, 16, 16)?;

        let mut this = Self {
            theme,
            gc: GcontextWrapper::create_gc(conn.clone(), pix.pixmap(), &CreateGCAux::new())?,
            buffers: HashMap::new(),
            current_cursor: CursorType::Arrow,
        };

        for &ty in POINTERS {
            let (cur, hot) = this.load_cursor(conn.clone(), root, ty.to_string())?;
            this.buffers.insert(ty, (cur, hot));
        }

        this.set_cursor(CursorType::Arrow)?;

        Ok(this)
    }
    pub(super) fn set_cursor(&mut self, ty: crate::app::CursorType) -> crate::Result<XCursor> {
        let buffer = self.buffers.get(&ty).ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "The requested cursor isn't loaded",
        ))?;

        self.current_cursor = ty;

        Ok(buffer.0)
    }
}
