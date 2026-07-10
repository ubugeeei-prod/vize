use super::{CorsaProjectClient, session::spawn_project_session};
use std::path::{Path, PathBuf};
use vize_carton::{
    String,
    corsa_resolver::{CorsaResolveError, CorsaResolveRequest},
    cstr,
};

pub(super) fn resolve_corsa_executable(
    corsa_path: Option<&str>,
    working_dir: Option<&str>,
) -> Result<String, String> {
    let request = CorsaResolveRequest {
        explicit_path: corsa_path.map(Path::new),
        project_root: working_dir.map(Path::new),
    };

    match vize_carton::corsa_resolver::resolve_corsa_executable(request) {
        Ok(path) => Ok(path.to_string_lossy().into()),
        // Preserve the historical lenient fallback: spawning a bare `corsa`
        // still lets `PATH` changes made after resolution take effect.
        Err(CorsaResolveError::NotFound) => Ok("corsa".into()),
        Err(error @ CorsaResolveError::ExplicitNotFound { .. }) => Err(cstr!("{error}")),
    }
}

impl CorsaProjectClient {
    pub(super) fn spawn_initialized_client(
        executable: &str,
        cwd: PathBuf,
        root_path: Option<PathBuf>,
        temp_dir: Option<PathBuf>,
    ) -> Result<Self, String> {
        let project_root = root_path.as_deref().unwrap_or(&cwd);
        let (session, capabilities) = spawn_project_session(executable, &cwd, project_root)?;
        Ok(Self {
            executable: executable.into(),
            cwd: cwd.clone(),
            session,
            capabilities,
            overlay_api_disabled: false,
            project_root: project_root.to_path_buf(),
            diagnostics: Default::default(),
            overlay_versions: Default::default(),
            document_texts: Default::default(),
            session_document_uris: Default::default(),
            external_document_uris: Default::default(),
            temp_dir,
            closed: false,
        })
    }
}
