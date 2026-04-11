use std::os::raw::c_char;

#[repr(C)]
pub struct RustinoInitParams {
    pub title: *const c_char,
    pub icon_file: *const c_char,
    pub width: i32,
    pub height: i32,
    pub center_on_initialize: i32,
    pub use_os_default_size: i32,
    pub resizable: i32,
    pub topmost: i32,
    pub devtools_enabled: i32,
    pub clipboard_enabled: i32,
    pub ignore_certificate_errors: i32,
    pub web_security_enabled: i32,
    pub log_verbosity: i32,
}

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
    pub topmost: bool,
    pub center: bool,
    pub use_os_default_size: bool,
    pub devtools_enabled: bool,
    pub clipboard_enabled: bool,
    pub ignore_certificate_errors: bool,
    pub web_security_enabled: bool,
    pub icon_file: Option<String>,
    pub start_url: Option<String>,
    pub start_html: Option<String>,
    pub log_verbosity: i32,

    pub transparent: bool,
    pub decorations: bool,
    pub visible: bool,
    pub maximized: bool,
    pub fullscreen: bool,
    pub position: Option<(i32, i32)>,
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
    pub background_color: Option<(u8, u8, u8, u8)>,
    pub user_agent: Option<String>,
    pub user_data_folder: Option<String>,
    pub media_autoplay: bool,
    pub zoom_hotkeys: bool,
    pub initialization_scripts: Vec<String>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            width: 800,
            height: 600,
            resizable: true,
            topmost: false,
            center: false,
            use_os_default_size: false,
            devtools_enabled: false,
            clipboard_enabled: false,
            ignore_certificate_errors: false,
            web_security_enabled: true,
            icon_file: None,
            start_url: None,
            start_html: None,
            log_verbosity: 0,
            transparent: false,
            decorations: true,
            visible: true,
            maximized: false,
            fullscreen: false,
            position: None,
            min_size: None,
            max_size: None,
            background_color: None,
            user_agent: None,
            user_data_folder: None,
            media_autoplay: true,
            zoom_hotkeys: false,
            initialization_scripts: Vec::new(),
        }
    }
}

impl WindowConfig {
    pub fn from_params(params: &RustinoInitParams) -> Self {
        use crate::util::cstr_to_string;

        Self {
            title: unsafe { cstr_to_string(params.title) }
                .unwrap_or_else(|| "Rustino Window".to_string()),
            width: if params.width > 0 { params.width as u32 } else { 800 },
            height: if params.height > 0 { params.height as u32 } else { 600 },
            resizable: params.resizable != 0,
            topmost: params.topmost != 0,
            center: params.center_on_initialize != 0,
            use_os_default_size: params.use_os_default_size != 0,
            devtools_enabled: params.devtools_enabled != 0,
            clipboard_enabled: params.clipboard_enabled != 0,
            ignore_certificate_errors: params.ignore_certificate_errors != 0,
            web_security_enabled: params.web_security_enabled != 0,
            icon_file: unsafe { cstr_to_string(params.icon_file) },
            start_url: None,
            start_html: None,
            log_verbosity: params.log_verbosity,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn make_params(title: Option<&str>) -> (RustinoInitParams, Vec<CString>) {
        let mut keep = Vec::new();
        let title_ptr = match title {
            Some(t) => {
                let cs = CString::new(t).unwrap();
                let ptr = cs.as_ptr();
                keep.push(cs);
                ptr
            }
            None => std::ptr::null(),
        };
        let params = RustinoInitParams {
            title: title_ptr,
            icon_file: std::ptr::null(),
            width: 1024,
            height: 768,
            center_on_initialize: 1,
            use_os_default_size: 0,
            resizable: 1,
            topmost: 0,
            devtools_enabled: 1,
            clipboard_enabled: 0,
            ignore_certificate_errors: 0,
            web_security_enabled: 1,
            log_verbosity: 2,
        };
        (params, keep)
    }

    #[test]
    fn from_params_maps_fields() {
        let (params, _keep) = make_params(Some("Test Window"));
        let config = WindowConfig::from_params(&params);
        assert_eq!(config.title, "Test Window");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert!(config.center);
        assert!(!config.use_os_default_size);
        assert!(config.resizable);
        assert!(!config.topmost);
        assert!(config.devtools_enabled);
        assert!(!config.clipboard_enabled);
        assert!(!config.ignore_certificate_errors);
        assert!(config.web_security_enabled);
        assert_eq!(config.log_verbosity, 2);
    }

    #[test]
    fn from_params_null_title_uses_default() {
        let (params, _keep) = make_params(None);
        let config = WindowConfig::from_params(&params);
        assert_eq!(config.title, "Rustino Window");
    }

    #[test]
    fn from_params_zero_dimensions_use_defaults() {
        let (mut params, _keep) = make_params(Some("x"));
        params.width = 0;
        params.height = -1;
        let config = WindowConfig::from_params(&params);
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
    }

    #[test]
    fn from_params_extended_fields_are_defaults() {
        let (params, _keep) = make_params(Some("x"));
        let config = WindowConfig::from_params(&params);
        assert!(!config.transparent);
        assert!(config.decorations);
        assert!(config.visible);
        assert!(!config.maximized);
        assert!(!config.fullscreen);
        assert!(config.position.is_none());
        assert!(config.min_size.is_none());
        assert!(config.max_size.is_none());
        assert!(config.background_color.is_none());
        assert!(config.user_agent.is_none());
        assert!(config.user_data_folder.is_none());
        assert!(config.media_autoplay);
        assert!(!config.zoom_hotkeys);
        assert!(config.initialization_scripts.is_empty());
    }

    #[test]
    fn default_has_sensible_values() {
        let config = WindowConfig::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(config.resizable);
        assert!(config.web_security_enabled);
        assert!(config.decorations);
        assert!(config.visible);
        assert!(config.media_autoplay);
    }
}
