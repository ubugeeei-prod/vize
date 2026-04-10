use super::{BatchTypeChecker, DeclarationEmitOptions, Diagnostic, TypeCheckResult};
use crate::batch::TypeChecker;
use crate::sfc_typecheck::{type_check_sfc, SfcTypeCheckOptions};
use corsa::{
    api::{ApiMode, ApiSpawnConfig, ProjectSession},
    runtime::block_on,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
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
    let project_root = unique_case_dir("scan");
    let _ = std::fs::remove_dir_all(&project_root);
    let src_dir = project_root.join("src");
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

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => return,
    };

    checker.scan_project().unwrap();
    assert_eq!(checker.file_count(), 2);

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_snapshots_vue_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
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
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
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
    let project_root = unique_case_dir("template-ref");
    let _ = std::fs::remove_dir_all(&project_root);
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    if link_workspace_node_modules(&project_root).is_err() {
        return;
    }
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

    let mut checker = match BatchTypeChecker::new(&project_root) {
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
                relative_path(&project_root, &diagnostic.file),
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

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_snapshots_cross_file_vue_prop_error() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "cross-file-vue-props",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{
  count: number
}>()
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child :count="'oops'" />
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_cross_file_vue_prop_error", snapshot);
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_snapshots_ts_imports_vue_component() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "ts-imports-vue",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
defineProps<{
  count: number
}>()
</script>

<template>
  <div>{{ count }}</div>
</template>
"#,
            ),
            (
                "src/main.ts",
                r#"import App from './App.vue'

type AppProps = InstanceType<typeof App>['$props']

const props: AppProps = {
  count: 'oops',
}

void props
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_ts_imports_vue_component", snapshot);
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_snapshots_ambient_dts_global_usage() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "ambient-dts",
        &[
            ("src/env.d.ts", r#"declare const APP_VERSION: string;"#),
            (
                "src/App.vue",
                r#"<template>
  <div>{{ APP_VERSION.toFixed(2) }}</div>
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_ambient_dts_global_usage", snapshot);
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_snapshots_declaration_emit_outputs() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "declaration-emit",
        &[
            (
                "src/App.vue",
                r#"<script setup lang="ts">
export interface PublicProps {
  count: number
}

const props = defineProps<PublicProps>()
</script>

<template>
  <div>{{ props.count }}</div>
</template>
"#,
            ),
            (
                "src/index.ts",
                r#"export { default as App } from './App.vue'
"#,
            ),
        ],
    );

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => return,
    };
    checker.scan_project().unwrap();
    let out_dir = project_root.join("types");
    let emitted = checker
        .emit_declarations(&DeclarationEmitOptions::new(out_dir.clone()))
        .unwrap();
    let snapshot: Vec<_> = emitted
        .files
        .into_iter()
        .map(|file| (relative_path(&out_dir, &file.path), file.content))
        .collect();

    insta::with_settings!({
        snapshot_path => "../../snapshots"
    }, {
        insta::assert_debug_snapshot!("batch_type_checker_declaration_emit_outputs", snapshot);
    });

    let _ = std::fs::remove_dir_all(&project_root);
}

fn relative_path(root: &std::path::Path, file: &std::path::Path) -> String {
    file.strip_prefix(root)
        .map(|path| cstr!("{}", path.display()))
        .unwrap_or_else(|_| cstr!("{}", file.display()))
}

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    workspace_root
        .join("__agent_only")
        .join("tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}

fn create_project_case(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    link_workspace_node_modules(&project_root).unwrap();
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
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();

    for (path, source) in files {
        let file_path = project_root.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(file_path, source).unwrap();
    }

    project_root
}

fn snapshot_project_diagnostics(project_root: &Path) -> Option<Vec<(String, Option<u32>, String)>> {
    let mut checker = BatchTypeChecker::new(project_root).ok()?;
    checker.scan_project().ok()?;
    let result = checker.check_project().ok()?;

    let mut snapshot: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}:{} {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    match diagnostic.severity {
                        1 => "error",
                        2 => "warning",
                        3 => "info",
                        _ => "hint",
                    },
                    diagnostic.message
                ),
            )
        })
        .collect();
    snapshot.sort();
    Some(snapshot)
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
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }

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
    let workspace_node_modules = resolve_workspace_node_modules(workspace_root);

    let target = project_root.join("node_modules");
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(&target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(&target)?;
    }
    std::fs::create_dir_all(&target)?;

    if let Some(ref workspace_node_modules) = workspace_node_modules {
        for package in ["vue", "vite", "@vue"] {
            let source = workspace_node_modules.join(package);
            if source.exists() {
                symlink_path(&source, &target.join(package))?;
            }
        }
    } else {
        write_test_vue_stub(&target)?;
        write_test_vite_stub(&target)?;
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

fn resolve_workspace_node_modules(workspace_root: &Path) -> Option<PathBuf> {
    let override_path = std::env::var_os("VIZE_TEST_WORKSPACE_NODE_MODULES");
    if let Some(override_path) = override_path {
        let override_path = PathBuf::from(override_path);
        if override_path.as_os_str() == "__none__" {
            return None;
        }
        return override_path.exists().then_some(override_path);
    }

    let workspace_node_modules = workspace_root.join("node_modules");
    workspace_node_modules
        .exists()
        .then_some(workspace_node_modules)
}

fn write_test_vue_stub(target: &Path) -> std::io::Result<()> {
    let vue_dir = target.join("vue");
    std::fs::create_dir_all(&vue_dir)?;
    std::fs::write(
        vue_dir.join("package.json"),
        r#"{
  "name": "vue",
  "types": "index.d.ts"
}"#,
    )?;
    std::fs::write(
        vue_dir.join("index.d.ts"),
        r#"export interface Ref<T = any, S = T> {
  value: T;
}

export interface ShallowRef<T = any, S = T> extends Ref<T, S> {}

export interface ComponentPublicInstance {
  $attrs: any;
  $slots: any;
  $refs: any;
  $emit: (...args: any[]) => void;
}

export declare function ref<T>(value: T): Ref<T>;
export declare function useTemplateRef<T = any>(key: string): ShallowRef<T | null>;
"#,
    )?;
    Ok(())
}

fn write_test_vite_stub(target: &Path) -> std::io::Result<()> {
    let vite_dir = target.join("vite");
    std::fs::create_dir_all(&vite_dir)?;
    std::fs::write(
        vite_dir.join("package.json"),
        r#"{
  "name": "vite",
  "types": "client.d.ts"
}"#,
    )?;
    std::fs::write(vite_dir.join("client.d.ts"), "")?;
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
