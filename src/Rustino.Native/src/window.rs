use std::collections::HashMap;
use std::ffi::CString;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};

use muda::MenuId;
use tao::dpi::{PhysicalPosition, PhysicalSize};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::window::WindowBuilder;
#[cfg(target_os = "windows")]
use tao::platform::windows::EventLoopBuilderExtWindows;
use wry::WebViewBuilder;

#[cfg(target_os = "windows")]
use std::sync::Mutex as StdMutex;

use crate::callbacks::RustinoCallbacks;
use crate::commands::{DialogParams, RustinoCommand};
use crate::config::WindowConfig;
use crate::icon;
use crate::menu;
use crate::state::SharedState;

pub struct RustinoWindow {
    pub config: WindowConfig,
    pub callbacks: RustinoCallbacks,
    pub proxy: RwLock<Option<EventLoopProxy<RustinoCommand>>>,
    pub state: Arc<SharedState>,
}

impl RustinoWindow {
    pub fn new(config: WindowConfig) -> Self {
        let state = Arc::new(SharedState::new(config.width, config.height));
        Self {
            config,
            callbacks: RustinoCallbacks::default(),
            proxy: RwLock::new(None),
            state,
        }
    }

    pub fn send_command(&self, cmd: RustinoCommand) -> bool {
        if let Ok(guard) = self.proxy.read() {
            if let Some(proxy) = guard.as_ref() {
                return proxy.send_event(cmd).is_ok();
            }
        }
        false
    }

    pub fn run(&mut self) {
        let config = std::mem::take(&mut self.config);
        let callbacks = self.callbacks;

        configure_webview2_args(&config);

        let mut builder = EventLoopBuilder::<RustinoCommand>::with_user_event();
        #[cfg(target_os = "windows")]
        builder.with_any_thread(true);
        let mut event_loop = builder.build();
        *self.proxy.write().unwrap() = Some(event_loop.create_proxy());

        // --- Build window ---
        let mut window_builder = WindowBuilder::new()
            .with_title(&config.title)
            .with_resizable(config.resizable)
            .with_always_on_top(config.topmost)
            .with_decorations(config.decorations)
            .with_visible(config.visible)
            .with_maximized(config.maximized);

        if config.transparent {
            window_builder = window_builder.with_transparent(true);
        }

        if !config.use_os_default_size {
            window_builder =
                window_builder.with_inner_size(PhysicalSize::new(config.width, config.height));
        }

        if let Some((w, h)) = config.min_size {
            window_builder = window_builder.with_min_inner_size(PhysicalSize::new(w, h));
        }

        if let Some((w, h)) = config.max_size {
            window_builder = window_builder.with_max_inner_size(PhysicalSize::new(w, h));
        }

        if let Some(ref icon_path) = config.icon_file {
            if let Some(ico) = icon::load_icon(icon_path) {
                window_builder = window_builder.with_window_icon(Some(ico));
            }
        }

        if let Some(color) = config.background_color {
            window_builder = window_builder.with_background_color(color);
        }

        let window = window_builder
            .build(&event_loop)
            .expect("failed to build window");

        if config.center {
            center_window(&window);
        }

        if let Some((x, y)) = config.position {
            window.set_outer_position(PhysicalPosition::new(x, y));
        }

        if config.fullscreen {
            window.set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)));
        }

        // --- Build webview ---
        let mut web_context = config
            .user_data_folder
            .as_ref()
            .map(|p| wry::WebContext::new(Some(std::path::PathBuf::from(p))));

        let mut webview_builder = match web_context {
            Some(ref mut ctx) => WebViewBuilder::new_with_web_context(ctx),
            None => WebViewBuilder::new(),
        };

        if config.devtools_enabled {
            webview_builder = webview_builder.with_devtools(true);
        }

        if config.clipboard_enabled {
            webview_builder = webview_builder.with_clipboard(true);
        }

        if config.transparent {
            webview_builder = webview_builder.with_transparent(true);
        }

        if let Some(color) = config.background_color {
            webview_builder = webview_builder.with_background_color(color);
        }

        if let Some(ref ua) = config.user_agent {
            webview_builder = webview_builder.with_user_agent(ua);
        }

        webview_builder = webview_builder.with_autoplay(config.media_autoplay);

        if config.zoom_hotkeys {
            webview_builder = webview_builder.with_hotkeys_zoom(true);
        }

        for script in &config.initialization_scripts {
            webview_builder = webview_builder.with_initialization_script(script);
        }

        // IPC handler: JS → Rust
        let ctx = callbacks.context;
        if let Some(cb) = callbacks.on_web_message {
            webview_builder = webview_builder.with_ipc_handler(move |req: wry::http::Request<String>| {
                if let Ok(cstr) = CString::new(req.into_body()) {
                    unsafe { cb(ctx, cstr.as_ptr()) };
                }
            });
        }

        // Navigation handler
        if let Some(cb) = callbacks.on_navigation {
            webview_builder = webview_builder.with_navigation_handler(move |url| {
                match CString::new(url) {
                    Ok(cstr) => unsafe { cb(ctx, cstr.as_ptr()) == 0 },
                    Err(_) => true,
                }
            });
        }

        // Page load handler
        if let Some(cb) = callbacks.on_page_load {
            webview_builder =
                webview_builder.with_on_page_load_handler(move |event, url| {
                    let event_code = match event {
                        wry::PageLoadEvent::Started => 0,
                        wry::PageLoadEvent::Finished => 1,
                    };
                    if let Ok(cstr) = CString::new(url) {
                        unsafe { cb(ctx, event_code, cstr.as_ptr()) };
                    }
                });
        }

        if let Some(ref url) = config.start_url {
            webview_builder = webview_builder.with_url(url);
        } else if let Some(ref html) = config.start_html {
            webview_builder = webview_builder.with_html(html);
        }

        let webview = webview_builder
            .build(&window)
            .expect("failed to build webview");

        // Initialize shared state from actual window
        let size = window.inner_size();
        let pos = window
            .outer_position()
            .unwrap_or(PhysicalPosition::new(0, 0));
        self.state.store_size(size.width, size.height);
        self.state.store_position(pos.x, pos.y);
        self.state
            .is_maximized
            .store(window.is_maximized(), Ordering::Release);
        self.state
            .is_fullscreen
            .store(config.fullscreen, Ordering::Release);
        self.state
            .is_visible
            .store(config.visible, Ordering::Release);

        let state = Arc::clone(&self.state);

        // Wire menu events → EventLoopProxy
        let menu_id_map: Arc<std::sync::Mutex<HashMap<MenuId, String>>> =
            Arc::new(std::sync::Mutex::new(HashMap::new()));
        {
            let proxy = event_loop.create_proxy();
            let map = Arc::clone(&menu_id_map);
            muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
                if let Ok(guard) = map.lock() {
                    if let Some(id) = guard.get(&event.id) {
                        let _ = proxy.send_event(RustinoCommand::MenuEventFired(id.clone()));
                    }
                }
            }));
        }

        // Wire tray icon events → EventLoopProxy
        {
            let proxy = event_loop.create_proxy();
            tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
                if let tray_icon::TrayIconEvent::Click { .. } = event {
                    let _ = proxy.send_event(RustinoCommand::TrayIconClicked);
                }
            }));
        }

        let mut current_menu: Option<muda::Menu> = None;
        let mut tray: Option<tray_icon::TrayIcon> = None;

        event_loop.run_return(move |event, _, control_flow| {
            if *control_flow != ControlFlow::Exit {
                *control_flow = ControlFlow::Wait;
            }

            match event {
                Event::UserEvent(cmd) => {
                    if dispatch_command(
                        cmd,
                        &window,
                        &webview,
                        &state,
                        callbacks,
                        &menu_id_map,
                        &mut current_menu,
                        &mut tray,
                    ) {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::WindowEvent {
                    event: ref win_event, ..
                } => {
                    match win_event {
                        WindowEvent::CloseRequested => {
                            if let Some(cb) = callbacks.on_closing {
                                if unsafe { cb(callbacks.context) } != 0 {
                                    return;
                                }
                            }
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(size) => {
                            state.store_size(size.width, size.height);
                            state
                                .is_maximized
                                .store(window.is_maximized(), Ordering::Release);
                            if let Some(cb) = callbacks.on_resized {
                                unsafe {
                                    cb(callbacks.context, size.width as i32, size.height as i32)
                                };
                            }
                        }
                        WindowEvent::Moved(pos) => {
                            state.store_position(pos.x, pos.y);
                            if let Some(cb) = callbacks.on_moved {
                                unsafe { cb(callbacks.context, pos.x, pos.y) };
                            }
                        }
                        WindowEvent::Focused(focused) => {
                            state.is_focused.store(*focused, Ordering::Release);
                            if let Some(cb) = callbacks.on_focus_changed {
                                unsafe {
                                    cb(callbacks.context, if *focused { 1 } else { 0 })
                                };
                            }
                        }
                        _ => {}
                    }
                }
                Event::LoopDestroyed => {
                    if let Some(cb) = callbacks.on_closed {
                        unsafe { cb(callbacks.context) };
                    }
                }
                _ => {}
            }
        });

        muda::MenuEvent::set_event_handler(None::<Box<dyn Fn(muda::MenuEvent) + Send + Sync>>);
        tray_icon::TrayIconEvent::set_event_handler(
            None::<Box<dyn Fn(tray_icon::TrayIconEvent) + Send + Sync>>,
        );
        *self.proxy.write().unwrap() = None;
    }
}

fn configure_webview2_args(config: &WindowConfig) {
    #[cfg(target_os = "windows")]
    {
        if !config.web_security_enabled {
            append_webview2_arg("--disable-web-security");
        }
        if config.ignore_certificate_errors {
            append_webview2_arg("--ignore-certificate-errors");
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if !config.web_security_enabled {
            eprintln!("[rustino] Warning: SetWebSecurityEnabled(false) is only supported on Windows (WebView2). Ignored on this platform.");
        }
        if config.ignore_certificate_errors {
            eprintln!("[rustino] Warning: SetIgnoreCertificateErrorsEnabled(true) is only supported on Windows (WebView2). Ignored on this platform.");
        }
    }
}

#[cfg(target_os = "windows")]
fn append_webview2_arg(arg: &str) {
    static LOCK: StdMutex<()> = StdMutex::new(());
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let existing =
        std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").unwrap_or_default();
    if !existing.contains(arg) {
        let new_val = if existing.is_empty() {
            arg.to_string()
        } else {
            format!("{existing} {arg}")
        };
        unsafe {
            std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", &new_val);
        }
    }
}

fn center_window(window: &tao::window::Window) {
    if let Some(monitor) = window.current_monitor() {
        let monitor_size = monitor.size();
        let monitor_pos = monitor.position();
        let window_size = window.outer_size();
        let x = monitor_pos.x + ((monitor_size.width as i32 - window_size.width as i32) / 2);
        let y = monitor_pos.y + ((monitor_size.height as i32 - window_size.height as i32) / 2);
        window.set_outer_position(PhysicalPosition::new(x, y));
    }
}

fn dispatch_command(
    cmd: RustinoCommand,
    window: &tao::window::Window,
    webview: &wry::WebView,
    state: &SharedState,
    callbacks: RustinoCallbacks,
    menu_id_map: &Arc<std::sync::Mutex<HashMap<MenuId, String>>>,
    current_menu: &mut Option<muda::Menu>,
    tray: &mut Option<tray_icon::TrayIcon>,
) -> bool {
    match cmd {
        RustinoCommand::SetTitle(title) => window.set_title(&title),
        RustinoCommand::SetSize(w, h) => {
            window.set_inner_size(PhysicalSize::new(w, h));
        }
        RustinoCommand::SetMinimized(v) => {
            window.set_minimized(v);
            state.is_minimized.store(v, Ordering::Release);
        }
        RustinoCommand::SetMaximized(v) => {
            window.set_maximized(v);
            state.is_maximized.store(v, Ordering::Release);
        }
        RustinoCommand::SetFullscreen(v) => {
            if v {
                window.set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)));
            } else {
                window.set_fullscreen(None);
            }
            state.is_fullscreen.store(v, Ordering::Release);
        }
        RustinoCommand::SetVisible(v) => {
            window.set_visible(v);
            state.is_visible.store(v, Ordering::Release);
        }
        RustinoCommand::SetFocus => window.set_focus(),
        RustinoCommand::SetDecorations(v) => window.set_decorations(v),
        RustinoCommand::SetPosition(x, y) => {
            window.set_outer_position(PhysicalPosition::new(x, y));
        }
        RustinoCommand::Center => center_window(window),
        RustinoCommand::SetMinSize(size) => {
            window.set_min_inner_size(size.map(|(w, h)| PhysicalSize::new(w, h)));
        }
        RustinoCommand::SetMaxSize(size) => {
            window.set_max_inner_size(size.map(|(w, h)| PhysicalSize::new(w, h)));
        }
        RustinoCommand::SetResizable(v) => window.set_resizable(v),
        RustinoCommand::SetTopmost(v) => window.set_always_on_top(v),
        RustinoCommand::SetIconFile(path) => {
            if let Some(ico) = icon::load_icon(&path) {
                window.set_window_icon(Some(ico));
            }
        }
        RustinoCommand::EvaluateScript(js) => {
            let _ = webview.evaluate_script(&js);
        }
        RustinoCommand::SendWebMessage(msg) => {
            let mut escaped = String::with_capacity(msg.len() + 16);
            for ch in msg.chars() {
                match ch {
                    '\\' => escaped.push_str("\\\\"),
                    '\'' => escaped.push_str("\\'"),
                    '"' => escaped.push_str("\\\""),
                    '\n' => escaped.push_str("\\n"),
                    '\r' => escaped.push_str("\\r"),
                    '\t' => escaped.push_str("\\t"),
                    '\0' => escaped.push_str("\\0"),
                    '\u{0008}' => escaped.push_str("\\b"),
                    '\u{000C}' => escaped.push_str("\\f"),
                    '\u{2028}' => escaped.push_str("\\u2028"),
                    '\u{2029}' => escaped.push_str("\\u2029"),
                    '<' => escaped.push_str("\\x3c"),
                    _ => escaped.push(ch),
                }
            }
            let js = format!(
                "window.dispatchEvent(new MessageEvent('message',{{data:'{escaped}'}}));"
            );
            let _ = webview.evaluate_script(&js);
        }
        RustinoCommand::LoadUrl(url) => {
            let _ = webview.load_url(&url);
        }
        RustinoCommand::LoadHtml(html) => {
            let _ = webview.load_html(&html);
        }
        RustinoCommand::SetZoom(factor) => {
            let _ = webview.zoom(factor);
        }
        RustinoCommand::SetBackgroundColor(r, g, b, a) => {
            let _ = webview.set_background_color((r, g, b, a));
        }
        RustinoCommand::SetMenu(json) => {
            if let Some(old) = current_menu.take() {
                remove_menu_from_window(&old, window);
            }
            if let Some(built) = menu::build_menu(&json) {
                attach_menu_to_window(&built.menu, window);
                if let Ok(mut map) = menu_id_map.lock() {
                    map.extend(built.id_map);
                }
                *current_menu = Some(built.menu);
            }
        }
        RustinoCommand::RemoveMenu => {
            if let Some(old) = current_menu.take() {
                remove_menu_from_window(&old, window);
            }
        }
        RustinoCommand::ShowContextMenu(json, pos) => {
            if let Some(built) = menu::build_menu(&json) {
                if let Ok(mut map) = menu_id_map.lock() {
                    map.extend(built.id_map);
                }
                show_context_menu(&built.menu, window, pos);
            }
        }
        RustinoCommand::SetTrayIcon(params) => {
            *tray = None;
            if let Some(ico) = load_tray_icon(&params.icon_path) {
                let mut builder = tray_icon::TrayIconBuilder::new().with_icon(ico);
                if let Some(ref tooltip) = params.tooltip {
                    builder = builder.with_tooltip(tooltip);
                }
                if let Some(ref menu_json) = params.menu_json {
                    if let Some(built) = menu::build_menu(menu_json) {
                        if let Ok(mut map) = menu_id_map.lock() {
                            map.extend(built.id_map);
                        }
                        builder = builder.with_menu(Box::new(built.menu));
                    }
                }
                *tray = builder.build().ok();
            }
        }
        RustinoCommand::RemoveTrayIcon => {
            *tray = None;
        }
        RustinoCommand::MenuEventFired(id) => {
            if let Some(cb) = callbacks.on_menu_item_clicked {
                if let Ok(cstr) = CString::new(id.as_str()) {
                    unsafe { cb(callbacks.context, cstr.as_ptr()) };
                }
            }
        }
        RustinoCommand::TrayIconClicked => {
            if let Some(cb) = callbacks.on_tray_icon_clicked {
                unsafe { cb(callbacks.context) };
            }
        }
        RustinoCommand::ShowOpenFileDialog(params, tx) => {
            let result = run_open_file_dialog(window, &params);
            let _ = tx.send(result);
        }
        RustinoCommand::ShowSaveFileDialog(params, tx) => {
            let result = run_save_file_dialog(window, &params);
            let _ = tx.send(result);
        }
        RustinoCommand::ShowSelectFolderDialog(params, tx) => {
            let result = run_select_folder_dialog(window, &params);
            let _ = tx.send(result);
        }
        RustinoCommand::GetMonitors(tx) => {
            let primary = window.primary_monitor();
            let monitors: Vec<serde_json::Value> = window
                .available_monitors()
                .map(|m| {
                    let pos = m.position();
                    let size = m.size();
                    let is_primary = primary.as_ref().map_or(false, |p| p.name() == m.name() && p.position() == m.position());
                    serde_json::json!({
                        "name": m.name(),
                        "x": pos.x,
                        "y": pos.y,
                        "width": size.width,
                        "height": size.height,
                        "scaleFactor": m.scale_factor(),
                        "isPrimary": is_primary
                    })
                })
                .collect();
            let _ = tx.send(serde_json::to_string(&monitors).unwrap_or_default());
        }
        RustinoCommand::GetCurrentMonitor(tx) => {
            let primary = window.primary_monitor();
            let json = if let Some(m) = window.current_monitor() {
                let pos = m.position();
                let size = m.size();
                let is_primary = primary.as_ref().map_or(false, |p| p.name() == m.name() && p.position() == m.position());
                serde_json::to_string(&serde_json::json!({
                    "name": m.name(),
                    "x": pos.x,
                    "y": pos.y,
                    "width": size.width,
                    "height": size.height,
                    "scaleFactor": m.scale_factor(),
                    "isPrimary": is_primary
                })).unwrap_or_default()
            } else {
                String::new()
            };
            let _ = tx.send(json);
        }
        RustinoCommand::Close => return true,
    }
    false
}

fn apply_dialog_common(
    dialog: rfd::FileDialog,
    window: &tao::window::Window,
    params: &DialogParams,
) -> rfd::FileDialog {
    let mut d = dialog.set_parent(window);
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

fn run_open_file_dialog(
    window: &tao::window::Window,
    params: &DialogParams,
) -> Option<Vec<String>> {
    let d = apply_dialog_common(rfd::FileDialog::new(), window, params);
    if params.multi_select {
        d.pick_files()
            .map(|paths| paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
    } else {
        d.pick_file()
            .map(|p| vec![p.to_string_lossy().into_owned()])
    }
}

fn run_save_file_dialog(
    window: &tao::window::Window,
    params: &DialogParams,
) -> Option<String> {
    let d = apply_dialog_common(rfd::FileDialog::new(), window, params);
    d.save_file().map(|p| p.to_string_lossy().into_owned())
}

fn run_select_folder_dialog(
    window: &tao::window::Window,
    params: &DialogParams,
) -> Option<Vec<String>> {
    let d = apply_dialog_common(rfd::FileDialog::new(), window, params);
    if params.multi_select {
        d.pick_folders()
            .map(|paths| paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
    } else {
        d.pick_folder()
            .map(|p| vec![p.to_string_lossy().into_owned()])
    }
}

// --- Menu platform helpers ---

fn attach_menu_to_window(menu: &muda::Menu, _window: &tao::window::Window) {
    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::WindowExtWindows;
        unsafe { let _ = menu.init_for_hwnd(_window.hwnd() as _); }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = menu.init_for_nsapp();
    }
    #[cfg(target_os = "linux")]
    {
        use tao::platform::unix::WindowExtUnix;
        let _ = menu.init_for_gtk_window(_window.gtk_window(), None);
    }
}

fn remove_menu_from_window(menu: &muda::Menu, _window: &tao::window::Window) {
    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::WindowExtWindows;
        unsafe { let _ = menu.remove_for_hwnd(_window.hwnd() as _); }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = menu.remove_for_nsapp();
    }
    #[cfg(target_os = "linux")]
    {
        use tao::platform::unix::WindowExtUnix;
        let _ = menu.remove_for_gtk_window(_window.gtk_window());
    }
}

fn show_context_menu(
    menu: &muda::Menu,
    window: &tao::window::Window,
    pos: Option<(f64, f64)>,
) {
    use muda::ContextMenu;
    let position = pos.map(|(x, y)| muda::dpi::Position::Physical(muda::dpi::PhysicalPosition::new(x as i32, y as i32)));
    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::WindowExtWindows;
        let _ = unsafe { menu.show_context_menu_for_hwnd(window.hwnd() as _, position) };
    }
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::WindowExtMacOS;
        let _ = unsafe { menu.show_context_menu_for_nsview(window.ns_view() as _, position) };
    }
    #[cfg(target_os = "linux")]
    {
        use tao::platform::unix::WindowExtUnix;
        let _ = menu.show_context_menu_for_gtk_window(window.gtk_window(), position);
    }
}

fn load_tray_icon(path: &str) -> Option<tray_icon::Icon> {
    let img = image::open(path).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    tray_icon::Icon::from_rgba(img.into_raw(), w, h).ok()
}
