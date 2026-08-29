use super::{link_workspace_node_modules, tsgo::resolve_test_tsgo_binary};
use corsa::{
    CorsaError,
    api::{ApiMode, ApiSpawnConfig, ProjectSession},
    runtime::block_on,
};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

pub(super) fn corsa_type_mismatch_snapshot(
    file_text: &str,
    declaration_marker: &str,
    initializer_marker: &str,
) -> Option<Vec<(std::string::String, std::string::String)>> {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    let project_root = workspace_root
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(format!("corsa-type-probe-{}-{case_id}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).expect("project root should exist");
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).expect("src dir should exist");
    link_workspace_node_modules(&project_root).expect("workspace node_modules should link");
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}"#,
    )
    .expect("tsconfig should write");
    let file = src_dir.join("App.virtual.ts");
    std::fs::write(&file, file_text).expect("virtual ts should write");

    let corsa_path =
        resolve_test_tsgo_binary().expect("tsgo executable should resolve for corsa api tests");
    let config_wire = project_root.join("tsconfig.json").display().to_string();
    let file_wire = file.display().to_string();
    let declaration_offset = file_text
        .find(declaration_marker)
        .expect("declaration marker should exist");
    let initializer_offset = file_text
        .find(initializer_marker)
        .map(|offset| offset + initializer_marker.len().saturating_sub(1))
        .expect("initializer marker should exist");

    let result = block_on(async {
        let session =
            spawn_test_project_session(&corsa_path, project_root.as_path(), config_wire.as_str())
                .await?;
        assert!(
            session
                .project()
                .root_files
                .iter()
                .any(|file| file.ends_with("App.virtual.ts")),
            "root files did not include App.virtual.ts: {:?}",
            session.project().root_files
        );
        let declaration = session
            .get_type_at_position(file_wire.as_str(), declaration_offset as u32)
            .await
            .expect("declaration type should load")
            .expect("declaration type should exist");
        let initializer = session
            .get_type_at_position(file_wire.as_str(), initializer_offset as u32)
            .await
            .expect("initializer type should load")
            .expect("initializer type should exist");
        let declaration_text = session
            .type_to_string(declaration.id, None, None)
            .await
            .expect("declaration type should render");
        let initializer_text = session
            .type_to_string(initializer.id, None, None)
            .await
            .expect("initializer type should render");
        session.close().await.expect("session should close");
        Some(vec![
            ("declaration".into(), declaration_text),
            ("initializer".into(), initializer_text),
        ])
    });
    let _ = std::fs::remove_dir_all(&project_root);
    result
}

pub(super) async fn test_project_session_is_available(
    corsa_path: &Path,
    project_root: &Path,
) -> bool {
    let config_wire = project_root.join("tsconfig.json").display().to_string();
    let Some(session) =
        spawn_test_project_session(corsa_path, project_root, config_wire.as_str()).await
    else {
        return false;
    };
    session
        .close()
        .await
        .expect("test project session should close");
    true
}

async fn spawn_test_project_session(
    corsa_path: &Path,
    project_root: &Path,
    config_wire: &str,
) -> Option<ProjectSession> {
    match ProjectSession::spawn(
        ApiSpawnConfig::new(corsa_path)
            .with_mode(ApiMode::AsyncJsonRpcStdio)
            .with_cwd(project_root),
        config_wire,
        None,
    )
    .await
    {
        Ok(session) => Some(session),
        Err(error) if is_standard_tsgo_project_session_gap(&error) => None,
        Err(error) => panic!("corsa project session should initialize: {error}"),
    }
}

fn is_standard_tsgo_project_session_gap(error: &CorsaError) -> bool {
    matches!(
        error,
        CorsaError::Protocol(detail)
            if detail.contains("project session did not resolve a project")
    )
}
