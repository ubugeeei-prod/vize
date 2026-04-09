use super::{
    paths::{
        corsa_search_roots, find_corsa_in_common_locations, find_corsa_in_path,
        find_corsa_in_search_roots,
    },
    session::spawn_project_session,
    CorsaProjectClient,
};
use std::path::{Path, PathBuf};
use vize_carton::String;

pub(super) fn resolve_corsa_executable(
    corsa_path: Option<&str>,
    working_dir: Option<&str>,
) -> String {
    let search_roots = corsa_search_roots(working_dir.map(Path::new));

    corsa_path
        .map(String::from)
        .or_else(|| std::env::var("CORSA_PATH").ok().map(String::from))
        .or_else(|| std::env::var("TSGO_PATH").ok().map(String::from))
        .or_else(|| find_corsa_in_search_roots(&search_roots))
        .or_else(find_corsa_in_common_locations)
        .or_else(find_corsa_in_path)
        .unwrap_or_else(|| "corsa".into())
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
