use std::sync::{Arc, Mutex};
use std::thread;
use tao::dpi::{LogicalSize, PhysicalPosition};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use tao::window::WindowBuilder;

pub struct SplashWindow {
    thread_handle: Option<thread::JoinHandle<()>>,
    close_sender: Arc<Mutex<Option<std::sync::mpsc::Sender<()>>>>,
}

impl SplashWindow {
    pub fn new(image_path: &str, width: u32, height: u32) -> Result<Self, String> {
        // Read image file on calling thread
        let image_data = std::fs::read(image_path)
            .map_err(|e| format!("Failed to read image file: {}", e))?;

        // Detect MIME type based on file extension
        let mime_type = if image_path.ends_with(".png") {
            "image/png"
        } else if image_path.ends_with(".jpg") || image_path.ends_with(".jpeg") {
            "image/jpeg"
        } else if image_path.ends_with(".ico") {
            "image/x-icon"
        } else if image_path.ends_with(".gif") {
            "image/gif"
        } else if image_path.ends_with(".webp") {
            "image/webp"
        } else {
            "image/png" // Default fallback
        };

        // Convert to base64 data URL
        let base64_data = base64_encode(&image_data);
        let data_url = format!("data:{};base64,{}", mime_type, base64_data);

        // Create minimal HTML to display the image
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <style>
        body {{
            margin: 0;
            padding: 0;
            overflow: hidden;
            display: flex;
            justify-content: center;
            align-items: center;
            width: 100vw;
            height: 100vh;
            background: transparent;
        }}
        img {{
            max-width: 100%;
            max-height: 100%;
            object-fit: contain;
        }}
    </style>
</head>
<body>
    <img src="{}" alt="Splash">
</body>
</html>"#,
            data_url
        );

        let (close_tx, close_rx) = std::sync::mpsc::channel::<()>();
        let close_sender = Arc::new(Mutex::new(Some(close_tx)));

        // Spawn a dedicated thread with its own event loop
        let handle = thread::spawn(move || {
            let event_loop = EventLoop::new();

            let mut window_builder = WindowBuilder::new()
                .with_title("") // No title for splash
                .with_inner_size(LogicalSize::new(width, height))
                .with_resizable(false)
                .with_decorations(false)
                .with_transparent(true)
                .with_always_on_top(true);

            // Platform-specific window attributes to hide from taskbar/dock
            #[cfg(target_os = "windows")]
            {
                use tao::platform::windows::WindowBuilderExtWindows;
                window_builder = window_builder.with_skip_taskbar(true);
            }

            #[cfg(target_os = "macos")]
            {
                use tao::platform::macos::WindowBuilderExtMacOS;
                window_builder = window_builder
                    .with_title_hidden(true)
                    .with_titlebar_transparent(true)
                    .with_fullsize_content_view(true);
            }

            #[cfg(target_os = "linux")]
            {
                use tao::platform::unix::WindowBuilderExtUnix;
                window_builder = window_builder.with_skip_taskbar(true);
            }

            let window = match window_builder.build(&event_loop) {
                Ok(w) => w,
                Err(_) => return,
            };

            // Center the window on the primary monitor
            if let Some(monitor) = window
                .current_monitor()
                .or_else(|| window.available_monitors().next())
            {
                let monitor_size = monitor.size();
                let monitor_pos = monitor.position();
                let window_size = window.outer_size();

                let x = monitor_pos.x + (monitor_size.width as i32 - window_size.width as i32) / 2;
                let y = monitor_pos.y + (monitor_size.height as i32 - window_size.height as i32) / 2;

                window.set_outer_position(PhysicalPosition::new(x, y));
            }

            let window = Arc::new(window);

            // Create a minimal WebView to display the HTML
            let _webview = match wry::WebViewBuilder::new()
                .with_html(html)
                .with_transparent(true)
                .build(&window)
            {
                Ok(wv) => wv,
                Err(_) => return,
            };

            // Run the event loop
            event_loop.run(move |event, _target: &EventLoopWindowTarget<()>, control_flow| {
                *control_flow = ControlFlow::Wait;

                // Check for close signal
                if close_rx.try_recv().is_ok() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            });
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

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let mut i = 0;

    while i + 2 < data.len() {
        let b1 = data[i];
        let b2 = data[i + 1];
        let b3 = data[i + 2];

        result.push(CHARS[(b1 >> 2) as usize] as char);
        result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
        result.push(CHARS[(((b2 & 0x0F) << 2) | (b3 >> 6)) as usize] as char);
        result.push(CHARS[(b3 & 0x3F) as usize] as char);

        i += 3;
    }

    // Handle remaining bytes
    if i < data.len() {
        let b1 = data[i];
        result.push(CHARS[(b1 >> 2) as usize] as char);

        if i + 1 < data.len() {
            let b2 = data[i + 1];
            result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
            result.push(CHARS[((b2 & 0x0F) << 2) as usize] as char);
            result.push('=');
        } else {
            result.push(CHARS[((b1 & 0x03) << 4) as usize] as char);
            result.push_str("==");
        }
    }

    result
}
