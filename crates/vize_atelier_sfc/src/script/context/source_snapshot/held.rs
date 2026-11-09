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

/// Handles one snapshot may hold: a transform needs a few, and a host burst
/// sharing one snapshot must stay far below the descriptor limit.
#[cfg(target_os = "linux")]
const MAX_HELD: usize = 64;

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

    pub(super) fn hold(&self, file: std::fs::File) {
        if let Some(handles) = &self.handles
            && let Ok(mut handles) = handles.lock()
            && handles.len() < MAX_HELD
        {
            handles.push(file);
        }
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
