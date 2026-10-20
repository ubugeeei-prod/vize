//! Private lifetime owner for editor Canon mirrors and their in-memory cache.

use std::fs::{File, OpenOptions, TryLockError};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, SystemTime};

use super::types::CorsaBridgeError;
use super::vue_dependencies_alias::{SessionCache, recover_lock};

/// One bridge/check-server session owns one private mirror namespace. Live
/// overlay bytes and the cache describing them must never be shared with an
/// independent native TypeScript session.
pub(crate) struct EditorMirrorSession {
    storage: Result<EditorStorage, vize_carton::String>,
    cache: Mutex<SessionCache>,
    /// The workspace preparation in flight, shared by equal requests.
    preparing: Mutex<Option<Preparation>>,
    #[expect(
        clippy::disallowed_types,
        reason = "immutable snapshots are shared by editor owners"
    )]
    compiler_configuration: Option<std::sync::Arc<crate::CompilerConfigurationSnapshot>>,
}

struct EditorStorage {
    _lifetime_lock: File,
    directory: tempfile::TempDir,
}

/// One preparation identity and the signal its worker thread resolves.
pub(in crate::corsa_bridge) type Preparation = (
    u64,
    futures::future::Shared<futures::channel::oneshot::Receiver<()>>,
);

impl EditorMirrorSession {
    pub(crate) fn new() -> Self {
        Self::with_compiler_configuration(None)
    }

    #[expect(
        clippy::disallowed_types,
        reason = "immutable snapshots are shared by editor owners"
    )]
    pub(crate) fn with_compiler_configuration(
        compiler_configuration: Option<std::sync::Arc<crate::CompilerConfigurationSnapshot>>,
    ) -> Self {
        let storage = create_storage().map_err(|error| {
            vize_carton::cstr!("failed to create private editor mirror storage: {error}")
        });
        Self {
            storage,
            cache: Mutex::new(SessionCache::default()),
            preparing: Mutex::new(None),
            compiler_configuration,
        }
    }

    #[expect(
        clippy::disallowed_types,
        reason = "immutable snapshots are shared by editor owners"
    )]
    pub(crate) fn compiler_configuration(
        &self,
    ) -> Option<&std::sync::Arc<crate::CompilerConfigurationSnapshot>> {
        self.compiler_configuration.as_ref()
    }

    pub(crate) fn root(&self) -> Result<&Path, CorsaBridgeError> {
        self.storage
            .as_ref()
            .map(|storage| storage.directory.path())
            .map_err(|error| CorsaBridgeError::CommunicationError(error.clone()))
    }

    pub(in crate::corsa_bridge) fn cache(&self) -> MutexGuard<'_, SessionCache> {
        recover_lock(&self.cache)
    }

    pub(in crate::corsa_bridge) fn preparing(&self) -> MutexGuard<'_, Option<Preparation>> {
        recover_lock(&self.preparing)
    }

    pub(in crate::corsa_bridge) fn trust_watched_disk_inputs(&self) {
        self.cache().trust_watched_disk_inputs();
    }

    pub(in crate::corsa_bridge) fn invalidate_disk_inputs(&self) {
        self.cache().invalidate_disk_inputs();
    }

    pub(crate) fn clear(&self) {
        self.cache().clear();
        let Ok(root) = self.root() else {
            return;
        };
        let Ok(entries) = std::fs::read_dir(root) else {
            return;
        };
        for entry in entries.flatten() {
            if entry.file_name() == ".session.lock" {
                continue;
            }
            let path = entry.path();
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                let _ = std::fs::remove_dir_all(path);
            } else {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

impl Default for EditorMirrorSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub(crate) fn fallback_editor_session() -> &'static EditorMirrorSession {
    thread_local! {
        static SESSION: &'static EditorMirrorSession =
            Box::leak(Box::new(EditorMirrorSession::new()));
    }
    SESSION.with(|session| *session)
}

fn create_storage() -> std::io::Result<EditorStorage> {
    let base = vize_carton::path::canonicalize_non_verbatim(&std::env::temp_dir())
        .join("vize-canon")
        .join("editor")
        .join("sessions");
    std::fs::create_dir_all(&base)?;
    let gc_lock = open_lock(&base.join(".gc.lock"))?;
    gc_lock.lock()?;
    reap_abandoned_sessions(&base);
    let directory = tempfile::Builder::new()
        .prefix("session-")
        .tempdir_in(base)?;
    let lifetime_lock = open_lock(&directory.path().join(".session.lock"))?;
    lifetime_lock.lock()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(EditorStorage {
        _lifetime_lock: lifetime_lock,
        directory,
    })
}

fn open_lock(path: &Path) -> std::io::Result<File> {
    OpenOptions::new()
        .create(true)
        .read(true)
        .truncate(false)
        .write(true)
        .open(path)
}

fn reap_abandoned_sessions(base: &Path) {
    const LEGACY_GRACE: Duration = Duration::from_secs(24 * 60 * 60);
    let Ok(entries) = std::fs::read_dir(base) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !entry.file_type().is_ok_and(|kind| kind.is_dir())
            || !entry.file_name().to_string_lossy().starts_with("session-")
        {
            continue;
        }
        let lock_path = path.join(".session.lock");
        let removable = if lock_path.exists() {
            match open_lock(&lock_path) {
                Ok(lock) => match lock.try_lock() {
                    Ok(()) => true,
                    Err(TryLockError::WouldBlock | TryLockError::Error(_)) => false,
                },
                Err(_) => false,
            }
        } else {
            legacy_session_expired(&path, LEGACY_GRACE)
        };
        if removable {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

fn legacy_session_expired(path: &Path, grace: Duration) -> bool {
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age >= grace)
}

#[cfg(test)]
mod tests {
    use super::{EditorMirrorSession, open_lock, reap_abandoned_sessions};

    #[test]
    fn sessions_own_distinct_private_roots() {
        let first = EditorMirrorSession::new();
        let second = EditorMirrorSession::new();
        assert_ne!(first.root().unwrap(), second.root().unwrap());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(first.root().unwrap())
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
        }

        let relative = std::path::Path::new("projects/same-config/src/Host.vue.ts");
        let first_host = first.root().unwrap().join(relative);
        let second_host = second.root().unwrap().join(relative);
        std::fs::create_dir_all(first_host.parent().unwrap()).unwrap();
        std::fs::create_dir_all(second_host.parent().unwrap()).unwrap();
        std::fs::write(&first_host, "export const overlay = 'first'\n").unwrap();
        std::fs::write(&second_host, "export const overlay = 'second'\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(&first_host).unwrap(),
            "export const overlay = 'first'\n"
        );
        assert_eq!(
            std::fs::read_to_string(&second_host).unwrap(),
            "export const overlay = 'second'\n"
        );
    }

    #[test]
    fn startup_reaps_abandoned_sessions_but_keeps_live_owners() {
        let base = tempfile::tempdir().unwrap();
        let abandoned = base.path().join("session-abandoned");
        let live = base.path().join("session-live");
        std::fs::create_dir(&abandoned).unwrap();
        std::fs::create_dir(&live).unwrap();
        std::fs::write(abandoned.join("payload"), "stale").unwrap();
        std::fs::write(live.join("payload"), "live").unwrap();
        drop(open_lock(&abandoned.join(".session.lock")).unwrap());
        let live_lock = open_lock(&live.join(".session.lock")).unwrap();
        live_lock.lock().unwrap();

        reap_abandoned_sessions(base.path());

        assert!(!abandoned.exists());
        assert!(live.is_dir());
    }
}
