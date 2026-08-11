#[cfg(unix)]
pub(crate) fn create_fifo(path: &std::path::Path) {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};

    let encoded = CString::new(path.as_os_str().as_bytes()).unwrap();
    // SAFETY: `encoded` is a NUL-terminated filesystem path and mode is valid.
    let result = unsafe { libc::mkfifo(encoded.as_ptr(), 0o600) };
    assert_eq!(
        result,
        0,
        "failed to create FIFO {}: {}",
        path.display(),
        std::io::Error::last_os_error()
    );
}
