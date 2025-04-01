use std::{mem::MaybeUninit, os::raw::c_void, rc::Rc};

use windows::{
    core::{BOOL, PCWSTR},
    Win32::{
        Foundation::{GetLastError, HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateCompatibleBitmap, DrawStateW, EndPaint,
            SetDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, COLOR_WINDOW, DIB_RGB_COLORS,
            DST_BITMAP, HBRUSH, RGBQUAD,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, GetWindowLongPtrW,
            LoadCursorW, LoadIconW, PostQuitMessage, RegisterClassExW, SetWindowLongPtrW,
            TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, GWLP_USERDATA, IDC_ARROW,
            IDI_APPLICATION, WM_CLOSE, WM_CREATE, WM_DESTROY, WM_PAINT, WNDCLASSEXW,
        },
    },
};

use crate::event::{Event, RawEvent};

use super::window::WindowData;

pub(crate) struct App {
    pub(super) module: HMODULE,
    pub(super) class: u16,
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

        Ok(Rc::new(App { module, class }))
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
                Ok(Some(RawEvent {
                    window: 0,
                    event: Event::Unknown,
                }))
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
            let mut paint_struct = MaybeUninit::zeroed();
            let hdc = BeginPaint(hwnd, paint_struct.as_mut_ptr());
            if hdc.is_invalid() {
                return LRESULT(0);
            }
            let hbm = CreateCompatibleBitmap(hdc, 800, 600);
            if hbm.is_invalid() {
                return LRESULT(0);
            }
            let binfo = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: 800,
                    biHeight: -600,
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
            let win_data = window_data.unwrap();
            let buf = win_data.buffer.borrow();
            let bgr_data = buf
                .data()
                .chunks_exact(3)
                .flat_map(|s| [s[2], s[1], s[0]])
                .collect::<Vec<_>>();

            let result = SetDIBits(
                Some(hdc),
                hbm,
                0,
                600,
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
                LPARAM(hbm.0 as isize),
                WPARAM(0),
                0,
                0,
                800,
                600,
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
