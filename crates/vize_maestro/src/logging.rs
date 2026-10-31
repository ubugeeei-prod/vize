//! Process logs live in the user's cache, never in an installed project.

use std::{
    ffi::OsString,
    fs::{File, OpenOptions, create_dir_all},
    path::PathBuf,
    sync::Once,
};
use tracing_subscriber::fmt::writer::MakeWriterExt;

fn log_path(explicit: Option<OsString>, cache: Option<PathBuf>) -> Option<PathBuf> {
    let path = match explicit {
        Some(path) => PathBuf::from(path),
        None => cache?.join("vize/logs/lsp.log"),
    };
    // An invalid override or missing user-cache directory uses stderr only.
    // Never reinterpret a relative path against an editor's project cwd.
    path.is_absolute().then_some(path)
}

fn open_log(path: PathBuf) -> Option<File> {
    create_dir_all(path.parent()?).ok()?;
    OpenOptions::new().create(true).append(true).open(path).ok()
}

pub(super) fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let file =
            log_path(std::env::var_os("VIZE_LSP_LOG_FILE"), dirs::cache_dir()).and_then(open_log);
        // Editors may close stderr immediately after shutdown. Keep internal
        // write-error reporting off so that cannot make logging panic.
        if let Some(file) = file {
            tracing_subscriber::fmt()
                .with_writer(file.and(std::io::stderr))
                .with_ansi(false)
                .log_internal_errors(false)
                .init();
        } else {
            tracing_subscriber::fmt()
                .with_writer(std::io::stderr)
                .with_ansi(false)
                .log_internal_errors(false)
                .init();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{log_path, open_log};
    use std::io::Write;

    #[test]
    fn default_log_uses_only_the_user_cache() {
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(
            log_path(None, Some(directory.path().to_path_buf())),
            Some(directory.path().join("vize/logs/lsp.log"))
        );
        assert_eq!(log_path(None, None), None);
        assert_eq!(log_path(None, Some("relative-cache".into())), None);
    }

    #[test]
    fn explicit_log_requires_an_absolute_path_and_never_falls_back() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("owned/session.log");
        let cache = Some(directory.path().join("unused-cache"));
        assert_eq!(
            log_path(Some(path.clone().into_os_string()), cache.clone()),
            Some(path)
        );
        assert_eq!(log_path(Some("relative.log".into()), cache.clone()), None);
        assert_eq!(log_path(Some("".into()), cache), None);
    }

    #[test]
    fn log_file_appends_without_replacing_existing_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("owned/session.log");
        open_log(path.clone())
            .unwrap()
            .write_all(b"first\n")
            .unwrap();
        open_log(path.clone())
            .unwrap()
            .write_all(b"second\n")
            .unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"first\nsecond\n");
        assert!(open_log(path.join("not-a-directory.log")).is_none());
        assert_eq!(std::fs::read(path).unwrap(), b"first\nsecond\n");
    }
}
