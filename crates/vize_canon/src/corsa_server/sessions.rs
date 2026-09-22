//! One `ProjectSession` per [`CorsaSessionKey`], reused across checks.

use std::path::{Path, PathBuf};
use std::time::Instant;

use vize_carton::{String, cstr};

use crate::corsa_client::CorsaProjectClient;
use crate::corsa_session_cache::{
    CorsaSessionKey, corsa_build_identity, project_tsconfig, typescript_project_inits,
};

use super::CorsaServer;

impl CorsaServer {
    pub(crate) fn project_inits(&self) -> u64 {
        self.observed_project_inits
    }

    pub(crate) fn live_sessions(&self) -> usize {
        self.sessions.len()
    }

    /// Drop sessions that have been idle since before `now`.
    #[cfg(test)]
    pub(crate) fn reap_idle_at(&mut self, now: Instant) {
        self.sessions.reap(now);
    }

    /// Run `body` against the session for `source`'s project and `flags`.
    /// The TypeScript init counter moves only when this call has to spawn.
    pub(super) fn with_project_session<R>(
        &mut self,
        source: &Path,
        flags: &str,
        body: impl FnOnce(&mut CorsaProjectClient) -> Result<R, String>,
    ) -> Result<R, String> {
        let before = typescript_project_inits();
        let result = self.with_project_session_inner(source, flags, body);
        let inits = typescript_project_inits();
        self.observed_project_inits = self
            .observed_project_inits
            .saturating_add(inits.saturating_sub(before));
        result
    }

    fn with_project_session_inner<R>(
        &mut self,
        source: &Path,
        flags: &str,
        body: impl FnOnce(&mut CorsaProjectClient) -> Result<R, String>,
    ) -> Result<R, String> {
        let key = self.session_key(source, flags)?;
        let now = Instant::now();
        self.sessions.reap(now);
        if self.sessions.get_mut(&key, now).is_none() {
            let client = CorsaProjectClient::new(
                self.config.corsa_path.as_deref(),
                self.config.working_dir.as_deref(),
            )?;
            self.sessions
                .insert_spawned(key.clone(), client, Instant::now());
        }
        let result = {
            let client = self
                .sessions
                .get_mut(&key, Instant::now())
                .expect("the session was reused or just inserted");
            body(client)
        };
        // A check can outlast a short idle window. The session was in use the
        // whole time, so idle is measured from the moment it became free.
        let _ = self.sessions.get_mut(&key, Instant::now());
        result
    }

    fn session_key(&self, source: &Path, flags: &str) -> Result<CorsaSessionKey, String> {
        let root = self.working_dir();
        let source = std::fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf());
        let root = std::fs::canonicalize(&root).unwrap_or(root);
        let tsconfig = project_tsconfig(&source, &root);
        let executable = corsa_executable(self.config.corsa_path.as_deref(), &root)?;
        let identity = corsa_build_identity(&executable);
        Ok(CorsaSessionKey::new(&tsconfig, identity.as_str(), flags))
    }
}

fn corsa_executable(corsa_path: Option<&str>, working_dir: &Path) -> Result<PathBuf, String> {
    let request = vize_carton::corsa_resolver::CorsaResolveRequest {
        explicit_path: corsa_path.map(Path::new),
        project_root: Some(working_dir),
    };
    match vize_carton::corsa_resolver::resolve_corsa_executable(request) {
        Ok(path) => Ok(path),
        Err(vize_carton::corsa_resolver::CorsaResolveError::NotFound) => Ok(PathBuf::from("corsa")),
        Err(error) => Err(cstr!("{error}")),
    }
}
