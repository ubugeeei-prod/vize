//! Fresh Linux directory enumeration without allocating an owned name for each
//! entry. No directory contents or file descriptors survive a lookup.

use std::{
    collections::{HashMap, HashSet},
    ffi::{CString, OsStr},
    fs::OpenOptions,
    io,
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::ffi::OsStrExt,
        unix::fs::OpenOptionsExt,
    },
    path::{Path, PathBuf},
    sync::Mutex,
};

use super::DirectoryMarkers;

const WATCH_MASK: u32 = libc::IN_CREATE
    | libc::IN_DELETE
    | libc::IN_MOVED_FROM
    | libc::IN_MOVED_TO
    | libc::IN_DELETE_SELF
    | libc::IN_MOVE_SELF
    | libc::IN_ONLYDIR;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DirectoryGeneration {
    token: u64,
    changes: u64,
    overflows: u64,
}

#[derive(Default)]
pub(super) struct DirectoryChangeTracker {
    state: Mutex<ChangeTrackerState>,
}

impl DirectoryChangeTracker {
    pub(super) fn generation(&self, directory: &Path) -> Option<DirectoryGeneration> {
        let mut state = self.state.lock().ok()?;
        state.drain();
        state.watch(directory)
    }

    pub(super) fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            *state = ChangeTrackerState::default();
        }
    }
}

struct ChangeTrackerState {
    descriptor: Option<OwnedFd>,
    paths: HashMap<PathBuf, i32>,
    watches: HashMap<i32, Watch>,
    next_token: u64,
    overflows: u64,
}

impl Default for ChangeTrackerState {
    fn default() -> Self {
        // SAFETY: inotify_init1 has no borrowed arguments. A successful return
        // transfers ownership of a new close-on-exec descriptor to this state.
        let descriptor = unsafe { libc::inotify_init1(libc::IN_NONBLOCK | libc::IN_CLOEXEC) };
        Self {
            descriptor: (descriptor >= 0).then(|| {
                // SAFETY: the non-negative descriptor was just created above
                // and has not been wrapped or transferred elsewhere.
                unsafe { OwnedFd::from_raw_fd(descriptor) }
            }),
            paths: HashMap::new(),
            watches: HashMap::new(),
            next_token: 1,
            overflows: 0,
        }
    }
}

struct Watch {
    token: u64,
    changes: u64,
    paths: HashSet<PathBuf>,
}

impl ChangeTrackerState {
    fn watch(&mut self, directory: &Path) -> Option<DirectoryGeneration> {
        if let Some(watch) = self
            .paths
            .get(directory)
            .and_then(|descriptor| self.watches.get(descriptor))
        {
            return Some(DirectoryGeneration {
                token: watch.token,
                changes: watch.changes,
                overflows: self.overflows,
            });
        }
        let descriptor = self.descriptor.as_ref()?.as_raw_fd();
        let encoded = CString::new(directory.as_os_str().as_bytes()).ok()?;
        if !supports_local_generations(&encoded) {
            return None;
        }
        // SAFETY: the tracker owns the live descriptor and `encoded` is a
        // NUL-terminated pathname for the duration of this call.
        vize_l0::micro_count!("fs.inotify.watch_add.calls", 1);
        let watch_descriptor =
            unsafe { libc::inotify_add_watch(descriptor, encoded.as_ptr(), WATCH_MASK) };
        if watch_descriptor < 0 {
            return None;
        }
        let path = directory.to_path_buf();
        self.paths.insert(path.clone(), watch_descriptor);
        if !self.watches.contains_key(&watch_descriptor) {
            let token = self.next_token;
            self.next_token = self.next_token.wrapping_add(1).max(1);
            self.watches.insert(
                watch_descriptor,
                Watch {
                    token,
                    changes: 0,
                    paths: HashSet::new(),
                },
            );
        }
        let watch = self.watches.get_mut(&watch_descriptor)?;
        watch.paths.insert(path);
        Some(DirectoryGeneration {
            token: watch.token,
            changes: watch.changes,
            overflows: self.overflows,
        })
    }

    fn drain(&mut self) {
        let Some(descriptor) = self.descriptor.as_ref().map(AsRawFd::as_raw_fd) else {
            return;
        };
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            // SAFETY: `buffer` is exclusively writable for its complete
            // supplied length and read does not retain the pointer.
            vize_l0::micro_count!("fs.inotify.read.calls", 1);
            let length =
                unsafe { libc::read(descriptor, buffer.as_mut_ptr().cast(), buffer.len()) };
            if length < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                if error.kind() != io::ErrorKind::WouldBlock {
                    self.disable();
                }
                return;
            }
            if length == 0 {
                return;
            }
            let Some(events) = buffer.get(..length as usize) else {
                self.disable();
                return;
            };
            if !self.observe_events(events) {
                self.disable();
                return;
            }
        }
    }

    fn observe_events(&mut self, mut bytes: &[u8]) -> bool {
        const HEADER: usize = 16;
        while !bytes.is_empty() {
            let Some(header) = bytes.get(..HEADER) else {
                return false;
            };
            let Some(watch_descriptor) = event_field(header, 0).map(i32::from_ne_bytes) else {
                return false;
            };
            let Some(mask) = event_field(header, 4).map(u32::from_ne_bytes) else {
                return false;
            };
            let Some(name_length) = event_field(header, 12).map(u32::from_ne_bytes) else {
                return false;
            };
            let name_length = name_length as usize;
            let Some((_, remaining)) = bytes.split_at_checked(HEADER + name_length) else {
                return false;
            };
            bytes = remaining;
            vize_l0::micro_count!("fs.inotify.events", 1);

            if mask & libc::IN_Q_OVERFLOW != 0 {
                vize_l0::micro_count!("fs.inotify.queue_overflows", 1);
                self.overflows = self.overflows.wrapping_add(1);
                continue;
            }
            if let Some(watch) = self.watches.get_mut(&watch_descriptor) {
                watch.changes = watch.changes.wrapping_add(1);
            }
            if mask & (libc::IN_IGNORED | libc::IN_DELETE_SELF | libc::IN_MOVE_SELF) != 0 {
                self.remove_watch(watch_descriptor);
            }
        }
        true
    }

    fn remove_watch(&mut self, descriptor: i32) {
        if let Some(inotify) = self.descriptor.as_ref() {
            // SAFETY: the tracker owns the inotify descriptor. Removing an
            // already ignored watch may fail with EINVAL and needs no retry.
            unsafe {
                libc::inotify_rm_watch(inotify.as_raw_fd(), descriptor);
            }
        }
        if let Some(watch) = self.watches.remove(&descriptor) {
            for path in watch.paths {
                self.paths.remove(&path);
            }
        }
    }

    fn disable(&mut self) {
        self.descriptor = None;
        self.paths.clear();
        self.watches.clear();
    }
}

fn event_field(bytes: &[u8], offset: usize) -> Option<[u8; 4]> {
    bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()
}

fn supports_local_generations(path: &std::ffi::CStr) -> bool {
    // Remote and user-space filesystems do not necessarily report mutations
    // made by another host/process through inotify. Keep those paths uncached.
    const EXT: u64 = 0x0000_EF53;
    const XFS: u64 = 0x5846_5342;
    const BTRFS: u64 = 0x9123_683E;
    const TMPFS: u64 = 0x0102_1994;
    const OVERLAY: u64 = 0x794C_7630;
    const ZFS: u64 = 0x2FC1_2FC1;
    const RAMFS: u64 = 0x8584_58F6;

    let mut storage = std::mem::MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: `path` is NUL-terminated and live for this call; on success the
    // kernel has initialized the complete statfs output structure.
    vize_l0::micro_count!("fs.statfs.calls", 1);
    if unsafe { libc::statfs(path.as_ptr(), storage.as_mut_ptr()) } != 0 {
        return false;
    }
    // SAFETY: guarded by the successful statfs call above.
    let filesystem = unsafe { storage.assume_init() };
    matches!(
        filesystem.f_type as u64,
        EXT | XFS | BTRFS | TMPFS | OVERLAY | ZFS | RAMFS
    )
}

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
