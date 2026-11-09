//! `O_PATH` handles a request's snapshot holds until it drops, on Linux.
//!
//! A 9P mount without a dentry cache (WSL2 drvfs, `cache=mmap`) drops every
//! dentry nothing references once a call returns, so the next call walks its
//! path again from the working directory: a round trip per component, and
//! another to release each. A snapshot that serves one request holds the
//! handle of its root and of each directory and file it then resolves, so
//! later calls below them walk only the names under them. A held entry is not
//! revalidated while the snapshot lives, which is the moment the snapshot
//! already stands for (it freezes canonical directories and hits as well);
//! the next snapshot walks again. The content mapper drops a snapshot after
//! its reply is written, so the releases leave the reply path.
//!
//! Long-lived snapshots (an editor revision) hold nothing: a held directory
//! could keep a Windows program from removing it. A request snapshot that a
//! host burst shares stops holding at its second root and goes back to
//! `stat` and listings shared per directory, which cost a burst less.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use vize_carton::FxHashMap;

use super::content_stamp::Stamp;

/// Handles one snapshot may hold: a transform needs a few.
const MAX_HELD: usize = 64;

/// Handles all snapshots may hold at once, far below the usual soft limit of
/// 1024 descriptors, so holding never starves a read of one.
const MAX_HELD_IN_PROCESS: usize = 256;

static HELD_IN_PROCESS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Default)]
pub(super) struct HeldHandles {
    /// `None` for snapshots that hold nothing.
    handles: Option<Mutex<Vec<std::fs::File>>>,
    first_root: OnceLock<PathBuf>,
    shared: AtomicBool,
    /// What each found file's probe `fstat` saw, which validates text an
    /// earlier request read.
    stamps: Mutex<FxHashMap<PathBuf, Stamp>>,
}

impl HeldHandles {
    pub(super) fn holding() -> Self {
        Self {
            handles: Some(Mutex::default()),
            first_root: OnceLock::new(),
            shared: AtomicBool::new(false),
            stamps: Mutex::default(),
        }
    }

    /// Whether this snapshot serves one request's root, so its lookups open
    /// handles to hold rather than `lstat` and list.
    pub(super) fn enabled(&self) -> bool {
        self.handles.is_some() && !self.shared.load(Ordering::Relaxed)
    }

    /// Record a root this snapshot resolves; `true` the first time, when its
    /// handle is worth holding.
    pub(super) fn note_root(&self, root: &Path) -> bool {
        if self.handles.is_none() {
            return false;
        }
        let mut first = false;
        let held = self.first_root.get_or_init(|| {
            first = true;
            root.to_path_buf()
        });
        if held != root {
            self.shared.store(true, Ordering::Relaxed);
        }
        first
    }

    /// Record what a found file's probe saw.
    pub(super) fn stamp(&self, path: &Path, metadata: &std::fs::Metadata) {
        if self.enabled()
            && let Ok(mut stamps) = self.stamps.lock()
        {
            stamps.insert(path.to_path_buf(), Stamp::of(metadata));
        }
    }

    pub(super) fn stamp_of(&self, path: &Path) -> Option<Stamp> {
        self.stamps.lock().ok()?.get(path).copied()
    }

    /// Keep `file` open until the snapshot drops, within both limits;
    /// otherwise it closes now.
    pub(super) fn hold(&self, file: std::fs::File) {
        if !self.enabled() {
            return;
        }
        let Some(Ok(mut handles)) = self.handles.as_ref().map(Mutex::lock) else {
            return;
        };
        if handles.len() >= MAX_HELD {
            return;
        }
        if HELD_IN_PROCESS.fetch_add(1, Ordering::Relaxed) >= MAX_HELD_IN_PROCESS {
            HELD_IN_PROCESS.fetch_sub(1, Ordering::Relaxed);
            return;
        }
        handles.push(file);
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.handles
            .as_ref()
            .and_then(|handles| handles.lock().ok().map(|handles| handles.len()))
            .unwrap_or(0)
    }
}

impl Drop for HeldHandles {
    fn drop(&mut self) {
        let held = self
            .handles
            .as_mut()
            .and_then(|handles| handles.get_mut().ok().map(|handles| handles.len()))
            .unwrap_or(0);
        HELD_IN_PROCESS.fetch_sub(held, Ordering::Relaxed);
    }
}
