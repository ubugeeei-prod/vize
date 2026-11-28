//! Win32 console calls shared by raw mode, the panic hook and the
//! console-control handler.
//!
//! Each function wraps exactly one Win32 call. None of them allocates or takes
//! a lock (`io::Error::last_os_error` stores only the error code), so the
//! emergency paths can call them.

use std::{io, ptr};

use windows_sys::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::WriteFile,
    System::Console::{GetConsoleMode, GetStdHandle, STD_HANDLE, SetConsoleMode},
};

/// Return the process standard handle for `selector`
/// (`STD_INPUT_HANDLE` or `STD_OUTPUT_HANDLE`).
pub(super) fn std_handle(selector: STD_HANDLE) -> io::Result<HANDLE> {
    // SAFETY: `GetStdHandle` (Win32 processenv.h) takes the selector by value
    // and dereferences no pointer. It returns the handle, null when the
    // process has none, or `INVALID_HANDLE_VALUE` with the last error set.
    let handle = unsafe { GetStdHandle(selector) };
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    Ok(handle)
}

/// Read the console mode of `handle`.
pub(super) fn console_mode(handle: HANDLE) -> io::Result<u32> {
    let mut mode = 0_u32;
    // SAFETY: `GetConsoleMode` (Win32 consoleapi.h) writes one `DWORD` through
    // its second argument, which points to the live local `mode`. A handle
    // that is not a console handle makes the call fail; it reads no memory of
    // ours.
    if unsafe { GetConsoleMode(handle, &raw mut mode) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(mode)
    }
}

/// Set the console mode of `handle`.
pub(super) fn set_console_mode(handle: HANDLE, mode: u32) -> io::Result<()> {
    // SAFETY: `SetConsoleMode` (Win32 consoleapi.h) takes both arguments by
    // value and dereferences no pointer.
    if unsafe { SetConsoleMode(handle, mode) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Write a prefix of `bytes` (at most `u32::MAX` of them) synchronously and
/// return how many bytes Windows reports written.
pub(super) fn write_file(handle: HANDLE, bytes: &[u8]) -> io::Result<usize> {
    let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    let mut written = 0_u32;
    // SAFETY: `WriteFile` (Win32 fileapi.h) reads `len` bytes from the buffer,
    // and `len <= bytes.len()`, so the read stays inside the borrowed slice for
    // the whole call. It writes the count through the third argument, which
    // points to the live local `written`. A null `OVERLAPPED` pointer requests
    // synchronous I/O, so no pointer outlives the call.
    let ok = unsafe {
        WriteFile(
            handle,
            bytes.as_ptr(),
            len,
            &raw mut written,
            ptr::null_mut(),
        )
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(written as usize)
}

#[cfg(test)]
mod tests;
