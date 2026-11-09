//! `O_PATH` handles a request's snapshot holds until it drops, on Linux.
//!
//! A 9P mount without a dentry cache (WSL2 drvfs, `cache=mmap`) drops every
//! dentry nothing references once a call returns, so the next call walks its
//! path again from the working directory: a round trip per component, and
//! another to release each. A snapshot that serves one request holds the
//! handle of each root, walked directory and found file it resolved, so later
//! calls below them walk only the names under them. A held entry is not
//! revalidated while the snapshot lives, which is the moment the snapshot
//! already stands for (it freezes canonical directories and hits as well);
//! the next snapshot walks again. The content mapper drops a snapshot after
//! its reply is written, so the releases leave the reply path. Long-lived
//! snapshots (an editor revision) hold nothing: a held directory could keep
//! a Windows program from removing it.

/// Handles one snapshot may hold: a transform needs a few.
#[cfg(target_os = "linux")]
const MAX_HELD: usize = 64;

/// Handles all snapshots may hold at once, far below the usual soft limit of
/// 1024 descriptors, so holding never starves a read of one.
#[cfg(target_os = "linux")]
const MAX_HELD_IN_PROCESS: usize = 256;

#[cfg(target_os = "linux")]
static HELD_IN_PROCESS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[derive(Debug, Default)]
pub(super) struct HeldHandles {
    /// `None` for snapshots that hold nothing.
    #[cfg(target_os = "linux")]
    handles: Option<std::sync::Mutex<Vec<std::fs::File>>>,
}

impl HeldHandles {
    pub(super) fn holding() -> Self {
        Self {
            #[cfg(target_os = "linux")]
            handles: Some(std::sync::Mutex::default()),
        }
    }
}

#[cfg(target_os = "linux")]
impl HeldHandles {
    /// Whether a probe should open a handle to hold rather than `lstat`.
    pub(super) fn enabled(&self) -> bool {
        self.handles.is_some()
    }

    /// Keep `file` open until the snapshot drops, within both limits;
    /// otherwise it closes now.
    pub(super) fn hold(&self, file: std::fs::File) {
        use std::sync::atomic::Ordering;
        let Some(Ok(mut handles)) = self.handles.as_ref().map(|handles| handles.lock()) else {
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
}

#[cfg(target_os = "linux")]
impl Drop for HeldHandles {
    fn drop(&mut self) {
        let held = self
            .handles
            .as_mut()
            .and_then(|handles| handles.get_mut().ok().map(|handles| handles.len()))
            .unwrap_or(0);
        HELD_IN_PROCESS.fetch_sub(held, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(all(test, target_os = "linux"))]
impl HeldHandles {
    pub(super) fn len(&self) -> usize {
        self.handles
            .as_ref()
            .and_then(|handles| handles.lock().ok().map(|handles| handles.len()))
            .unwrap_or(0)
    }
}
