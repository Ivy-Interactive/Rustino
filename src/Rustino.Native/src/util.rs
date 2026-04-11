use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub unsafe fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(|s| s.to_string())
}

pub fn free_cstring(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn cstr_to_string_null_returns_none() {
        let result = unsafe { cstr_to_string(std::ptr::null()) };
        assert_eq!(result, None);
    }

    #[test]
    fn cstr_to_string_valid_utf8() {
        let s = CString::new("hello world").unwrap();
        let result = unsafe { cstr_to_string(s.as_ptr()) };
        assert_eq!(result, Some("hello world".to_string()));
    }

    #[test]
    fn cstr_to_string_empty_string() {
        let s = CString::new("").unwrap();
        let result = unsafe { cstr_to_string(s.as_ptr()) };
        assert_eq!(result, Some(String::new()));
    }

    #[test]
    fn cstr_to_string_unicode() {
        let s = CString::new("こんにちは 🌍").unwrap();
        let result = unsafe { cstr_to_string(s.as_ptr()) };
        assert_eq!(result, Some("こんにちは 🌍".to_string()));
    }

    #[test]
    fn free_cstring_null_is_safe() {
        free_cstring(std::ptr::null_mut());
    }

    #[test]
    fn free_cstring_valid_pointer() {
        let s = CString::new("test").unwrap();
        let ptr = s.into_raw();
        free_cstring(ptr);
    }
}
