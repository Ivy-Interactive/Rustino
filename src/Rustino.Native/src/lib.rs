#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::unnecessary_map_or)]

mod callbacks;
mod commands;
mod config;
mod icon;
mod menu;
mod splash;
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
pub unsafe extern "C" fn rustino_ctor(params: *const RustinoInitParams) -> *mut RustinoWindow {
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
pub unsafe extern "C" fn rustino_dtor(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if !instance.is_null() {
            unsafe {
                drop(Box::from_raw(instance));
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_wait_for_exit(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.run();
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_close(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            inst.send_command(RustinoCommand::Close);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_free_string(s: *mut c_char) {
    let _ = catch_unwind(|| {
        util::free_cstring(s);
    });
}

// ---------------------------------------------------------------------------
// Notifications (standalone — no instance required)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_show_notification(
    title: *const c_char,
    body: *const c_char,
    icon: *const c_char,
    app_id: *const c_char,
) -> i32 {
    catch_unwind(|| {
        let title = unsafe { util::cstr_to_string(title) }.unwrap_or_default();
        let body = unsafe { util::cstr_to_string(body) }.unwrap_or_default();
        let mut n = notify_rust::Notification::new();
        n.summary(&title).body(&body);
        if let Some(icon_path) = unsafe { util::cstr_to_string(icon) } {
            n.icon(&icon_path);
        }
        #[cfg(target_os = "windows")]
        if let Some(id) = unsafe { util::cstr_to_string(app_id) } {
            n.app_id(&id);
        }
        #[cfg(target_os = "macos")]
        {
            let requested = unsafe { util::cstr_to_string(app_id) };
            macos_notification::ensure_application_set(requested.as_deref());
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let _ = app_id;
        if n.show().is_ok() { 1 } else { 0 }
    })
    .unwrap_or(0)
}

// notify-rust's macOS backend lazily calls `set_application("use_default")` on the
// first notification if nobody has set one, which makes macOS pop a "Where is
// use_default?" app-picker dialog (see Ivy-Tendril#1682). We must call
// `notify_rust::set_application` ourselves before that happens — and since its
// internal `Once` latches even on failure, it can only ever be called once, with
// an id we already know is valid.
#[cfg(target_os = "macos")]
mod macos_notification {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSBundle, NSString};
    use std::sync::Once;

    static ENSURE_APPLICATION_SET: Once = Once::new();

    /// Picks a LaunchServices-registered bundle id and calls `notify_rust::set_application`
    /// with it exactly once. Must be called before the first `Notification::show()`.
    pub fn ensure_application_set(requested_app_id: Option<&str>) {
        ENSURE_APPLICATION_SET.call_once(|| {
            let bundle_id = pick_bundle_id(requested_app_id);
            let _ = notify_rust::set_application(&bundle_id);
        });
    }

    /// Returns the first candidate that is actually registered with LaunchServices,
    /// falling back to "com.apple.Finder" (guaranteed installed; notify-rust's own
    /// eventual default) if nothing else resolves.
    fn pick_bundle_id(requested_app_id: Option<&str>) -> String {
        let main_bundle_id = main_bundle_identifier();
        let candidates = [
            requested_app_id,
            main_bundle_id.as_deref(),
            Some("com.apple.Terminal"),
        ];
        candidates
            .into_iter()
            .flatten()
            .find(|id| !id.is_empty() && is_registered_bundle_id(id))
            .map(str::to_string)
            .unwrap_or_else(|| "com.apple.Finder".to_string())
    }

    /// The running app's own bundle identifier, when running inside a real .app bundle.
    fn main_bundle_identifier() -> Option<String> {
        NSBundle::mainBundle().bundleIdentifier().map(|s| s.to_string())
    }

    /// LaunchServices only accepts bundle ids it knows about; `setApplication` silently
    /// rejects anything else, so we pre-validate via the same lookup it uses internally
    /// (`LSCopyApplicationURLsForBundleIdentifier`, exposed here as `URLForApplication...`).
    fn is_registered_bundle_id(bundle_id: &str) -> bool {
        let workspace = NSWorkspace::sharedWorkspace();
        let id = NSString::from_str(bundle_id);
        workspace.URLForApplicationWithBundleIdentifier(&id).is_some()
    }
}

// ---------------------------------------------------------------------------
// Dual-mode setters (pre-run: modify config, post-run: send command)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_title(instance: *mut RustinoWindow, title: *const c_char) {
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
pub unsafe extern "C" fn rustino_set_size(instance: *mut RustinoWindow, width: i32, height: i32) {
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
pub unsafe extern "C" fn rustino_set_resizable(instance: *mut RustinoWindow, resizable: i32) {
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
pub unsafe extern "C" fn rustino_set_topmost(instance: *mut RustinoWindow, topmost: i32) {
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
pub unsafe extern "C" fn rustino_set_icon_file(instance: *mut RustinoWindow, path: *const c_char) {
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
pub unsafe extern "C" fn rustino_center(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            if !inst.send_command(RustinoCommand::Center) {
                inst.config.center = true;
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_about_name(instance: *mut RustinoWindow, name: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.about_name = unsafe { util::cstr_to_string(name) };
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_about_version(instance: *mut RustinoWindow, version: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.about_version = unsafe { util::cstr_to_string(version) };
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_about_copyright(instance: *mut RustinoWindow, copyright: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.about_copyright = unsafe { util::cstr_to_string(copyright) };
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_about_website(instance: *mut RustinoWindow, website: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.about_website = unsafe { util::cstr_to_string(website) };
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_about_license(instance: *mut RustinoWindow, license: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.about_license = unsafe { util::cstr_to_string(license) };
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_about_authors(instance: *mut RustinoWindow, authors: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.about_authors = unsafe { util::cstr_to_string(authors) }
                .map(|s| s.lines().map(|l| l.to_string()).collect())
                .unwrap_or_default();
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_about_comments(instance: *mut RustinoWindow, comments: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.about_comments = unsafe { util::cstr_to_string(comments) };
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_navigate_to_url(instance: *mut RustinoWindow, url: *const c_char) {
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
pub unsafe extern "C" fn rustino_navigate_to_string(
    instance: *mut RustinoWindow,
    content: *const c_char,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() }
            && let Some(c) = unsafe { util::cstr_to_string(content) }
            && !inst.send_command(RustinoCommand::LoadHtml(c.clone()))
        {
            inst.config.start_html = Some(c);
            inst.config.start_url = None;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_background_color(
    instance: *mut RustinoWindow,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() }
            && !inst.send_command(RustinoCommand::SetBackgroundColor(r, g, b, a))
        {
            inst.config.background_color = Some((r, g, b, a));
        }
    });
}

// ---------------------------------------------------------------------------
// Pre-run only setters (builder-time configuration)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_use_os_default_size(instance: *mut RustinoWindow, use_default: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.use_os_default_size = use_default != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_devtools_enabled(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.devtools_enabled = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_clipboard_enabled(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.clipboard_enabled = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_ignore_cert_errors(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.ignore_certificate_errors = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_web_security_enabled(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.web_security_enabled = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_transparent(instance: *mut RustinoWindow, transparent: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.transparent = transparent != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_decorations(instance: *mut RustinoWindow, decorated: i32) {
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
pub unsafe extern "C" fn rustino_set_user_agent(instance: *mut RustinoWindow, ua: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.user_agent = unsafe { util::cstr_to_string(ua) };
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_user_data_folder(instance: *mut RustinoWindow, path: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.user_data_folder = unsafe { util::cstr_to_string(path) };
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_media_autoplay(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.media_autoplay = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_zoom_hotkeys(instance: *mut RustinoWindow, enabled: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.config.zoom_hotkeys = enabled != 0;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_add_init_script(instance: *mut RustinoWindow, js: *const c_char) {
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
pub unsafe extern "C" fn rustino_set_minimized(instance: *mut RustinoWindow, minimized: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            inst.send_command(RustinoCommand::SetMinimized(minimized != 0));
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_maximized(instance: *mut RustinoWindow, maximized: i32) {
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
pub unsafe extern "C" fn rustino_set_fullscreen(instance: *mut RustinoWindow, fullscreen: i32) {
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
pub unsafe extern "C" fn rustino_set_visible(instance: *mut RustinoWindow, visible: i32) {
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
pub unsafe extern "C" fn rustino_set_focus(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            inst.send_command(RustinoCommand::SetFocus);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_position(instance: *mut RustinoWindow, x: i32, y: i32) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            if !inst.send_command(RustinoCommand::SetPosition(x, y)) {
                inst.config.position = Some((x, y));
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_min_size(instance: *mut RustinoWindow, width: i32, height: i32) {
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
pub unsafe extern "C" fn rustino_set_max_size(instance: *mut RustinoWindow, width: i32, height: i32) {
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
pub unsafe extern "C" fn rustino_is_minimized(instance: *mut RustinoWindow) -> i32 {
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
pub unsafe extern "C" fn rustino_is_maximized(instance: *mut RustinoWindow) -> i32 {
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
pub unsafe extern "C" fn rustino_is_fullscreen(instance: *mut RustinoWindow) -> i32 {
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
pub unsafe extern "C" fn rustino_get_position(
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
pub unsafe extern "C" fn rustino_get_size(
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
pub unsafe extern "C" fn rustino_evaluate_script(instance: *mut RustinoWindow, js: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            if let Some(s) = unsafe { util::cstr_to_string(js) } {
                inst.send_command(RustinoCommand::EvaluateScript(s));
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_send_web_message(instance: *mut RustinoWindow, msg: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            if let Some(m) = unsafe { util::cstr_to_string(msg) } {
                inst.send_command(RustinoCommand::SendWebMessage(m));
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_zoom(instance: *mut RustinoWindow, factor: f64) {
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

fn apply_dialog_common(
    dialog: rfd::FileDialog,
    params: &commands::DialogParams,
) -> rfd::FileDialog {
    let mut d = dialog;
    if let Some(ref title) = params.title {
        d = d.set_title(title);
    }
    if let Some(ref path) = params.default_path {
        let p = std::path::Path::new(path);
        if p.is_dir() {
            d = d.set_directory(p);
        } else {
            if let Some(parent) = p.parent() {
                d = d.set_directory(parent);
            }
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                d = d.set_file_name(name);
            }
        }
    }
    for (name, exts) in &params.filters {
        let ext_refs: Vec<&str> = exts.iter().map(|s| s.as_str()).collect();
        d = d.add_filter(name, &ext_refs);
    }
    d
}

#[cfg(target_os = "windows")]
fn init_dialog_com() {
    unsafe {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
}

#[cfg(not(target_os = "windows"))]
#[allow(dead_code)]
fn init_dialog_com() {}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_show_open_file_dialog(
    _instance: *mut RustinoWindow,
    title: *const c_char,
    default_path: *const c_char,
    filters: *const c_char,
    multi_select: i32,
) -> *mut c_char {
    catch_unwind(|| {
        let params = commands::DialogParams {
            title: unsafe { util::cstr_to_string(title) },
            default_path: unsafe { util::cstr_to_string(default_path) },
            filters: parse_filters(filters),
            multi_select: multi_select != 0,
        };

        #[cfg(target_os = "macos")]
        {
            // On macOS, call directly on main thread to avoid deadlock
            // (rfd uses dispatch_sync which would deadlock if we spawn a thread)
            let d = apply_dialog_common(rfd::FileDialog::new(), &params);
            let result = if params.multi_select {
                d.pick_files().map(|paths| paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
            } else {
                d.pick_file().map(|p| vec![p.to_string_lossy().into_owned()])
            };
            let paths = result?;
            let joined = paths.join("\n");
            std::ffi::CString::new(joined).ok().map(|s| s.into_raw())
        }

        #[cfg(not(target_os = "macos"))]
        {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                init_dialog_com();
                let d = apply_dialog_common(rfd::FileDialog::new(), &params);
                let result = if params.multi_select {
                    d.pick_files().map(|paths| paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
                } else {
                    d.pick_file().map(|p| vec![p.to_string_lossy().into_owned()])
                };
                let _ = tx.send(result);
            });
            let paths = rx.recv().ok()??;
            let joined = paths.join("\n");
            std::ffi::CString::new(joined).ok().map(|s| s.into_raw())
        }
    })
    .ok()
    .flatten()
    .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_show_save_file_dialog(
    _instance: *mut RustinoWindow,
    title: *const c_char,
    default_path: *const c_char,
    filters: *const c_char,
) -> *mut c_char {
    catch_unwind(|| {
        let params = commands::DialogParams {
            title: unsafe { util::cstr_to_string(title) },
            default_path: unsafe { util::cstr_to_string(default_path) },
            filters: parse_filters(filters),
            multi_select: false,
        };

        #[cfg(target_os = "macos")]
        {
            // On macOS, call directly on main thread to avoid deadlock
            // (rfd uses dispatch_sync which would deadlock if we spawn a thread)
            let d = apply_dialog_common(rfd::FileDialog::new(), &params);
            let result = d.save_file().map(|p| p.to_string_lossy().into_owned());
            let path = result?;
            std::ffi::CString::new(path).ok().map(|s| s.into_raw())
        }

        #[cfg(not(target_os = "macos"))]
        {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                init_dialog_com();
                let d = apply_dialog_common(rfd::FileDialog::new(), &params);
                let result = d.save_file().map(|p| p.to_string_lossy().into_owned());
                let _ = tx.send(result);
            });
            let path = rx.recv().ok()??;
            std::ffi::CString::new(path).ok().map(|s| s.into_raw())
        }
    })
    .ok()
    .flatten()
    .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_show_select_folder_dialog(
    _instance: *mut RustinoWindow,
    title: *const c_char,
    default_path: *const c_char,
    multi_select: i32,
) -> *mut c_char {
    catch_unwind(|| {
        let params = commands::DialogParams {
            title: unsafe { util::cstr_to_string(title) },
            default_path: unsafe { util::cstr_to_string(default_path) },
            filters: Vec::new(),
            multi_select: multi_select != 0,
        };

        #[cfg(target_os = "macos")]
        {
            // On macOS, call directly on main thread to avoid deadlock
            // (rfd uses dispatch_sync which would deadlock if we spawn a thread)
            let d = apply_dialog_common(rfd::FileDialog::new(), &params);
            let result = if params.multi_select {
                d.pick_folders().map(|paths| paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
            } else {
                d.pick_folder().map(|p| vec![p.to_string_lossy().into_owned()])
            };
            let paths = result?;
            let joined = paths.join("\n");
            std::ffi::CString::new(joined).ok().map(|s| s.into_raw())
        }

        #[cfg(not(target_os = "macos"))]
        {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                init_dialog_com();
                let d = apply_dialog_common(rfd::FileDialog::new(), &params);
                let result = if params.multi_select {
                    d.pick_folders().map(|paths| paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
                } else {
                    d.pick_folder().map(|p| vec![p.to_string_lossy().into_owned()])
                };
                let _ = tx.send(result);
            });
            let paths = rx.recv().ok()??;
            let joined = paths.join("\n");
            std::ffi::CString::new(joined).ok().map(|s| s.into_raw())
        }
    })
    .ok()
    .flatten()
    .unwrap_or(std::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// Taskbar badge (post-run only)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_badge_count(
    instance: *mut RustinoWindow,
    count: i32,
    bg_r: u8,
    bg_g: u8,
    bg_b: u8,
    fg_r: u8,
    fg_g: u8,
    fg_b: u8,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            let badge = if count > 0 { Some(count as u32) } else { None };
            inst.send_command(RustinoCommand::SetBadgeCount {
                count: badge,
                bg_r,
                bg_g,
                bg_b,
                fg_r,
                fg_g,
                fg_b,
            });
        }
    });
}

// ---------------------------------------------------------------------------
// Monitor enumeration (post-run only)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_get_monitors(instance: *mut RustinoWindow) -> *mut c_char {
    catch_unwind(|| {
        let inst = unsafe { instance.as_ref() }?;
        let json = inst.state.load_monitors();
        if json.is_empty() { return None; }
        std::ffi::CString::new(json).ok().map(|s| s.into_raw())
    })
    .ok()
    .flatten()
    .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_get_current_monitor(instance: *mut RustinoWindow) -> *mut c_char {
    catch_unwind(|| {
        let inst = unsafe { instance.as_ref() }?;
        let json = inst.state.load_current_monitor();
        if json.is_empty() { return None; }
        std::ffi::CString::new(json).ok().map(|s| s.into_raw())
    })
    .ok()
    .flatten()
    .unwrap_or(std::ptr::null_mut())
}

// ---------------------------------------------------------------------------
// Menus (post-run only)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_menu(instance: *mut RustinoWindow, json: *const c_char) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            if let Some(j) = unsafe { util::cstr_to_string(json) } {
                inst.send_command(RustinoCommand::SetMenu(j));
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_remove_menu(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            inst.send_command(RustinoCommand::RemoveMenu);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_show_context_menu(
    instance: *mut RustinoWindow,
    json: *const c_char,
    x: f64,
    y: f64,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            if let Some(j) = unsafe { util::cstr_to_string(json) } {
                let pos = if x < 0.0 && y < 0.0 {
                    None
                } else {
                    Some((x, y))
                };
                inst.send_command(RustinoCommand::ShowContextMenu(j, pos));
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_tray_icon(
    instance: *mut RustinoWindow,
    icon_path: *const c_char,
    tooltip: *const c_char,
    menu_json: *const c_char,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            if let Some(path) = unsafe { util::cstr_to_string(icon_path) } {
                let params = commands::TrayParams {
                    icon_path: path,
                    tooltip: unsafe { util::cstr_to_string(tooltip) },
                    menu_json: unsafe { util::cstr_to_string(menu_json) },
                };
                inst.send_command(RustinoCommand::SetTrayIcon(params));
            }
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_remove_tray_icon(instance: *mut RustinoWindow) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_ref() } {
            inst.send_command(RustinoCommand::RemoveTrayIcon);
        }
    });
}

// ---------------------------------------------------------------------------
// Callback registration (pre-run only)
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_callback_context(
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
pub unsafe extern "C" fn rustino_set_closing_handler(
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
pub unsafe extern "C" fn rustino_set_closed_handler(
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
pub unsafe extern "C" fn rustino_set_resized_handler(
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
pub unsafe extern "C" fn rustino_set_moved_handler(
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
pub unsafe extern "C" fn rustino_set_focus_changed_handler(
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
pub unsafe extern "C" fn rustino_set_web_message_received_handler(
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
pub unsafe extern "C" fn rustino_set_page_load_handler(
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
pub unsafe extern "C" fn rustino_set_navigation_handler(
    instance: *mut RustinoWindow,
    handler: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> i32>,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.on_navigation = handler;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_menu_event_handler(
    instance: *mut RustinoWindow,
    handler: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.on_menu_item_clicked = handler;
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_set_tray_icon_event_handler(
    instance: *mut RustinoWindow,
    handler: Option<unsafe extern "C" fn(*mut c_void)>,
) {
    let _ = catch_unwind(|| {
        if let Some(inst) = unsafe { instance.as_mut() } {
            inst.callbacks.on_tray_icon_clicked = handler;
        }
    });
}

// ---------------------------------------------------------------------------
// Splashscreen (standalone — no event loop required)
// ---------------------------------------------------------------------------

/// # Safety
/// `image_path` must be a valid null-terminated C string pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_splash_create(
    image_path: *const c_char,
    width: i32,
    height: i32,
) -> *mut splash::SplashWindow {
    let path = match unsafe { util::cstr_to_string(image_path) } {
        Some(p) => p,
        None => return std::ptr::null_mut(),
    };
    let w = width.max(1) as u32;
    let h = height.max(1) as u32;

    match splash::SplashWindow::new(&path, w, h) {
        Ok(splash) => Box::into_raw(Box::new(splash)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// `splash` must be a valid pointer returned from `rustino_splash_create`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_splash_close(splash: *mut splash::SplashWindow) {
    if let Some(s) = unsafe { splash.as_ref() } {
        s.close();
    }
}

/// # Safety
/// `splash` must be a valid pointer returned from `rustino_splash_create`, or null.
/// After calling this, the pointer is invalid and must not be used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rustino_splash_dtor(splash: *mut splash::SplashWindow) {
    if !splash.is_null() {
        unsafe {
            drop(Box::from_raw(splash));
        }
    }
}
