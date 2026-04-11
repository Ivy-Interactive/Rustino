use std::ffi::c_void;
use std::os::raw::c_char;

#[derive(Clone, Copy)]
pub struct RustinoCallbacks {
    pub context: *mut c_void,
    pub on_closing: Option<unsafe extern "C" fn(*mut c_void) -> i32>,
    pub on_closed: Option<unsafe extern "C" fn(*mut c_void)>,
    pub on_resized: Option<unsafe extern "C" fn(*mut c_void, i32, i32)>,
    pub on_moved: Option<unsafe extern "C" fn(*mut c_void, i32, i32)>,
    pub on_focus_changed: Option<unsafe extern "C" fn(*mut c_void, i32)>,
    pub on_web_message: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
    pub on_page_load: Option<unsafe extern "C" fn(*mut c_void, i32, *const c_char)>,
    pub on_navigation: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> i32>,
}

impl Default for RustinoCallbacks {
    fn default() -> Self {
        Self {
            context: std::ptr::null_mut(),
            on_closing: None,
            on_closed: None,
            on_resized: None,
            on_moved: None,
            on_focus_changed: None,
            on_web_message: None,
            on_page_load: None,
            on_navigation: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_all_none() {
        let cb = RustinoCallbacks::default();
        assert!(cb.context.is_null());
        assert!(cb.on_closing.is_none());
        assert!(cb.on_closed.is_none());
        assert!(cb.on_resized.is_none());
        assert!(cb.on_moved.is_none());
        assert!(cb.on_focus_changed.is_none());
        assert!(cb.on_web_message.is_none());
        assert!(cb.on_page_load.is_none());
        assert!(cb.on_navigation.is_none());
    }

    #[test]
    fn is_copy_and_clone() {
        let cb = RustinoCallbacks::default();
        let cb2 = cb;
        let _cb3 = cb2.clone();
        assert!(cb.context.is_null());
    }
}
