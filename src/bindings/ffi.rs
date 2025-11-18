//! C Foreign Function Interface (FFI) for JCL
//!
//! This module provides a C-compatible API for embedding JCL in other languages.
//!
//! # Safety
//!
//! All functions in this module are marked as `unsafe` because they deal with raw pointers
//! and C memory management. Callers must ensure:
//! - Pointers are valid and non-null (unless explicitly allowed)
//! - Strings are null-terminated UTF-8
//! - Memory is properly freed using `jcl_free_string`

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::{formatter, linter, parser, docgen};

/// Opaque handle to a JCL parse result
#[repr(C)]
pub struct JclModule {
    _private: [u8; 0],
}

/// Result of a JCL operation
#[repr(C)]
pub struct JclResult {
    pub success: bool,
    pub value: *mut c_char,  // Caller must free with jcl_free_string
    pub error: *mut c_char,  // Caller must free with jcl_free_string
}

impl JclResult {
    fn success(value: String) -> Self {
        Self {
            success: true,
            value: CString::new(value).unwrap().into_raw(),
            error: ptr::null_mut(),
        }
    }

    fn error(error: String) -> Self {
        Self {
            success: false,
            value: ptr::null_mut(),
            error: CString::new(error).unwrap().into_raw(),
        }
    }
}

/// Initialize JCL library (currently a no-op, but may be used for future initialization)
///
/// # Returns
/// 0 on success, non-zero on error
#[no_mangle]
pub extern "C" fn jcl_init() -> i32 {
    0
}

/// Parse JCL source code
///
/// # Arguments
/// - `source`: Null-terminated UTF-8 string containing JCL source code
///
/// # Returns
/// JclResult with parse status. Caller must free result with jcl_free_result.
///
/// # Safety
/// `source` must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn jcl_parse(source: *const c_char) -> JclResult {
    if source.is_null() {
        return JclResult::error("Null source pointer".to_string());
    }

    let c_str = match CStr::from_ptr(source).to_str() {
        Ok(s) => s,
        Err(e) => return JclResult::error(format!("Invalid UTF-8: {}", e)),
    };

    match crate::parse_str(c_str) {
        Ok(_module) => JclResult::success("Parse successful".to_string()),
        Err(e) => JclResult::error(format!("Parse error: {}", e)),
    }
}

/// Format JCL source code
///
/// # Arguments
/// - `source`: Null-terminated UTF-8 string containing JCL source code
///
/// # Returns
/// JclResult with formatted code. Caller must free result with jcl_free_result.
///
/// # Safety
/// `source` must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn jcl_format(source: *const c_char) -> JclResult {
    if source.is_null() {
        return JclResult::error("Null source pointer".to_string());
    }

    let c_str = match CStr::from_ptr(source).to_str() {
        Ok(s) => s,
        Err(e) => return JclResult::error(format!("Invalid UTF-8: {}", e)),
    };

    match crate::parse_str(c_str) {
        Ok(module) => match formatter::format(&module) {
            Ok(formatted) => JclResult::success(formatted),
            Err(e) => JclResult::error(format!("Format error: {}", e)),
        },
        Err(e) => JclResult::error(format!("Parse error: {}", e)),
    }
}

/// Lint JCL source code
///
/// # Arguments
/// - `source`: Null-terminated UTF-8 string containing JCL source code
///
/// # Returns
/// JclResult with lint issues as JSON. Caller must free result with jcl_free_result.
///
/// # Safety
/// `source` must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn jcl_lint(source: *const c_char) -> JclResult {
    if source.is_null() {
        return JclResult::error("Null source pointer".to_string());
    }

    let c_str = match CStr::from_ptr(source).to_str() {
        Ok(s) => s,
        Err(e) => return JclResult::error(format!("Invalid UTF-8: {}", e)),
    };

    match crate::parse_str(c_str) {
        Ok(module) => match linter::lint(&module) {
            Ok(issues) => {
                if issues.is_empty() {
                    JclResult::success("No issues found".to_string())
                } else {
                    match serde_json::to_string_pretty(&issues) {
                        Ok(json) => JclResult::success(json),
                        Err(e) => JclResult::error(format!("JSON serialization error: {}", e)),
                    }
                }
            }
            Err(e) => JclResult::error(format!("Linter error: {}", e)),
        },
        Err(e) => JclResult::error(format!("Parse error: {}", e)),
    }
}

/// Generate documentation from JCL source code
///
/// # Arguments
/// - `source`: Null-terminated UTF-8 string containing JCL source code
/// - `module_name`: Null-terminated UTF-8 string for the module name
///
/// # Returns
/// JclResult with Markdown documentation. Caller must free result with jcl_free_result.
///
/// # Safety
/// Both `source` and `module_name` must be valid null-terminated UTF-8 strings
#[no_mangle]
pub unsafe extern "C" fn jcl_generate_docs(
    source: *const c_char,
    module_name: *const c_char,
) -> JclResult {
    if source.is_null() {
        return JclResult::error("Null source pointer".to_string());
    }
    if module_name.is_null() {
        return JclResult::error("Null module_name pointer".to_string());
    }

    let source_str = match CStr::from_ptr(source).to_str() {
        Ok(s) => s,
        Err(e) => return JclResult::error(format!("Invalid UTF-8 in source: {}", e)),
    };

    let module_name_str = match CStr::from_ptr(module_name).to_str() {
        Ok(s) => s,
        Err(e) => return JclResult::error(format!("Invalid UTF-8 in module_name: {}", e)),
    };

    match crate::parse_str(source_str) {
        Ok(module) => match docgen::generate(&module) {
            Ok(doc) => {
                let markdown = docgen::format_markdown(&doc, module_name_str);
                JclResult::success(markdown)
            }
            Err(e) => JclResult::error(format!("Doc generation error: {}", e)),
        },
        Err(e) => JclResult::error(format!("Parse error: {}", e)),
    }
}

/// Get JCL version
///
/// # Returns
/// Pointer to a static null-terminated UTF-8 string. Do NOT free this pointer.
#[no_mangle]
pub extern "C" fn jcl_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Free a string returned by JCL functions
///
/// # Arguments
/// - `ptr`: Pointer to string returned by JCL function
///
/// # Safety
/// - `ptr` must have been allocated by JCL
/// - `ptr` must not be used after this call
/// - `ptr` must not be a static string (like the one from jcl_version)
/// - This function is safe to call with null pointers (no-op)
#[no_mangle]
pub unsafe extern "C" fn jcl_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Free a JclResult returned by JCL functions
///
/// # Arguments
/// - `result`: Pointer to JclResult to free
///
/// # Safety
/// - `result` must point to a valid JclResult
/// - `result` and its contents must not be used after this call
/// - This function is safe to call with null pointer (no-op)
#[no_mangle]
pub unsafe extern "C" fn jcl_free_result(result: *mut JclResult) {
    if result.is_null() {
        return;
    }

    let result = &*result;

    if !result.value.is_null() {
        jcl_free_string(result.value);
    }

    if !result.error.is_null() {
        jcl_free_string(result.error);
    }

    // Don't free the JclResult itself - it's typically stack-allocated in C
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_jcl_init() {
        assert_eq!(jcl_init(), 0);
    }

    #[test]
    fn test_jcl_parse_valid() {
        let source = CString::new("x = 42").unwrap();
        let result = unsafe { jcl_parse(source.as_ptr()) };

        assert!(result.success);
        assert!(!result.value.is_null());
        assert!(result.error.is_null());

        unsafe {
            jcl_free_result(&result as *const _ as *mut _);
        }
    }

    #[test]
    fn test_jcl_parse_invalid() {
        let source = CString::new("x = ").unwrap();
        let result = unsafe { jcl_parse(source.as_ptr()) };

        assert!(!result.success);
        assert!(result.value.is_null());
        assert!(!result.error.is_null());

        unsafe {
            jcl_free_result(&result as *const _ as *mut _);
        }
    }

    #[test]
    fn test_jcl_format() {
        let source = CString::new("x=42").unwrap();
        let result = unsafe { jcl_format(source.as_ptr()) };

        assert!(result.success);
        assert!(!result.value.is_null());

        unsafe {
            let formatted = CStr::from_ptr(result.value).to_str().unwrap();
            assert!(formatted.contains("x = 42"));
            jcl_free_result(&result as *const _ as *mut _);
        }
    }

    #[test]
    fn test_jcl_version() {
        let version_ptr = jcl_version();
        assert!(!version_ptr.is_null());

        unsafe {
            let version = CStr::from_ptr(version_ptr).to_str().unwrap();
            assert!(!version.is_empty());
        }
    }
}
