//! Module text kept across requests, keyed by the file's stamp, on Linux.
//!
//! An editor's transforms re-read the same imported modules on every request,
//! and on a WSL2 Windows mount each read costs an open, a `statx`, two reads
//! and a close: five 9P round trips. A request's snapshot already `fstat`s
//! every module its probes find (to tell a regular file from a link), so the
//! stamp `(dev, ino, size, mtime, ctime)` of that call validates text read by
//! an earlier request with no further call.
//!
//! Why an equal stamp means equal text: every write, truncation or rename
//! onto the path changes the inode, the size, or the modification and change
//! times, which the kernel sets from the clock at the moment of the change
//! (`ctime` cannot be set from user space). Text is only kept when both times
//! were at least [`SETTLED`] older than the read, so any later change lands in
//! a later timestamp tick than the kept one, even on coarse clocks (ext4 ticks
//! with the jiffy, NTFS behind drvfs with the Windows timer, up to about
//! 16 ms). Where that does not hold, the text is read again: a stamp that
//! differs, a file changed within the window, or a snapshot that did not
//! probe the file. What remains outside the guarantee is a filesystem clock
//! running more than [`SETTLED`] behind this process's clock (a drvfs host
//! clock against a WSL2 guest clock that jumped ahead), combined with two
//! same-size writes inside one tick around the read.

use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, SystemTime};

use vize_carton::FxHashMap;

/// How long both file times must predate a read before its text is kept.
const SETTLED: Duration = Duration::from_secs(2);

/// Kept modules; past either limit the memo starts over.
const MAX_MODULES: usize = 4096;
const MAX_BYTES: usize = 64 * 1024 * 1024;

/// What a write, truncation or rename changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Stamp {
    device: u64,
    inode: u64,
    size: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}

impl Stamp {
    pub(super) fn of(metadata: &std::fs::Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            size: metadata.size(),
            modified: (metadata.mtime(), metadata.mtime_nsec()),
            changed: (metadata.ctime(), metadata.ctime_nsec()),
        }
    }

    /// Both times at least `margin` before `read_at`.
    fn settled(&self, read_at: SystemTime, margin: Duration) -> bool {
        let settled = |(seconds, nanos): (i64, i64)| {
            u64::try_from(seconds)
                .ok()
                .zip(u64::try_from(nanos).ok())
                .and_then(|(seconds, nanos)| {
                    SystemTime::UNIX_EPOCH
                        .checked_add(Duration::from_secs(seconds))?
                        .checked_add(Duration::from_nanos(nanos))?
                        .checked_add(margin)
                })
                .is_some_and(|settled_at| settled_at <= read_at)
        };
        settled(self.modified) && settled(self.changed)
    }
}

#[derive(Default)]
struct Memo {
    modules: FxHashMap<PathBuf, (Stamp, Arc<str>)>,
    bytes: usize,
}

static MEMO: LazyLock<Mutex<Memo>> = LazyLock::new(Mutex::default);

/// The text of `path`, whose probe saw `stamp`: kept text when the stamp
/// matches, otherwise read with as many calls as `std::fs::read_to_string`.
pub(super) fn read(path: &Path, stamp: Stamp) -> Option<Arc<str>> {
    read_settled(path, stamp, SETTLED)
}

fn read_settled(path: &Path, stamp: Stamp, margin: Duration) -> Option<Arc<str>> {
    if let Ok(memo) = MEMO.lock()
        && let Some((kept, text)) = memo.modules.get(path)
        && *kept == stamp
    {
        return Some(Arc::clone(text));
    }
    let read_at = SystemTime::now();
    let mut file = std::fs::File::open(path).ok()?;
    let metadata = file.metadata().ok()?;
    let text = read_text(&mut file, metadata.len())?;
    let stamp = Stamp::of(&metadata);
    if stamp.settled(read_at, margin)
        && let Ok(mut memo) = MEMO.lock()
    {
        if memo.modules.len() >= MAX_MODULES || memo.bytes + text.len() > MAX_BYTES {
            *memo = Memo::default();
        }
        memo.bytes += text.len();
        if let Some((_, previous)) = memo
            .modules
            .insert(path.to_path_buf(), (stamp, Arc::clone(&text)))
        {
            memo.bytes = memo.bytes.saturating_sub(previous.len());
        }
    }
    Some(text)
}

/// The rest of `file` as UTF-8 with the size already known: one read of the
/// whole file and one at its end, without the `statx` and `lseek` that
/// `Read::read_to_string` on a `File` spends on a size hint.
fn read_text(file: &mut std::fs::File, size: u64) -> Option<Arc<str>> {
    let mut bytes = vec![0_u8; usize::try_from(size).ok()?.saturating_add(1)];
    let mut len = 0;
    loop {
        if len == bytes.len() {
            bytes.resize(bytes.len().saturating_mul(2), 0);
        }
        match file.read(bytes.get_mut(len..)?) {
            Ok(0) => break,
            Ok(read) => len += read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return None,
        }
    }
    bytes.truncate(len);
    std::str::from_utf8(&bytes).ok().map(Arc::from)
}

#[cfg(test)]
mod tests {
    use super::{MEMO, SETTLED, Stamp, read_settled};
    use std::time::Duration;

    #[test]
    fn kept_text_needs_the_probe_stamp_and_settled_times() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model.ts");
        let stamp = |path: &std::path::Path| Stamp::of(&std::fs::metadata(path).unwrap());
        let kept = |path: &std::path::Path| MEMO.lock().unwrap().modules.contains_key(path);
        std::fs::write(&path, "one").unwrap();
        // Times written just now are not settled: the text is read, not kept.
        assert_eq!(
            read_settled(&path, stamp(&path), SETTLED).as_deref(),
            Some("one")
        );
        assert!(!kept(&path));
        assert_eq!(
            read_settled(&path, stamp(&path), Duration::ZERO).as_deref(),
            Some("one")
        );
        assert!(kept(&path));
        // The kept text answers its stamp without opening the file.
        let first = stamp(&path);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            read_settled(&path, first, Duration::ZERO).as_deref(),
            Some("one")
        );
        // Another stamp reads the file again.
        std::fs::write(&path, "three").unwrap();
        assert_eq!(
            read_settled(&path, stamp(&path), Duration::ZERO).as_deref(),
            Some("three")
        );
        // Larger than the first read's buffer, and not UTF-8.
        let large = "x".repeat(100_000);
        std::fs::write(&path, &large).unwrap();
        assert_eq!(
            read_settled(&path, stamp(&path), SETTLED).as_deref(),
            Some(large.as_str())
        );
        std::fs::write(&path, [0xff_u8, 0xfe]).unwrap();
        assert_eq!(read_settled(&path, stamp(&path), SETTLED), None);
    }
}
