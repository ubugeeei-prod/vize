use super::api_mode_for_executable;
use crate::linter::corsa_session::CorsaTypeAwareSession;
use corsa::api::ApiMode;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use vize_s0::{corsa_resolver::platform_suffix, cstr};

static NEXT_CASE_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn uses_json_rpc_for_node_wrappers() {
    assert_eq!(
        api_mode_for_executable(Path::new("/workspace/node_modules/.bin/tsgo")),
        ApiMode::AsyncJsonRpcStdio
    );
    assert_eq!(
        api_mode_for_executable(Path::new(
            "/workspace/node_modules/@typescript/native-preview/bin/tsgo.js"
        )),
        ApiMode::AsyncJsonRpcStdio
    );
}

#[test]
fn uses_json_rpc_for_typescript_native_preview_binaries() {
    assert_eq!(
        api_mode_for_executable(Path::new(
            "/workspace/node_modules/@typescript/native-preview-darwin-arm64/lib/tsgo"
        )),
        ApiMode::AsyncJsonRpcStdio
    );
}

#[test]
fn uses_json_rpc_for_typescript_seven_native_binaries() {
    let suffix = platform_suffix();
    for executable in ["tsc", "tsc.exe"] {
        let path = PathBuf::from(format!(
            "/workspace/node_modules/@typescript/typescript-{suffix}/lib/{executable}"
        ));

        assert_eq!(
            api_mode_for_executable(&path),
            ApiMode::AsyncJsonRpcStdio,
            "{}",
            path.display()
        );
    }
}

#[test]
fn uses_msgpack_for_generic_native_binaries() {
    assert_eq!(
        api_mode_for_executable(Path::new("/workspace/bin/corsa")),
        ApiMode::SyncMsgpackStdio
    );
}

#[test]
fn cleans_session_root_when_spawn_fails() {
    let root = case_dir("spawn-fails");
    let _ = std::fs::remove_dir_all(&root);
    let source = root.join("Component.vue");
    let invalid_corsa = root.join("not-corsa");

    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("package.json"), "{}").unwrap();
    std::fs::write(&invalid_corsa, "").unwrap();

    let error = match CorsaTypeAwareSession::new_with_corsa_path(
        source.to_str().unwrap(),
        Some(invalid_corsa.as_path()),
    ) {
        Ok(mut session) => {
            session.close();
            panic!("invalid corsa executable unexpectedly started");
        }
        Err(error) => error,
    };

    assert!(error.contains("Failed to start corsa type-aware session"));
    assert!(!root.join(".vize").join("patina").exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn starts_type_aware_session_from_package_root() {
    let Some(corsa_path) = test_corsa_path() else {
        return;
    };
    let root = case_dir("package-root");
    let _ = std::fs::remove_dir_all(&root);
    let source = root.join("src").join("Component.vue");

    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::write(root.join("package.json"), "{}").unwrap();
    std::fs::write(&source, "<template />").unwrap();

    let mut session = match CorsaTypeAwareSession::new_with_corsa_path(
        source.to_str().unwrap(),
        Some(&corsa_path),
    ) {
        Ok(session) => session,
        Err(error) if is_standard_tsgo_project_session_gap(&corsa_path, &error) => {
            let _ = std::fs::remove_dir_all(&root);
            return;
        }
        Err(error) => panic!("package-root patina session should start: {error:?}"),
    };
    session
        .open_virtual_project("const value: number = 1;\n")
        .expect("package-root patina session should refresh the virtual file");
    session.close();

    let _ = std::fs::remove_dir_all(&root);
}

fn case_dir(name: &str) -> std::path::PathBuf {
    let id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join(&*cstr!(
            "patina-corsa-session-{name}-{}-{id}",
            std::process::id()
        ))
}

fn test_corsa_path() -> Option<PathBuf> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .parent()?
        .to_path_buf();
    let stable = repo_root
        .join("node_modules")
        .join("@typescript")
        .join(&*cstr!("typescript-{}", platform_suffix()))
        .join("lib")
        .join(&*cstr!("tsc{}", std::env::consts::EXE_SUFFIX));
    if stable.is_file() {
        return Some(stable);
    }

    let native = repo_root
        .join("node_modules")
        .join("@typescript")
        .join(&*cstr!("native-preview-{}", platform_suffix()))
        .join("lib")
        .join(&*cstr!("tsgo{}", std::env::consts::EXE_SUFFIX));
    if native.is_file() {
        return Some(native);
    }

    let wrapper = repo_root.join("node_modules").join(".bin").join("tsgo");
    if wrapper.is_file() {
        Some(wrapper)
    } else {
        None
    }
}

fn is_standard_tsgo_project_session_gap(corsa_path: &Path, error: &str) -> bool {
    is_typescript_runtime(corsa_path)
        && (error.contains("project session did not resolve a project")
            || error.contains("no project found for file"))
}

fn is_typescript_runtime(corsa_path: &Path) -> bool {
    let mut after_typescript_scope = false;
    for component in corsa_path.components() {
        let text = component.as_os_str().to_string_lossy();
        if after_typescript_scope
            && (text.starts_with("typescript-")
                || text == "native-preview"
                || text.starts_with("native-preview-"))
        {
            return true;
        }
        after_typescript_scope = text == "@typescript";
    }
    is_typescript_bin_wrapper(corsa_path)
}

fn is_typescript_bin_wrapper(corsa_path: &Path) -> bool {
    if corsa_path
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        != Some(".bin")
    {
        return false;
    }

    let Some(name) = corsa_path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if name != "tsc" && name != "tsgo" {
        return false;
    }

    std::fs::read_to_string(corsa_path).is_ok_and(|contents| {
        contents.contains("@typescript/typescript-")
            || contents.contains("@typescript/native-preview")
    })
}

#[test]
fn standard_tsgo_project_session_gap_is_limited_to_typescript_runtimes() {
    let ts7 = Path::new("/workspace/node_modules/@typescript/typescript-darwin-arm64/lib/tsc");
    let native_preview =
        Path::new("/workspace/node_modules/@typescript/native-preview-darwin-arm64/lib/tsgo");
    let message = "Failed to start corsa type-aware session: protocol error: project session did not resolve a project";

    assert!(is_standard_tsgo_project_session_gap(ts7, message));
    assert!(is_standard_tsgo_project_session_gap(
        native_preview,
        message
    ));
    assert!(!is_standard_tsgo_project_session_gap(
        Path::new("/workspace/bin/corsa"),
        message
    ));
}

#[test]
fn standard_tsgo_project_session_gap_accepts_pnpm_bin_wrappers() {
    let root = case_dir("tsgo-wrapper");
    let bin_dir = root.join("node_modules").join(".bin");
    let wrapper = bin_dir.join("tsgo");
    std::fs::create_dir_all(&bin_dir).unwrap();
    std::fs::write(
        &wrapper,
        "exec node \"$basedir/../@typescript/native-preview/bin/tsgo.js\" \"$@\"\n",
    )
    .unwrap();

    assert!(is_standard_tsgo_project_session_gap(
        &wrapper,
        "api: client error: no project found for file active.patina.ts"
    ));

    let _ = std::fs::remove_dir_all(&root);
}
