//! Private lifetime owner for editor Canon mirrors and their in-memory cache.

use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use super::types::CorsaBridgeError;
use super::vue_dependencies_alias::{SessionCache, recover_lock};

/// One bridge/check-server session owns one private mirror namespace. Live
/// overlay bytes and the cache describing them must never be shared with an
/// independent native TypeScript session.
pub(crate) struct EditorMirrorSession {
    storage: Result<tempfile::TempDir, vize_carton::String>,
    cache: Mutex<SessionCache>,
}

impl EditorMirrorSession {
    pub(crate) fn new() -> Self {
        let storage = create_storage().map_err(|error| {
            vize_carton::cstr!("failed to create private editor mirror storage: {error}")
        });
        Self {
            storage,
            cache: Mutex::new(SessionCache::default()),
        }
    }

    pub(crate) fn root(&self) -> Result<&Path, CorsaBridgeError> {
        self.storage
            .as_ref()
            .map(|storage| storage.path())
            .map_err(|error| CorsaBridgeError::CommunicationError(error.clone()))
    }

    pub(in crate::corsa_bridge) fn cache(&self) -> MutexGuard<'_, SessionCache> {
        recover_lock(&self.cache)
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

fn create_storage() -> std::io::Result<tempfile::TempDir> {
    let base = vize_carton::path::canonicalize_non_verbatim(&std::env::temp_dir())
        .join("vize-canon")
        .join("editor")
        .join("sessions");
    std::fs::create_dir_all(&base)?;
    let storage = tempfile::Builder::new()
        .prefix("session-")
        .tempdir_in(base)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(storage.path(), std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(storage)
}

#[cfg(test)]
mod tests {
    use super::EditorMirrorSession;

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
}
