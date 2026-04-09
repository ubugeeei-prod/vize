use super::{BatchTypeChecker, Diagnostic, TypeCheckResult};
use crate::batch::TypeChecker;
use crate::sfc_typecheck::{type_check_sfc, SfcTypeCheckOptions};
use corsa::{
    api::{ApiMode, ApiSpawnConfig, ProjectSession},
    runtime::block_on,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::TempDir;
use vize_carton::{cstr, String};

#[test]
fn test_type_check_result() {
    let mut result = TypeCheckResult::default();
    assert!(!result.has_errors());
    assert_eq!(result.error_count(), 0);

    result.diagnostics.push(Diagnostic {
        file: PathBuf::from("test.vue"),
        line: 0,
        column: 0,
        message: "error".into(),
        code: Some(2304),
        severity: 1,
        block_type: None,
    });

    assert!(result.has_errors());
    assert_eq!(result.error_count(), 1);
}

#[test]
fn test_batch_type_checker_scan() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();

    let vue_content = r#"<template>
  <div>{{ message }}</div>
</template>

<script setup lang="ts">
const message = 'Hello'
</script>
"#;
    std::fs::write(src_dir.join("App.vue"), vue_content).unwrap();
    std::fs::write(src_dir.join("utils.ts"), "export const foo = 'bar';").unwrap();

    let mut checker = match BatchTypeChecker::new(temp_dir.path()) {
        Ok(checker) => checker,
        Err(_) => return,
    };

    checker.scan_project().unwrap();
    assert_eq!(checker.file_count(), 2);
}

#[test]
fn batch_type_checker_snapshots_vue_diagnostics() {
    let source = r#"<script setup lang="ts">
const count: number = 'oops'
</script>
"#;
    let virtual_ts = type_check_sfc(
        source,
        &SfcTypeCheckOptions::new("App.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual ts should be generated");
    let snapshot = corsa_type_mismatch_snapshot(&virtual_ts, "count: number", "'oops'");

    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_vue_diagnostics", snapshot);
    });
}

#[test]
fn batch_type_checker_snapshots_script_setup_type_error() {
    let virtual_ts = type_check_sfc(
        r#"<script setup lang="ts">
const count: string = 0;
</script>
"#,
        &SfcTypeCheckOptions::new("App.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual ts should be generated");
    let relevant = corsa_type_mismatch_snapshot(&virtual_ts, "count: string", "= 0");

    assert_eq!(
        relevant.len(),
        2,
        "expected declaration and initializer types, got: {relevant:#?}"
    );
    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_script_setup_type_error", relevant);
    });
}

#[test]
fn batch_type_checker_accepts_template_ref_unwrap_and_array_access() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    if link_workspace_node_modules(temp_dir.path()).is_err() {
        return;
    }
    std::fs::write(
        temp_dir.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["App.virtual.ts"]
}"#,
    )
    .unwrap();
    std::fs::write(
        src_dir.join("App.vue"),
        r#"<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'

const users = ref([{ id: 1 }])
const inputRef = useTemplateRef<HTMLInputElement>('input')
</script>

<template>
  <div>{{ users.length }} {{ inputRef && inputRef.focus() }}</div>
</template>
"#,
    )
    .unwrap();

    let mut checker = match BatchTypeChecker::new(temp_dir.path()) {
        Ok(checker) => checker,
        Err(_) => return,
    };
    checker.scan_project().unwrap();

    let result = match checker.check_project() {
        Ok(result) => result,
        Err(_) => return,
    };

    let relevant: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| matches!(diagnostic.code, Some(2339) | Some(2349)))
        .map(|diagnostic| {
            (
                relative_path(temp_dir.path(), &diagnostic.file),
                diagnostic.code,
                diagnostic.line,
                diagnostic.column,
                diagnostic.message.clone(),
                diagnostic.block_type,
            )
        })
        .collect();

    assert!(
        relevant.is_empty(),
        "unexpected template unwrap diagnostics: {relevant:#?}"
    );
}

fn relative_path(root: &std::path::Path, file: &std::path::Path) -> String {
    file.strip_prefix(root)
        .map(|path| cstr!("{}", path.display()))
        .unwrap_or_else(|_| cstr!("{}", file.display()))
}

fn corsa_type_mismatch_snapshot(
    file_text: &str,
    declaration_marker: &str,
    initializer_marker: &str,
) -> Vec<(std::string::String, std::string::String)> {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    let project_root = workspace_root
        .join("__agent_only")
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
        let session = ProjectSession::spawn(
            ApiSpawnConfig::new(corsa_path)
                .with_mode(ApiMode::AsyncJsonRpcStdio)
                .with_cwd(project_root.as_path()),
            config_wire,
            None,
        )
        .await
        .expect("corsa project session should initialize");
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
        vec![
            ("declaration".into(), declaration_text),
            ("initializer".into(), initializer_text),
        ]
    });
    let _ = std::fs::remove_dir_all(&project_root);
    result
}

fn resolve_test_tsgo_binary() -> Option<PathBuf> {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)?;
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache);
    }

    crate::lsp_client::paths::find_corsa_in_local_node_modules(Some(
        &workspace_root.display().to_string(),
    ))
    .map(|path| PathBuf::from(path.as_str()))
}

fn link_workspace_node_modules(project_root: &Path) -> std::io::Result<()> {
    let Some(workspace_root) = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
    else {
        return Err(std::io::Error::other("workspace root not found"));
    };
    let workspace_node_modules = workspace_root.join("node_modules");
    if !workspace_node_modules.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace node_modules not found",
        ));
    }

    let target = project_root.join("node_modules");
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(&target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(&target)?;
    }
    std::fs::create_dir_all(&target)?;

    for package in ["vue", "vite", "@vue"] {
        let source = workspace_node_modules.join(package);
        if source.exists() {
            symlink_path(&source, &target.join(package))?;
        }
    }

    if let Some(corsa_path) = crate::lsp_client::paths::find_corsa_in_local_node_modules(Some(
        &workspace_root.display().to_string(),
    )) {
        let source = PathBuf::from(corsa_path.as_str());
        if source.exists() {
            let file_name = source.file_name().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid corsa binary path",
                )
            })?;
            symlink_path(
                &source,
                &target
                    .join("@typescript")
                    .join("native-preview")
                    .join("lib")
                    .join(file_name),
            )?;
            symlink_path(&source, &target.join(".bin").join(file_name))?;
        }
    }

    Ok(())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(target)?;
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        let metadata = std::fs::metadata(source)?;
        if metadata.is_dir() {
            std::os::windows::fs::symlink_dir(source, target)
        } else {
            std::os::windows::fs::symlink_file(source, target)
        }
    }
}
