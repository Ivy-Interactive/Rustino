mod callbacks;
mod commands;
mod config;
mod icon;
mod state;
mod util;
mod window;

use std::ffi::c_void;
use std::os::raw::c_char;
use std::panic::catch_unwind;
use std::sync::atomic::Ordering;

use commands::RustinoCommand;
use config::{RustinoInitParams, WindowConfig};
use window::RustinoWindow;

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn rustino_ctor(params: *const RustinoInitParams) -> *mut RustinoWindow {
    catch_unwind(|| {
        if params.is_null() {
            return std::ptr::null_mut();
        }
        let config = WindowConfig::from_params(unsafe { &*params });
        let instance = RustinoWindow::new(config);
        Box::into_raw(Box::new(instance))
    })
    .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_dtor(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if !instance.is_null() {
            unsafe {
                drop(Box::from_raw(instance));
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_wait_for_exit(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.run();
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_close(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            inst.send_command(RustinoCommand::Close);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_free_string(s: *mut c_char) {
    let _ = catch_unwind(|| {
        util::free_cstring(s);
    });
}

// ---------------------------------------------------------------------------
// Dual-mode setters (pre-run: modify config, post-run: send command)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_title(instance: *mut RustinoWindow, title: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            if let Some(t) = unsafe { util::cstr_to_string(title) } {
                if !inst.send_command(RustinoCommand::SetTitle(t.clone())) {
                    inst.config.title = t;
                }
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_size(instance: *mut RustinoWindow, width: i32, height: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            let w = width.max(1) as u32;
            let h = height.max(1) as u32;
            if !inst.send_command(RustinoCommand::SetSize(w, h)) {
                inst.config.width = w;
                inst.config.height = h;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_resizable(instance: *mut RustinoWindow, resizable: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            let v = resizable != 0;
            if !inst.send_command(RustinoCommand::SetResizable(v)) {
                inst.config.resizable = v;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_topmost(instance: *mut RustinoWindow, topmost: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            let v = topmost != 0;
            if !inst.send_command(RustinoCommand::SetTopmost(v)) {
                inst.config.topmost = v;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_icon_file(instance: *mut RustinoWindow, path: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            if let Some(p) = unsafe { util::cstr_to_string(path) } {
                if !inst.send_command(RustinoCommand::SetIconFile(p.clone())) {
                    inst.config.icon_file = Some(p);
                }
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_center(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            if !inst.send_command(RustinoCommand::Center) {
                inst.config.center = true;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_navigate_to_url(instance: *mut RustinoWindow, url: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            if let Some(u) = unsafe { util::cstr_to_string(url) } {
                if !inst.send_command(RustinoCommand::LoadUrl(u.clone())) {
                    inst.config.start_url = Some(u);
                    inst.config.start_html = None;
                }
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_navigate_to_string(
    instance: *mut RustinoWindow,
    content: *const c_char,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            if let Some(c) = unsafe { util::cstr_to_string(content) } {
                if !inst.send_command(RustinoCommand::LoadHtml(c.clone())) {
                    inst.config.start_html = Some(c);
                    inst.config.start_url = None;
                }
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_background_color(
    instance: *mut RustinoWindow,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            if !inst.send_command(RustinoCommand::SetBackgroundColor(r, g, b, a)) {
                inst.config.background_color = Some((r, g, b, a));
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Pre-run only setters (builder-time configuration)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_use_os_default_size(instance: *mut RustinoWindow, use_default: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.use_os_default_size = use_default != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_devtools_enabled(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.devtools_enabled = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_clipboard_enabled(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.clipboard_enabled = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_ignore_cert_errors(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.ignore_certificate_errors = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_web_security_enabled(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.web_security_enabled = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_transparent(instance: *mut RustinoWindow, transparent: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.transparent = transparent != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_decorations(instance: *mut RustinoWindow, decorated: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            let v = decorated != 0;
            if !inst.send_command(RustinoCommand::SetDecorations(v)) {
                inst.config.decorations = v;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_user_agent(instance: *mut RustinoWindow, ua: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.user_agent = unsafe { util::cstr_to_string(ua) };
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_user_data_folder(instance: *mut RustinoWindow, path: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.user_data_folder = unsafe { util::cstr_to_string(path) };
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_media_autoplay(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.media_autoplay = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_zoom_hotkeys(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.zoom_hotkeys = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_add_init_script(instance: *mut RustinoWindow, js: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            if let Some(s) = unsafe { util::cstr_to_string(js) } {
                inst.config.initialization_scripts.push(s);
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Window state (post-run commands)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_minimized(instance: *mut RustinoWindow, minimized: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            inst.send_command(RustinoCommand::SetMinimized(minimized != 0));
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_maximized(instance: *mut RustinoWindow, maximized: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            let v = maximized != 0;
            if !inst.send_command(RustinoCommand::SetMaximized(v)) {
                inst.config.maximized = v;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_fullscreen(instance: *mut RustinoWindow, fullscreen: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            let v = fullscreen != 0;
            if !inst.send_command(RustinoCommand::SetFullscreen(v)) {
                inst.config.fullscreen = v;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_visible(instance: *mut RustinoWindow, visible: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            let v = visible != 0;
            if !inst.send_command(RustinoCommand::SetVisible(v)) {
                inst.config.visible = v;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_focus(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            inst.send_command(RustinoCommand::SetFocus);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_position(instance: *mut RustinoWindow, x: i32, y: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            if !inst.send_command(RustinoCommand::SetPosition(x, y)) {
                inst.config.position = Some((x, y));
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_min_size(instance: *mut RustinoWindow, width: i32, height: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            let size = if width > 0 && height > 0 {
                Some((width as u32, height as u32))
            } else {
                None
            };
            if !inst.send_command(RustinoCommand::SetMinSize(size)) {
                inst.config.min_size = size;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_max_size(instance: *mut RustinoWindow, width: i32, height: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            let size = if width > 0 && height > 0 {
                Some((width as u32, height as u32))
            } else {
                None
            };
            if !inst.send_command(RustinoCommand::SetMaxSize(size)) {
                inst.config.max_size = size;
            }
        }
    });
}

// ---------------------------------------------------------------------------
// State queries (read from shared state, no round-trip)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn rustino_is_minimized(instance: *mut RustinoWindow) -> i32 {
    catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            if inst.state.is_minimized.load(Ordering::Acquire) { 1 } else { 0 }
        } else {
            0
        }
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_is_maximized(instance: *mut RustinoWindow) -> i32 {
    catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            if inst.state.is_maximized.load(Ordering::Acquire) { 1 } else { 0 }
        } else {
            0
        }
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_is_fullscreen(instance: *mut RustinoWindow) -> i32 {
    catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            if inst.state.is_fullscreen.load(Ordering::Acquire) { 1 } else { 0 }
        } else {
            0
        }
    })
    .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_get_position(
    instance: *mut RustinoWindow,
    out_x: *mut i32,
    out_y: *mut i32,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            let (x, y) = inst.state.load_position();
            if !out_x.is_null() {
                unsafe { *out_x = x };
            }
            if !out_y.is_null() {
                unsafe { *out_y = y };
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_get_size(
    instance: *mut RustinoWindow,
    out_w: *mut i32,
    out_h: *mut i32,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            let (w, h) = inst.state.load_size();
            if !out_w.is_null() {
                unsafe { *out_w = w as i32 };
            }
            if !out_h.is_null() {
                unsafe { *out_h = h as i32 };
            }
        }
    });
}

// ---------------------------------------------------------------------------
// WebView operations (post-run only)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn rustino_evaluate_script(instance: *mut RustinoWindow, js: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            if let Some(s) = unsafe { util::cstr_to_string(js) } {
                inst.send_command(RustinoCommand::EvaluateScript(s));
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_send_web_message(instance: *mut RustinoWindow, msg: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            if let Some(m) = unsafe { util::cstr_to_string(msg) } {
                inst.send_command(RustinoCommand::SendWebMessage(m));
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_zoom(instance: *mut RustinoWindow, factor: f64) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            if factor.is_finite() && factor > 0.0 {
                inst.send_command(RustinoCommand::SetZoom(factor));
            }
        }
    });
}

// ---------------------------------------------------------------------------
// Dialogs (post-run only — dispatched on event loop thread)
// ---------------------------------------------------------------------------

fn parse_filters(raw: *const c_char) -> Vec<(String, Vec<String>)> {
    let s = match unsafe { util::cstr_to_string(raw) } {
        Some(s) if !s.is_empty() => s,
        _ => return Vec::new(),
    };
    s.split(';')
        .filter_map(|group| {
            let (name, exts) = group.split_once('|')?;
            let extensions = exts.split(',').map(|e| e.trim().to_string()).collect();
            Some((name.to_string(), extensions))
        })
        .collect()
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_show_open_file_dialog(
    instance: *mut RustinoWindow,
    title: *const c_char,
    default_path: *const c_char,
    filters: *const c_char,
    multi_select: i32,
) -> *mut c_char {
    catch_unwind(|| {
        let inst = unsafe { instance.as_ref() }?;
        let params = commands::DialogParams {
            title: unsafe { util::cstr_to_string(title) },
            default_path: unsafe { util::cstr_to_string(default_path) },
            filters: parse_filters(filters),
            multi_select: multi_select != 0,
        };
        let (tx, rx) = std::sync::mpsc::channel();
        inst.send_command(RustinoCommand::ShowOpenFileDialog(params, tx));
        let paths = rx.recv().ok()??;
        let joined = paths.join("\n");
        std::ffi::CString::new(joined).ok().map(|s| s.into_raw())
    })
    .ok()
    .flatten()
    .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_show_save_file_dialog(
    instance: *mut RustinoWindow,
    title: *const c_char,
    default_path: *const c_char,
    filters: *const c_char,
) -> *mut c_char {
    catch_unwind(|| {
        let inst = unsafe { instance.as_ref() }?;
        let params = commands::DialogParams {
            title: unsafe { util::cstr_to_string(title) },
            default_path: unsafe { util::cstr_to_string(default_path) },
            filters: parse_filters(filters),
            multi_select: false,
        };
        let (tx, rx) = std::sync::mpsc::channel();
        inst.send_command(RustinoCommand::ShowSaveFileDialog(params, tx));
        let path = rx.recv().ok()??;
        std::ffi::CString::new(path).ok().map(|s| s.into_raw())
    })
    .ok()
    .flatten()
    .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_show_select_folder_dialog(
    instance: *mut RustinoWindow,
    title: *const c_char,
    default_path: *const c_char,
    multi_select: i32,
) -> *mut c_char {
    catch_unwind(|| {
        let inst = unsafe { instance.as_ref() }?;
        let params = commands::DialogParams {
            title: unsafe { util::cstr_to_string(title) },
            default_path: unsafe { util::cstr_to_string(default_path) },
            filters: Vec::new(),
            multi_select: multi_select != 0,
        };
        let (tx, rx) = std::sync::mpsc::channel();
        inst.send_command(RustinoCommand::ShowSelectFolderDialog(params, tx));
        let paths = rx.recv().ok()??;
        let joined = paths.join("\n");
        std::ffi::CString::new(joined).ok().map(|s| s.into_raw())
    })
    .ok()
    .flatten()
    .unwrap_or(std::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// Callback registration (pre-run only)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_callback_context(
    instance: *mut RustinoWindow,
    context: *mut c_void,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.context = context;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_closing_handler(
    instance: *mut RustinoWindow,
    handler: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.on_closing = handler;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_closed_handler(
    instance: *mut RustinoWindow,
    handler: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.on_closed = handler;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_resized_handler(
    instance: *mut RustinoWindow,
    handler: Option<unsafe extern "C" fn(*mut c_void, i32, i32)>,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.on_resized = handler;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_moved_handler(
    instance: *mut RustinoWindow,
    handler: Option<unsafe extern "C" fn(*mut c_void, i32, i32)>,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.on_moved = handler;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_focus_changed_handler(
    instance: *mut RustinoWindow,
    handler: Option<unsafe extern "C" fn(*mut c_void, i32)>,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.on_focus_changed = handler;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_web_message_received_handler(
    instance: *mut RustinoWindow,
    handler: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.on_web_message = handler;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_page_load_handler(
    instance: *mut RustinoWindow,
    handler: Option<unsafe extern "C" fn(*mut c_void, i32, *const c_char)>,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.on_page_load = handler;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn rustino_set_navigation_handler(
    instance: *mut RustinoWindow,
    handler: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> i32>,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.on_navigation = handler;
        }
    });
}
