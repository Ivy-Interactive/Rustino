use std::sync::{Arc, Mutex};
use std::thread;

pub struct SplashWindow {
    thread_handle: Option<thread::JoinHandle<()>>,
    close_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<()>>>>,
}

impl SplashWindow {
    pub fn new(image_path: &str, width: u32, height: u32) -> Result<Self, String> {
        let image_data = std::fs::read(image_path)
            .map_err(|e| format!("Failed to read image file: {}", e))?;

        let img = image::load_from_memory(&image_data)
            .map_err(|e| format!("Failed to decode image: {}", e))?
            .resize_exact(width, height, image::imageops::FilterType::Lanczos3)
            .to_rgba8();

        let (close_tx, close_rx) = std::sync::mpsc::channel::<()>();
        let close_sender = Arc::new(Mutex::new(Some(close_tx)));

        let handle = thread::spawn(move || {
            #[cfg(target_os = "windows")]
            windows_splash::run(img, width, height, close_rx);

            #[cfg(not(target_os = "windows"))]
            {
                let _ = (img, width, height, close_rx);
            }
        });

        Ok(SplashWindow {
            thread_handle: Some(handle),
            close_sender,
        })
    }

    pub fn close(&self) {
        if let Ok(mut sender) = self.close_sender.lock() {
            if let Some(tx) = sender.take() {
                let _ = tx.send(());
            }
        }
    }
}

impl Drop for SplashWindow {
    fn drop(&mut self) {
        self.close();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_splash {
    use image::RgbaImage;
    use std::sync::mpsc::Receiver;
    use windows::core::*;
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::*;

    pub fn run(img: RgbaImage, width: u32, height: u32, close_rx: Receiver<()>) {
        unsafe {
            let class_name = w!("RustinoSplash");
            let instance: HINSTANCE = std::mem::zeroed();

            let wc = WNDCLASSEXW {
                cbSize: size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wnd_proc),
                hInstance: instance,
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
                lpszClassName: class_name,
                ..Default::default()
            };
            RegisterClassExW(&wc);

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let x = (screen_w - width as i32) / 2;
            let y = (screen_h - height as i32) / 2;

            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name,
                w!(""),
                WS_POPUP | WS_VISIBLE,
                x,
                y,
                width as i32,
                height as i32,
                None,
                None,
                Some(instance),
                None,
            )
            .unwrap();

            // Paint the image
            paint_image(hwnd, &img, width, height);

            let mut msg = MSG::default();
            loop {
                if close_rx.try_recv().is_ok() {
                    break;
                }

                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    if msg.message == WM_QUIT {
                        return;
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                std::thread::sleep(std::time::Duration::from_millis(16));
            }

            let _ = DestroyWindow(hwnd);
            let _ = UnregisterClassW(class_name, Some(instance));
        }
    }

    unsafe fn paint_image(hwnd: HWND, img: &RgbaImage, width: u32, height: u32) {
        unsafe {
            let hdc = GetDC(Some(hwnd));
            let mem_dc = CreateCompatibleDC(Some(hdc));

            let bi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width as i32,
                    biHeight: -(height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: 0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let bitmap = CreateDIBSection(Some(hdc), &bi, DIB_RGB_COLORS, &mut bits, None, 0)
                .unwrap_or_default();

            if !bits.is_null() {
                let dst = std::slice::from_raw_parts_mut(bits as *mut u8, (width * height * 4) as usize);
                for (dst_px, src_px) in dst.chunks_exact_mut(4).zip(img.pixels()) {
                    let [r, g, b, a] = src_px.0;
                    dst_px[0] = b;
                    dst_px[1] = g;
                    dst_px[2] = r;
                    dst_px[3] = a;
                }
            }

            let old = SelectObject(mem_dc, bitmap.into());
            let _ = BitBlt(hdc, 0, 0, width as i32, height as i32, Some(mem_dc), 0, 0, SRCCOPY);

            SelectObject(mem_dc, old);
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(mem_dc);
            ReleaseDC(Some(hwnd), hdc);
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            match msg {
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }
}
