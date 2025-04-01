use std::{cell::RefCell, collections::VecDeque, os::raw::c_void, rc::Rc};

use lite_graphics::draw::Buffer;
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{GetLastError, HINSTANCE, HWND},
        UI::WindowsAndMessaging::{
            CreateWindowExW, CW_USEDEFAULT, WINDOW_EX_STYLE, WS_CAPTION,
            WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE,
        },
    },
};

use crate::event::Event;

use super::App;

pub(crate) struct Window {
    hwnd: HWND,
    data: Rc<WindowData>,
}

pub(super) struct WindowData {
    pub(super) buffer: RefCell<Buffer>,
    pub(super) events: RefCell<VecDeque<Event>>,
}

impl Window {
    pub(crate) fn new(app: &Rc<App>) -> crate::Result<Rc<Self>> {
        let data = Rc::new(WindowData {
            buffer: RefCell::new(Buffer::new(800, 600)),
            events: RefCell::new(VecDeque::new()),
        });
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PCWSTR(app.class as usize as *const u16),
                w!("Hello!"),
                WS_CAPTION
                    | WS_SYSMENU
                    | WS_THICKFRAME
                    | WS_MINIMIZEBOX
                    | WS_MAXIMIZEBOX
                    | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                800,
                600,
                None,
                None,
                Some(HINSTANCE(app.module.0)),
                Some(Rc::into_raw(data.clone()) as *const c_void),
            )
        }?;
        if hwnd.is_invalid() {
            Err(unsafe { GetLastError() })?
        }

        Ok(Rc::new(Window { hwnd, data }))
    }
    pub(crate) fn draw(&self, buf: Buffer) -> crate::Result<()> {
        *self.data.buffer.borrow_mut() = buf;
        Ok(())
    }
    pub(crate) fn id(&self) -> u64 {
        self.hwnd.0 as usize as u64
    }
}
