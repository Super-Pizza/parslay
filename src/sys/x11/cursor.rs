use std::{borrow::Cow, collections::HashMap, fs, io, rc::Rc};

use lite_graphics::Offset;
use x11rb::{
    connection::Connection as _,
    image::{BitsPerPixel, Image, ImageOrder, ScanlinePad},
    protocol::{
        render::{
            CreatePictureAux, PictType, Pictformat, PictureWrapper, QueryPictFormatsReply,
            create_cursor, query_pict_formats,
        },
        xproto::{CreateGCAux, CursorWrapper, GcontextWrapper, PixmapWrapper},
    },
    rust_connection::RustConnection,
};

use crate::app::CursorType;

pub(super) struct Cursor {
    gc: GcontextWrapper<Rc<RustConnection>>,
    theme: xcursor::CursorTheme,
    buffers: HashMap<CursorType, (CursorWrapper<Rc<RustConnection>>, Offset)>,
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
    ) -> crate::Result<(CursorWrapper<Rc<RustConnection>>, Offset)> {
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
                32,
                root,
                cursor.width as u16,
                cursor.height as u16,
            )?;

            let img = Image::new(
                cursor.width as _,
                cursor.height as _,
                ScanlinePad::Pad8,
                32,
                BitsPerPixel::B32,
                ImageOrder::LsbFirst,
                Cow::Borrowed(&cursor.pixels_rgba),
            )
            .unwrap();
            img.put(&conn, source.pixmap(), self.gc.gcontext(), 0, 0)?;

            let picture = PictureWrapper::create_picture(
                conn.clone(),
                source.pixmap(),
                find_format(&query_pict_formats(&conn)?.reply()?),
                &CreatePictureAux::new(),
            )?;

            let cur_id = conn.generate_id()?;

            let cur = CursorWrapper::for_cursor(conn.clone(), cur_id);

            create_cursor(
                &conn,
                cur_id,
                picture.picture(),
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
        let pix = PixmapWrapper::create_pixmap(conn.clone(), 32, root, 16, 16)?;

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
    pub(super) fn set_cursor(&mut self, ty: crate::app::CursorType) -> crate::Result<u32> {
        let buffer = self.buffers.get(&ty).ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "The requested cursor isn't loaded",
        ))?;

        self.current_cursor = ty;

        Ok(buffer.0.cursor())
    }
}

fn find_format(reply: &QueryPictFormatsReply) -> Pictformat {
    reply
        .formats
        .iter()
        .filter(|format| {
            format.type_ == PictType::DIRECT
                && format.depth == 32
                && format.direct.red_shift == 16
                && format.direct.red_mask == 0xff
                && format.direct.green_shift == 8
                && format.direct.green_mask == 0xff
                && format.direct.blue_shift == 0
                && format.direct.blue_mask == 0xff
                && format.direct.alpha_shift == 24
                && format.direct.alpha_mask == 0xff
        })
        .map(|format| format.id)
        .next()
        .expect("The X11 server is missing the RENDER ARGB_32 standard format!")
}
