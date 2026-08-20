use super::fallback::{FallbackCause, classify_fallback_cause};
use super::{CorsaError, CorsaExecutor, collect_declaration_outputs, collect_virtual_file_uris};
use crate::batch::source_policy::SourceFilePolicy;
use crate::file_uri::path_to_file_uri;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use vize_carton::cstr;

use tempfile::TempDir;

mod declaration_emit;
#[cfg(unix)]
#[path = "tests/incremental_fallback.rs"]
mod incremental_fallback;

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(&*cstr!(
            "corsa-executor-{name}-{}-{case_id}",
            std::process::id()
        ))
}

#[test]
fn collects_virtual_type_script_files_only() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::write(root.join("index.ts"), "").unwrap();
    fs::write(root.join("component.vue.ts"), "").unwrap();
    fs::write(root.join("module.mts"), "").unwrap();
    fs::write(root.join("common.cts"), "").unwrap();
    fs::write(root.join("module-types.d.mts"), "").unwrap();
    fs::write(root.join("common-types.d.cts"), "").unwrap();
    fs::write(root.join("__vize_vue_modules.d.ts"), "").unwrap();
    fs::write(root.join("__vize_auto_imports.d.ts"), "").unwrap();
    fs::create_dir_all(root.join("node_modules/vue")).unwrap();
    fs::write(root.join("node_modules/vue/index.d.ts"), "").unwrap();
    fs::create_dir_all(root.join("node_modules/vite")).unwrap();
    fs::write(root.join("node_modules/vite/client.d.ts"), "").unwrap();
    fs::write(root.join("tsconfig.json"), "{}").unwrap();
    fs::write(root.join("ignored.js"), "").unwrap();

    let uris = collect_virtual_file_uris(root, SourceFilePolicy::default()).unwrap();

    assert_eq!(
        uris,
        vec![
            path_to_file_uri(root.join("common-types.d.cts").as_path()),
            path_to_file_uri(root.join("common.cts").as_path()),
            path_to_file_uri(root.join("component.vue.ts").as_path()),
            path_to_file_uri(root.join("index.ts").as_path()),
            path_to_file_uri(root.join("module-types.d.mts").as_path()),
            path_to_file_uri(root.join("module.mts").as_path()),
        ]
    );

    let allow_js = SourceFilePolicy::from_compiler_options(
        serde_json::json!({ "allowJs": true }).as_object().unwrap(),
    );
    let uris = collect_virtual_file_uris(root, allow_js).unwrap();
    assert!(
        uris.contains(&path_to_file_uri(root.join("ignored.js").as_path())),
        "allowJs should make the JavaScript family diagnostic inputs"
    );
}

#[test]
fn encodes_reserved_characters_in_virtual_file_uris() {
    let root = unique_case_dir("reserved-uri");
    let _ = fs::remove_dir_all(&root);
    let route_dir = root.join("pages").join("[[org]]").join("[packageName]");
    fs::create_dir_all(&route_dir).unwrap();
    fs::write(route_dir.join("[versionRange].vue.ts"), "").unwrap();

    let uris = collect_virtual_file_uris(root.as_path(), SourceFilePolicy::default()).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert_eq!(
        uris,
        vec![path_to_file_uri(
            route_dir.join("[versionRange].vue.ts").as_path()
        )]
    );
    assert!(uris[0].contains("%5B%5Borg%5D%5D"));
    assert!(uris[0].contains("%5BpackageName%5D"));
    assert!(uris[0].contains("%5BversionRange%5D.vue.ts"));
}

#[test]
fn normalizes_explicit_node_modules_bin_wrapper_to_native_preview_binary() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();
    let wrapper = root.join("node_modules/.bin/tsgo");
    let native = root
        .join("node_modules")
        .join("@typescript")
        .join("native-preview")
        .join("lib")
        .join("tsgo");

    fs::create_dir_all(wrapper.parent().unwrap()).unwrap();
    fs::create_dir_all(native.parent().unwrap()).unwrap();
    fs::write(&wrapper, "").unwrap();
    fs::write(&native, "").unwrap();

    let executor = CorsaExecutor::with_corsa_path(root, Some(&wrapper)).unwrap();

    assert_eq!(executor.corsa_path(), native.canonicalize().unwrap());
}

#[test]
fn uses_explicit_corsa_path() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project");
    let explicit = temp_dir.path().join("bin").join("tsgo");

    fs::create_dir_all(&project_root).unwrap();
    fs::create_dir_all(explicit.parent().unwrap()).unwrap();
    fs::write(&explicit, "").unwrap();

    let executor = CorsaExecutor::with_corsa_path(&project_root, Some(&explicit)).unwrap();

    assert_eq!(executor.corsa_path(), explicit.canonicalize().unwrap());
}

#[test]
fn resolves_relative_explicit_corsa_path_against_project_root() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project");
    let explicit = project_root.join("bin").join("tsgo");

    fs::create_dir_all(explicit.parent().unwrap()).unwrap();
    fs::write(&explicit, "").unwrap();

    let executor =
        CorsaExecutor::with_corsa_path(&project_root, Some(PathBuf::from("bin/tsgo").as_path()))
            .unwrap();

    assert_eq!(executor.corsa_path(), explicit.canonicalize().unwrap());
}

#[test]
fn collects_emitted_declaration_outputs() {
    let temp_dir = TempDir::new().unwrap();
    let out_dir = temp_dir.path().join("dist/types");
    fs::create_dir_all(&out_dir).unwrap();
    fs::write(out_dir.join("App.vue.d.ts"), "export {};\n").unwrap();
    fs::write(out_dir.join("skip.js"), "").unwrap();

    let files = collect_declaration_outputs(&out_dir).unwrap();

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, out_dir.join("App.vue.d.ts"));
    assert_eq!(files[0].content, "export {};\n");
}

#[cfg(unix)]
#[test]
fn cli_global_diagnostics_do_not_trigger_session_fallback() {
    use crate::batch::VirtualProject;
    use std::os::unix::fs::PermissionsExt;

    let case_dir = unique_case_dir("global-diagnostics");
    let _ = fs::remove_dir_all(&case_dir);
    let cache_dir = case_dir.join(".cache");
    let source = case_dir.join("src").join("main.ts");
    fs::create_dir_all(&cache_dir).unwrap();
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "const value: number = 1;\n").unwrap();

    // A runtime whose project check exits non-zero with only file-less
    // config diagnostics (e.g. TS2688) ran fine; treating that as a CLI
    // failure would fall back to the far slower project-session API
    // (`--api` here would hang the test forever). The project-level
    // diagnostic surfaces attributed to the project's tsconfig anchor.
    let tsgo = cache_dir.join("tsgo");
    fs::write(
        &tsgo,
        "#!/bin/sh\nif [ \"$1\" = \"--api\" ]; then exec sleep 600; fi\necho \"error TS2688: Cannot find type definition file for 'vite/client'.\"\nexit 2\n",
    )
    .unwrap();
    fs::set_permissions(&tsgo, fs::Permissions::from_mode(0o755)).unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&source).unwrap();
    let executor = CorsaExecutor::new(&case_dir).unwrap();
    let result = executor.check(&project).unwrap();

    assert!(!result.success);
    assert_eq!(result.exit_code, 2);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, Some(2688));
    assert_eq!(result.diagnostics[0].severity, 1);
    assert_eq!(result.diagnostics[0].file, case_dir);

    let _ = fs::remove_dir_all(&case_dir);
}

#[cfg(unix)]
#[test]
fn checks_with_cli_when_project_session_api_is_unavailable() {
    use crate::batch::VirtualProject;
    use std::os::unix::fs::PermissionsExt;

    let _fallback_guard = super::fallback::FALLBACK_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let case_dir = unique_case_dir("cli-fallback");
    let _ = fs::remove_dir_all(&case_dir);
    let cache_dir = case_dir.join(".cache");
    let source = case_dir.join("src").join("main.ts");
    fs::create_dir_all(&cache_dir).unwrap();
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "const value: number = 1;\n").unwrap();

    let tsgo = cache_dir.join("tsgo");
    fs::write(
        &tsgo,
        "#!/bin/sh\nif [ \"$1\" = \"--api\" ]; then printf 'api unavailable'; exit 0; fi\nexit 0\n",
    )
    .unwrap();
    fs::set_permissions(&tsgo, fs::Permissions::from_mode(0o755)).unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.register_path(&source).unwrap();
    let executor = CorsaExecutor::new(&case_dir).unwrap();
    let result = executor.check(&project).unwrap();

    assert!(result.success);
    assert!(result.diagnostics.is_empty());

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn classifies_spawn_failures() {
    let io = CorsaError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No such file or directory",
    ));
    assert_eq!(classify_fallback_cause(&io), FallbackCause::Spawn);

    let broken_pipe = CorsaError::CorsaExecution {
        exit_code: -1,
        message: "write failed: Broken pipe".into(),
    };
    assert_eq!(classify_fallback_cause(&broken_pipe), FallbackCause::Spawn);

    let panicked = CorsaError::CorsaExecution {
        exit_code: -1,
        message: "sharded corsa CLI worker panicked".into(),
    };
    assert_eq!(classify_fallback_cause(&panicked), FallbackCause::Spawn);
}

#[test]
fn classifies_parse_failures() {
    let parse = CorsaError::CorsaExecution {
        exit_code: -1,
        message: "expected tuple marker but found 0x00".into(),
    };
    assert_eq!(classify_fallback_cause(&parse), FallbackCause::Parse);
}

#[test]
fn classifies_check_failures() {
    let check = CorsaError::CorsaExecution {
        exit_code: 2,
        message: "Type 'string' is not assignable to type 'number'.".into(),
    };
    assert_eq!(classify_fallback_cause(&check), FallbackCause::Check);
}

#[test]
fn fallback_notice_is_observable_once_and_silenceable() {
    use super::fallback::{FallbackCause, FallbackStep, fallback_stderr_notice};

    let _fallback_guard = super::fallback::FALLBACK_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    // Re-arm the once-per-run guard and ensure the notice is not suppressed,
    // regardless of earlier degradations in this process.
    super::fallback::FALLBACK_NOTICE_EMITTED.store(false, std::sync::atomic::Ordering::Relaxed);
    // SAFETY: single-threaded test; the var is only read by the helper.
    unsafe { std::env::remove_var("VIZE_SILENCE_CORSA_FALLBACK") };

    // First degradation produces an observable, descriptive notice.
    let notice = fallback_stderr_notice(FallbackStep::SessionToCli, FallbackCause::Spawn)
        .expect("first fallback must produce an observable notice");
    assert!(
        notice.contains("corsa:") && notice.contains("slower path"),
        "expected an observable corsa fallback notice, got: {notice:?}"
    );
    assert!(
        notice.contains("project-session API unavailable") && notice.contains("spawn failure"),
        "notice must name the step and cause, got: {notice:?}"
    );

    // A second degradation in the same run must stay quiet (once per run).
    assert!(
        fallback_stderr_notice(FallbackStep::CliToSession, FallbackCause::Parse).is_none(),
        "the stderr notice must fire at most once per run"
    );

    // Opt-out suppresses the stderr notice without claiming the guard.
    super::fallback::FALLBACK_NOTICE_EMITTED.store(false, std::sync::atomic::Ordering::Relaxed);
    // SAFETY: single-threaded test; restored immediately after the call.
    unsafe { std::env::set_var("VIZE_SILENCE_CORSA_FALLBACK", "1") };
    let suppressed = fallback_stderr_notice(FallbackStep::CliToSession, FallbackCause::Check);
    // SAFETY: single-threaded test cleanup.
    unsafe { std::env::remove_var("VIZE_SILENCE_CORSA_FALLBACK") };
    assert!(
        suppressed.is_none(),
        "silenced fallback must not emit a notice"
    );
    assert!(
        !super::fallback::FALLBACK_NOTICE_EMITTED.load(std::sync::atomic::Ordering::Relaxed),
        "silenced fallback must not claim the once-per-run guard"
    );
}
