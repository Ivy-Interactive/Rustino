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
use crate::commands::RustinoCommand;
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
        if let Ok(guard) = self.proxy.read()
            && let Some(proxy) = guard.as_ref()
        {
            return proxy.send_event(cmd).is_ok();
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

        if let Some(ref icon_path) = config.icon_file
            && let Some(ico) = icon::load_icon(icon_path)
        {
            window_builder = window_builder.with_window_icon(Some(ico));
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

            webview_builder = webview_builder.with_new_window_req_handler(move |url, _features| {
                handle_new_window_req(url, ctx, cb)
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

        update_monitor_cache(&window, &self.state);

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

        #[cfg(target_os = "macos")]
        {
            let default_menu = create_default_macos_menu();
            attach_menu_to_window(&default_menu, &window);
            current_menu = Some(default_menu);
        }

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
                            update_monitor_cache(&window, &state);
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
                        WindowEvent::KeyboardInput { .. } => {
                            // Forward keyboard events to the webview by not consuming them
                            // The webview's internal handler will process these events
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
            log_warning(config, "[rustino] Warning: SetWebSecurityEnabled(false) is only supported on Windows (WebView2). Ignored on this platform.");
        }
        if config.ignore_certificate_errors {
            log_warning(config, "[rustino] Warning: SetIgnoreCertificateErrorsEnabled(true) is only supported on Windows (WebView2). Ignored on this platform.");
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn log_warning(config: &WindowConfig, message: &str) {
    if let Some(callback) = config.log_callback {
        let c_message = std::ffi::CString::new(message).unwrap_or_default();
        unsafe {
            callback(config.log_context, 3, c_message.as_ptr());
        }
    } else if config.log_verbosity > 0 {
        eprintln!("{}", message);
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

fn update_monitor_cache(window: &tao::window::Window, state: &SharedState) {
    let primary = window.primary_monitor();
    let monitors: Vec<serde_json::Value> = window
        .available_monitors()
        .map(|m| {
            let pos = m.position();
            let size = m.size();
            let is_primary = primary
                .as_ref()
                .map_or(false, |p| p.name() == m.name() && p.position() == m.position());
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
    let monitors_json = serde_json::to_string(&monitors).unwrap_or_default();

    let current_json = if let Some(m) = window.current_monitor() {
        let pos = m.position();
        let size = m.size();
        let is_primary = primary
            .as_ref()
            .map_or(false, |p| p.name() == m.name() && p.position() == m.position());
        serde_json::to_string(&serde_json::json!({
            "name": m.name(),
            "x": pos.x,
            "y": pos.y,
            "width": size.width,
            "height": size.height,
            "scaleFactor": m.scale_factor(),
            "isPrimary": is_primary
        }))
        .unwrap_or_default()
    } else {
        String::new()
    };

    state.store_monitors(&monitors_json, &current_json);
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
        RustinoCommand::SetBadgeCount(count) => {
            set_badge_count(window, count);
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
        RustinoCommand::ShowOpenFileDialog(..) |
        RustinoCommand::ShowSaveFileDialog(..) |
        RustinoCommand::ShowSelectFolderDialog(..) => {}
        RustinoCommand::GetMonitors(tx) => {
            update_monitor_cache(window, state);
            let _ = tx.send(state.load_monitors());
        }
        RustinoCommand::GetCurrentMonitor(tx) => {
            update_monitor_cache(window, state);
            let _ = tx.send(state.load_current_monitor());
        }
        RustinoCommand::Close => return true,
    }
    false
}

// --- Menu platform helpers ---

#[cfg(target_os = "macos")]
pub(crate) fn create_default_macos_menu() -> muda::Menu {
    let default_menu = muda::Menu::new();
    
    let app_menu = muda::Submenu::new("App", true);
    let _ = app_menu.append(&muda::PredefinedMenuItem::about(None, None));
    let _ = app_menu.append(&muda::PredefinedMenuItem::separator());
    let _ = app_menu.append(&muda::PredefinedMenuItem::quit(None));
    let _ = default_menu.append(&app_menu);

    let edit_menu = muda::Submenu::new("Edit", true);
    let _ = edit_menu.append(&muda::PredefinedMenuItem::undo(None));
    let _ = edit_menu.append(&muda::PredefinedMenuItem::redo(None));
    let _ = edit_menu.append(&muda::PredefinedMenuItem::separator());
    let _ = edit_menu.append(&muda::PredefinedMenuItem::cut(None));
    let _ = edit_menu.append(&muda::PredefinedMenuItem::copy(None));
    let _ = edit_menu.append(&muda::PredefinedMenuItem::paste(None));
    let _ = edit_menu.append(&muda::PredefinedMenuItem::select_all(None));
    let _ = default_menu.append(&edit_menu);

    default_menu
}

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
        let _ = menu.init_for_gtk_window(_window.gtk_window(), None::<&gtk::Container>);
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
        use gtk::prelude::Cast;
        let _ = menu.show_context_menu_for_gtk_window(window.gtk_window().upcast_ref::<gtk::Window>(), position);
    }
}

// --- Taskbar badge ---

fn set_badge_count(_window: &tao::window::Window, count: Option<u32>) {
    #[cfg(target_os = "windows")]
    {
        set_badge_count_windows(_window, count);
    }
    #[cfg(target_os = "macos")]
    {
        set_badge_count_macos(count);
    }
}

#[cfg(target_os = "windows")]
fn set_badge_count_windows(window: &tao::window::Window, count: Option<u32>) {
    use tao::platform::windows::WindowExtWindows;
    use windows::Win32::UI::Shell::{ITaskbarList3, TaskbarList};
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::Foundation::HWND;

    unsafe {
        let Ok(taskbar): Result<ITaskbarList3, _> =
            CoCreateInstance(&TaskbarList, None, CLSCTX_ALL)
        else {
            return;
        };

        let hwnd = HWND(window.hwnd() as *mut std::ffi::c_void);

        match count {
            None | Some(0) => {
                let _ = taskbar.SetOverlayIcon(hwnd, HICON::default(), None);
            }
            Some(n) => {
                if let Some(icon) = create_badge_icon(n) {
                    let _ = taskbar.SetOverlayIcon(hwnd, icon, None);
                    let _ = DestroyIcon(icon);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn create_badge_icon(count: u32) -> Option<windows::Win32::UI::WindowsAndMessaging::HICON> {
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::Graphics::Gdi::*;

    let size: i32 = 16;
    let pixels = size * size;
    let mut rgba = vec![0u8; (pixels * 4) as usize];

    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;
    let r = cx - 0.5;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= r {
                let alpha = ((r - dist).min(1.0) * 255.0) as u8;
                let idx = ((y * size + x) * 4) as usize;
                rgba[idx] = 0x33;     // B
                rgba[idx + 1] = 0x33; // G
                rgba[idx + 2] = 0xEE; // R (red)
                rgba[idx + 3] = alpha; // A
            }
        }
    }

    let text = if count > 99 { "!".to_string() } else { count.to_string() };
    let glyphs = render_badge_text(&text);
    let gw = glyphs.first().map_or(0, |row| row.len()) as i32;
    let gh = glyphs.len() as i32;
    let ox = (size - gw) / 2;
    let oy = (size - gh) / 2;

    for (gy, row) in glyphs.iter().enumerate() {
        for (gx, &pixel) in row.iter().enumerate() {
            if pixel {
                let px = ox + gx as i32;
                let py = oy + gy as i32;
                if px >= 0 && px < size && py >= 0 && py < size {
                    let idx = ((py * size + px) * 4) as usize;
                    if rgba[idx + 3] > 0 {
                        rgba[idx] = 0xFF;
                        rgba[idx + 1] = 0xFF;
                        rgba[idx + 2] = 0xFF;
                    }
                }
            }
        }
    }

    unsafe {
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = size;
        bmi.bmiHeader.biHeight = -(size);
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;

        let hdc = CreateCompatibleDC(None);
        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let bmp = CreateDIBSection(Some(hdc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0).ok()?;

        std::ptr::copy_nonoverlapping(rgba.as_ptr(), bits as *mut u8, rgba.len());

        let mask = CreateBitmap(size, size, 1, 1, None);
        let ii = ICONINFO {
            fIcon: true.into(),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask,
            hbmColor: bmp,
        };
        let icon = CreateIconIndirect(&ii).ok();

        let _ = DeleteObject(bmp.into());
        let _ = DeleteObject(mask.into());
        let _ = DeleteDC(hdc);

        icon
    }
}

#[cfg(target_os = "windows")]
fn render_badge_text(text: &str) -> Vec<Vec<bool>> {
    const FONT: [([u8; 5], u8); 11] = [
        ([0b111, 0b101, 0b101, 0b101, 0b111], 3), // 0
        ([0b010, 0b110, 0b010, 0b010, 0b111], 3), // 1
        ([0b111, 0b001, 0b111, 0b100, 0b111], 3), // 2
        ([0b111, 0b001, 0b111, 0b001, 0b111], 3), // 3
        ([0b101, 0b101, 0b111, 0b001, 0b001], 3), // 4
        ([0b111, 0b100, 0b111, 0b001, 0b111], 3), // 5
        ([0b111, 0b100, 0b111, 0b101, 0b111], 3), // 6
        ([0b111, 0b001, 0b010, 0b010, 0b010], 3), // 7
        ([0b111, 0b101, 0b111, 0b101, 0b111], 3), // 8
        ([0b111, 0b101, 0b111, 0b001, 0b111], 3), // 9
        ([0b010, 0b101, 0b010, 0b000, 0b010], 3), // ! (for 100+)
    ];

    let chars: Vec<usize> = text
        .chars()
        .filter_map(|c| match c {
            '0'..='9' => Some((c as u8 - b'0') as usize),
            '!' => Some(10),
            _ => None,
        })
        .collect();

    if chars.is_empty() {
        return vec![];
    }

    let total_width: usize = chars.iter().map(|&i| FONT[i].1 as usize).sum::<usize>()
        + chars.len().saturating_sub(1);

    let mut rows = vec![vec![false; total_width]; 5];
    let mut x_offset = 0usize;

    for (ci, &idx) in chars.iter().enumerate() {
        let (glyph_rows, w) = FONT[idx];
        let w = w as usize;
        for (row, &bits) in glyph_rows.iter().enumerate() {
            for col in 0..w {
                let bit = (bits >> (w - 1 - col)) & 1;
                if bit == 1 {
                    rows[row][x_offset + col] = true;
                }
            }
        }
        x_offset += w;
        if ci < chars.len() - 1 {
            x_offset += 1; // 1px spacing between characters
        }
    }

    rows
}

#[cfg(target_os = "macos")]
fn set_badge_count_macos(count: Option<u32>) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;
    use objc2_foundation::NSString;

    let Some(mtm) = MainThreadMarker::new() else { return };
    let app = NSApplication::sharedApplication(mtm);
    let dock_tile = app.dockTile();
    match count {
        None | Some(0) => {
            dock_tile.setBadgeLabel(Some(&NSString::from_str("")));
        }
        Some(n) => {
            dock_tile.setBadgeLabel(Some(&NSString::from_str(&n.to_string())));
        }
    }
}

fn load_tray_icon(path: &str) -> Option<tray_icon::Icon> {
    let img = image::open(path).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    tray_icon::Icon::from_rgba(img.into_raw(), w, h).ok()
}
pub(crate) fn handle_new_window_req(
    url: String,
    ctx: *mut std::ffi::c_void,
    cb: unsafe extern "C" fn(*mut std::ffi::c_void, *const std::ffi::c_char) -> i32,
) -> wry::NewWindowResponse {
    if let Ok(cstr) = CString::new(url) {
        unsafe { cb(ctx, cstr.as_ptr()) };
    }
    wry::NewWindowResponse::Deny
}

#[cfg(test)]
mod tests {
    use super::handle_new_window_req;
    use std::ffi::CStr;
    use std::sync::atomic::{AtomicBool, Ordering};

    static CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);

    unsafe extern "C" fn mock_cb(_ctx: *mut std::ffi::c_void, url: *const std::ffi::c_char) -> i32 {
        let c_str = unsafe { CStr::from_ptr(url) };
        assert_eq!(c_str.to_str().unwrap(), "https://example.com");
        CALLBACK_CALLED.store(true, Ordering::SeqCst);
        0
    }

    #[test]
    fn test_handle_new_window_req() {
        CALLBACK_CALLED.store(false, Ordering::SeqCst);
        let resp = handle_new_window_req(
            "https://example.com".to_string(),
            std::ptr::null_mut(),
            mock_cb,
        );
        assert!(CALLBACK_CALLED.load(Ordering::SeqCst));
        match resp {
            wry::NewWindowResponse::Deny => {}
            _ => panic!("Expected Deny"),
        }
    }

    #[test]
    #[ignore = "muda::Menu can only be created on the main thread on macOS"]
    #[cfg(target_os = "macos")]
    fn test_create_default_macos_menu() {
        let menu = super::create_default_macos_menu();
        let items = menu.items();
        assert_eq!(items.len(), 2, "Menu should have exactly 2 submenus (App and Edit)");
    }
}
