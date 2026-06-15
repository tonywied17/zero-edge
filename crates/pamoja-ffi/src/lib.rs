//! The curated C ABI surface for the pamoja SDK.
//!
//! This crate exposes a small, hand-written `extern "C"` API over
//! [`pamoja_core`] and the capability crates so that languages without a native
//! Rust bridge - C, C++, and C#/.NET through P/Invoke - can drive the SDK. It is
//! deliberately the project's single auditable `unsafe` boundary: every raw
//! pointer is dereferenced here and nowhere else.
//!
//! The committed header `include/pamoja.h` is generated from this source by
//! `cbindgen` (see `build.rs`) and is drift-checked in CI, so the C contract can
//! never fall behind the Rust surface.
//!
//! # Conventions
//!
//! - Fallible calls return a [`PamojaStatus`] code. On any non-`Ok` result a
//!   human-readable message is stored for the calling thread and can be read with
//!   [`pamoja_last_error_message`].
//! - Handles are opaque, heap-allocated, and owned by the caller, who must release
//!   each with its matching `*_free` function.
//! - All strings crossing the boundary are UTF-8. Inputs are borrowed for the
//!   duration of the call; returned pointers document their own lifetime.

// This crate is the FFI boundary, so raw-pointer work is its entire purpose; the
// workspace `unsafe_code = "warn"` lint is therefore allowed here. Safety is kept
// reviewable by confining every `unsafe` block to this crate.
#![allow(unsafe_code)]

use std::cell::RefCell;
use std::ffi::{c_char, CString};
use std::ptr;
use std::sync::OnceLock;

use pamoja_core::Error;

#[cfg(feature = "mqtt")]
mod mqtt;

/// The result of a fallible pamoja call.
///
/// A return of [`PamojaStatus::Ok`] means success; any other value indicates a
/// failure whose description is available from [`pamoja_last_error_message`] on
/// the same thread.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PamojaStatus {
    /// The call succeeded.
    Ok = 0,
    /// A transport-level failure while connecting, sending, or receiving.
    Transport = 1,
    /// A device or peripheral input/output operation failed.
    Io = 2,
    /// A payload could not be encoded or decoded.
    Codec = 3,
    /// The operation targeted a resource that is closed or disconnected.
    Closed = 4,
    /// The requested capability is not compiled into this build.
    Unsupported = 5,
    /// An argument was null or otherwise invalid, for example non-UTF-8 text.
    InvalidArgument = 6,
    /// A failure that does not map onto a more specific status.
    Other = 7,
    /// A Rust panic was caught at the boundary; the call had no effect.
    Panic = 8,
}

impl PamojaStatus {
    /// Maps a core [`Error`] onto the matching status code.
    ///
    /// # Arguments
    ///
    /// * `error` - the error returned by a core or capability call.
    ///
    /// # Returns
    ///
    /// The [`PamojaStatus`] that classifies `error`.
    pub(crate) fn from_error(error: &Error) -> Self {
        match error {
            Error::Transport(_) => Self::Transport,
            Error::Io(_) => Self::Io,
            Error::Codec(_) => Self::Codec,
            Error::Closed => Self::Closed,
            Error::Unsupported(_) => Self::Unsupported,
            _ => Self::Other,
        }
    }
}

thread_local! {
    /// The most recent error message produced on this thread.
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

/// Records `message` as the calling thread's most recent error.
///
/// # Arguments
///
/// * `message` - the human-readable description to expose through
///   [`pamoja_last_error_message`]. Any interior null byte is replaced with a
///   generic message so the value always stores cleanly as a C string.
pub(crate) fn set_last_error(message: String) {
    let value =
        CString::new(message).unwrap_or_else(|_| CString::new("pamoja error").expect("static"));
    LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(value));
}

/// Returns the calling thread's most recent error message, or null if none.
///
/// # Returns
///
/// A pointer to a null-terminated UTF-8 string owned by the library, valid until
/// the next failing call on the same thread, or null if no error has been
/// recorded. The caller must not free it and should copy it before making another
/// pamoja call on this thread.
#[no_mangle]
pub extern "C" fn pamoja_last_error_message() -> *const c_char {
    LAST_ERROR.with(|slot| match &*slot.borrow() {
        Some(value) => value.as_ptr(),
        None => ptr::null(),
    })
}

/// The version of the native pamoja library.
static VERSION: OnceLock<CString> = OnceLock::new();

/// Returns the version string of the native pamoja library.
///
/// # Returns
///
/// A pointer to a static null-terminated UTF-8 string owned by the library. The
/// caller must not free it; it is valid for the lifetime of the process.
#[no_mangle]
pub extern "C" fn pamoja_version() -> *const c_char {
    VERSION
        .get_or_init(|| CString::new(env!("CARGO_PKG_VERSION")).expect("version has no null byte"))
        .as_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_maps_each_error_variant() {
        assert!(matches!(
            PamojaStatus::from_error(&Error::Transport("x".into())),
            PamojaStatus::Transport
        ));
        assert!(matches!(
            PamojaStatus::from_error(&Error::Io("x".into())),
            PamojaStatus::Io
        ));
        assert!(matches!(
            PamojaStatus::from_error(&Error::Codec("x".into())),
            PamojaStatus::Codec
        ));
        assert!(matches!(
            PamojaStatus::from_error(&Error::Closed),
            PamojaStatus::Closed
        ));
        assert!(matches!(
            PamojaStatus::from_error(&Error::Unsupported("mqtt")),
            PamojaStatus::Unsupported
        ));
    }

    #[test]
    fn version_is_a_non_empty_c_string() {
        let ptr = pamoja_version();
        assert!(!ptr.is_null());
        // Safety: `pamoja_version` returns a valid static C string.
        let version = unsafe { std::ffi::CStr::from_ptr(ptr) };
        assert_eq!(version.to_str().expect("utf-8"), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn last_error_round_trips_on_this_thread() {
        set_last_error("transport error: boom".to_owned());
        let ptr = pamoja_last_error_message();
        assert!(!ptr.is_null());
        // Safety: a message was just recorded on this thread.
        let message = unsafe { std::ffi::CStr::from_ptr(ptr) };
        assert_eq!(message.to_str().expect("utf-8"), "transport error: boom");
    }
}
