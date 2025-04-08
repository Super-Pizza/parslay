use std::{cell::RefCell, mem::MaybeUninit, os::raw::c_void, rc::Rc};

use windows::{
    core::{BOOL, PCWSTR},
    Win32::{
        Foundation::{GetLastError, HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateCompatibleBitmap, DrawStateW, EndPaint, SetDIBits, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, COLOR_WINDOW, DIB_RGB_COLORS, DST_BITMAP, HBRUSH, RGBQUAD,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, GetWindowLongPtrW,
            LoadCursorW, LoadIconW, PostQuitMessage, RegisterClassExW, SetWindowLongPtrW,
            TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, IDC_ARROW,
            IDI_APPLICATION, WM_ACTIVATE, WM_CLOSE, WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN,
            WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_PAINT, WM_RBUTTONDOWN,
            WM_RBUTTONUP, WM_SIZE, WM_XBUTTONDOWN, WM_XBUTTONUP, WNDCLASSEXW,
        },
    },
};

use crate::event::{Button, Event, RawEvent, WidgetEvent, WindowEvent, WindowState};

use super::{window::WindowData, Window};

pub(crate) struct App {
    pub(super) module: HMODULE,
    pub(super) class: u16,
    pub(super) windows: RefCell<Vec<Rc<Window>>>,
}

impl App {
    pub(crate) fn new() -> crate::Result<Rc<Self>> {
        let module = unsafe { GetModuleHandleW(PCWSTR::null()) }?;

        let class_info = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: HINSTANCE(module.0),
            hIcon: unsafe { LoadIconW(None, IDI_APPLICATION) }?,
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }?,
            hbrBackground: HBRUSH((COLOR_WINDOW.0 as usize + 1) as *mut c_void),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: windows::core::w!("parslay.windows"),
            hIconSm: unsafe { LoadIconW(None, IDI_APPLICATION) }?,
        };

        let class = unsafe { RegisterClassExW(&class_info) };

        if class == 0 {
            Err(unsafe { GetLastError() })?
        }

        Ok(Rc::new(App {
            module,
            class,
            windows: RefCell::new(vec![]),
        }))
    }
    pub(crate) fn get_event(&self) -> crate::Result<Option<crate::event::RawEvent>> {
        let mut msg = MaybeUninit::uninit();
        let result = unsafe { GetMessageW(msg.as_mut_ptr(), None, 0, 0) };
        match result {
            BOOL(0) => Ok(None),
            BOOL(-1) => Err(unsafe { GetLastError().to_hresult().into() }),
            _ => unsafe {
                let _ = TranslateMessage(msg.as_ptr());
                DispatchMessageW(msg.as_ptr());
                let mut event = Event::Unknown;
                let mut wid = 0;
                for window in &*self.windows.borrow() {
                    let mut last_event = window.data.events.borrow_mut();
                    if let Some(ev) = last_event.pop_front() {
                        event = ev;
                        wid = window.id();
                    }
                }
                Ok(Some(RawEvent { window: wid, event }))
            },
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let userdata = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
    let window_data = if userdata == 0 {
        None
    } else {
        Rc::increment_strong_count(userdata as *const WindowData);
        Some(Rc::from_raw(userdata as *const WindowData))
    };
    match msg {
        WM_CREATE => {
            let create_struct = *(lparam.0 as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create_struct.lpCreateParams as isize);
            LRESULT(0)
        }
        WM_PAINT => {
            let window_data = window_data.unwrap();
            let mut paint_struct = MaybeUninit::zeroed();
            let hdc = BeginPaint(hwnd, paint_struct.as_mut_ptr());
            if hdc.is_invalid() {
                return LRESULT(0);
            }
            let mut win_hbm = window_data.hbm.borrow_mut();
            let size = window_data.buffer.borrow().size();

            if win_hbm.is_invalid() {
                *win_hbm = CreateCompatibleBitmap(hdc, size.w as i32, size.h as i32);
                if win_hbm.is_invalid() {
                    return LRESULT(0);
                }
            }

            let binfo = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: size.w as i32,
                    biHeight: -(size.h as i32),
                    biPlanes: 1,
                    biBitCount: 24,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 1000,
                    biYPelsPerMeter: 1000,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0,
                }],
            };
            let buf = window_data.buffer.borrow();
            let bgr_data = buf
                .data()
                .chunks_exact(3)
                .flat_map(|s| [s[2], s[1], s[0]])
                .collect::<Vec<_>>();

            let result = SetDIBits(
                Some(hdc),
                *win_hbm,
                0,
                size.h,
                bgr_data.as_ptr() as *const c_void,
                &binfo,
                DIB_RGB_COLORS,
            );
            if result == 0 {
                return LRESULT(0);
            }

            let result = DrawStateW(
                hdc,
                None,
                None,
                LPARAM(win_hbm.0 as isize),
                WPARAM(0),
                0,
                0,
                size.w as i32,
                size.h as i32,
                DST_BITMAP,
            );
            if !result.as_bool() {
                return LRESULT(0);
            }

            let result = EndPaint(hwnd, paint_struct.as_ptr());
            if !result.as_bool() {
                return LRESULT(0);
            }
            LRESULT(0)
        }
        WM_ACTIVATE => {
            let window_data = window_data.unwrap();
            let event = if wparam.0 & 0xFFFF == 0 {
                Event::Window(WindowEvent::StateChange(WindowState::Suspended))
            } else {
                Event::Window(WindowEvent::StateChange(WindowState::Activated))
            };
            window_data.events.borrow_mut().push_back(event);
            LRESULT(0)
        }
        WM_SIZE => {
            let window_data = window_data.unwrap();
            if wparam.0 == 2 {
                let event = Event::Window(WindowEvent::StateChange(WindowState::Maximized));
                window_data.events.borrow_mut().push_back(event);
            } else if wparam.0 == 0 {
                let event = Event::Window(WindowEvent::Resize(
                    (lparam.0 & 0xFFFF) as u32,
                    (lparam.0 >> 16) as u32,
                ));
                window_data.events.borrow_mut().push_back(event);
            }
            LRESULT(0)
        }
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN => {
            let window_data = window_data.unwrap();
            let x = (lparam.0 & 0xFFFF) as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i32;
            let button = if wparam.0 & 1 > 0 {
                Button::Left
            } else if wparam.0 & 2 > 0 {
                Button::Right
            } else if wparam.0 & 0x10 > 0 {
                Button::Middle
            } else if wparam.0 & 0x40 > 0 {
                Button::Forward
            } else if wparam.0 & 0x20 > 0 {
                Button::Back
            } else {
                Button::Other
            };
            window_data
                .events
                .borrow_mut()
                .push_back(Event::Widget(WidgetEvent::ButtonPress(button, x, y)));
            LRESULT(0)
        }
        WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP => {
            let window_data = window_data.unwrap();
            let x = (lparam.0 & 0xFFFF) as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i32;
            let button = if msg == WM_LBUTTONUP {
                Button::Left
            } else if msg == WM_RBUTTONUP {
                Button::Right
            } else if msg == WM_MBUTTONUP {
                Button::Middle
            } else if wparam.0 & 0x20000 > 0 {
                Button::Forward
            } else if wparam.0 & 0x10000 > 0 {
                Button::Back
            } else {
                Button::Other
            };
            window_data
                .events
                .borrow_mut()
                .push_back(Event::Widget(WidgetEvent::ButtonRelease(button, x, y)));
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let window_data = window_data.unwrap();
            let x = (lparam.0 & 0xFFFF) as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i32;
            window_data
                .events
                .borrow_mut()
                .push_back(Event::Widget(WidgetEvent::Move(x, y)));
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
