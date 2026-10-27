//! Fresh Linux directory enumeration without allocating an owned name for each
//! entry. No directory contents or file descriptors survive a lookup.

use std::{
    ffi::OsStr,
    fs::OpenOptions,
    io,
    os::{fd::AsRawFd, unix::ffi::OsStrExt, unix::fs::OpenOptionsExt},
    path::Path,
};

use super::DirectoryMarkers;

pub(super) fn directory_markers(directory: &Path) -> io::Result<DirectoryMarkers> {
    // O_DIRECTORY keeps a path replaced by a FIFO/device from blocking open.
    // Rust's OpenOptions also applies O_CLOEXEC. Symlinks retain read_dir's
    // existing follow policy; permission checks are performed on every lookup.
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY)
        .open(directory)?;
    let mut markers = DirectoryMarkers::default();
    let mut buffer = [0_u8; 32 * 1024];
    loop {
        vize_l0::micro_count!("fs.getdents64.calls", 1);
        // SAFETY: the live descriptor belongs to `file`; the writable buffer
        // remains exclusively borrowed and valid for its full supplied length.
        // getdents64 writes at most that length and does not retain its pointer.
        let length = unsafe {
            libc::syscall(
                libc::SYS_getdents64,
                file.as_raw_fd(),
                buffer.as_mut_ptr(),
                buffer.len(),
            )
        };
        if length < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            // Unsupported/denied syscalls and filesystem errors retry through
            // the established standard reader rather than skipping a directory.
            return Err(error);
        }
        let length = length as usize;
        if length == 0 {
            return Ok(markers);
        }
        let bytes = buffer
            .get(..length)
            .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
        vize_l0::micro_count!("fs.getdents64.bytes", length);
        if observe_records(bytes, &mut markers)? {
            return Ok(markers);
        }
    }
}

// The Linux UAPI has u64 inode, i64 offset, u16 record length, u8 type, then
// NUL-terminated name bytes (getdents(2)). Decode bytes instead of casting to
// structs: input records need no Rust alignment and no borrowed reference can
// outlive the current buffer. Neither inode nor d_type affects marker policy.
fn observe_records(mut bytes: &[u8], markers: &mut DirectoryMarkers) -> io::Result<bool> {
    const NAME_OFFSET: usize = 19;
    while !bytes.is_empty() {
        let encoded_length: [u8; 2] = bytes
            .get(16..18)
            .and_then(|field| field.try_into().ok())
            .ok_or(io::ErrorKind::InvalidData)?;
        let length = usize::from(u16::from_ne_bytes(encoded_length));
        let Some((record, remaining)) = bytes
            .split_at_checked(length)
            .filter(|(record, _)| record.len() > NAME_OFFSET)
        else {
            return Err(io::ErrorKind::InvalidData.into());
        };
        let name = record
            .get(NAME_OFFSET..)
            .ok_or(io::ErrorKind::InvalidData)?;
        let Some(terminator) = memchr::memchr(0, name) else {
            return Err(io::ErrorKind::InvalidData.into());
        };
        let name = name.get(..terminator).ok_or(io::ErrorKind::InvalidData)?;
        if name.is_empty() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        // Unlike std::fs::read_dir, getdents64 also includes these two entries.
        if name != b"." && name != b".." {
            vize_l0::micro_count!("fs.read_dir.entries", 1);
            if markers.observe(OsStr::from_bytes(name)) {
                return Ok(true);
            }
        }
        bytes = remaining;
    }
    Ok(false)
}

#[cfg(test)]
mod tests;
