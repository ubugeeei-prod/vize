//! Thread-local `errno` accessors for supported libc targets.

#[cfg(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "haiku",
    target_os = "illumos",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "nto",
    target_os = "openbsd",
    target_os = "solaris",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
))]
pub(super) const SUPPORTED: bool = true;
#[cfg(not(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "haiku",
    target_os = "illumos",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "nto",
    target_os = "openbsd",
    target_os = "solaris",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
)))]
pub(super) const SUPPORTED: bool = false;

#[cfg(target_os = "android")]
pub(super) unsafe fn location() -> *mut libc::c_int {
    // SAFETY: Android libc returns the calling thread's errno slot.
    unsafe { libc::__errno() }
}

#[cfg(any(target_os = "dragonfly", target_os = "linux"))]
pub(super) unsafe fn location() -> *mut libc::c_int {
    // SAFETY: these libc implementations return the calling thread's errno slot.
    unsafe { libc::__errno_location() }
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
))]
pub(super) unsafe fn location() -> *mut libc::c_int {
    // SAFETY: these libc implementations return the calling thread's errno slot.
    unsafe { libc::__error() }
}

#[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
pub(super) unsafe fn location() -> *mut libc::c_int {
    // SAFETY: these libc implementations return the calling thread's errno slot.
    unsafe { libc::__errno() }
}

#[cfg(any(target_os = "illumos", target_os = "solaris"))]
pub(super) unsafe fn location() -> *mut libc::c_int {
    // SAFETY: these libc implementations return the calling thread's errno slot.
    unsafe { libc::___errno() }
}

#[cfg(target_os = "haiku")]
pub(super) unsafe fn location() -> *mut libc::c_int {
    // SAFETY: Haiku libc returns the calling thread's errno slot.
    unsafe { libc::_errnop() }
}

#[cfg(target_os = "nto")]
pub(super) unsafe fn location() -> *mut libc::c_int {
    // SAFETY: QNX libc returns the calling thread's errno slot.
    unsafe { libc::__get_errno_ptr() }
}

#[cfg(not(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "haiku",
    target_os = "illumos",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "nto",
    target_os = "openbsd",
    target_os = "solaris",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos",
)))]
pub(super) unsafe fn location() -> *mut libc::c_int {
    std::ptr::null_mut()
}
