use super::{CorsaExecutor, fs, unique_case_dir};
use crate::batch::{IncrementalCheckMetrics, VirtualProject};
use std::os::unix::fs::PermissionsExt;

#[test]
fn incremental_session_fallback_is_counted_per_check() {
    let _fallback_guard = super::super::fallback::FALLBACK_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let case_dir = unique_case_dir("incremental-session-fallback-metrics");
    let _ = fs::remove_dir_all(&case_dir);
    let cache_dir = case_dir.join(".cache");
    let source = case_dir.join("src").join("main.ts");
    fs::create_dir_all(&cache_dir).unwrap();
    fs::create_dir_all(case_dir.join("node_modules")).unwrap();
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "const value: number = 1;\n").unwrap();

    // The project-session protocol exits before its handshake. The ordinary
    // CLI succeeds, making the degradation deterministic without a real Corsa
    // runtime and letting the metrics prove it was not silently hidden.
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
    let result = executor
        .check_incremental_session(&mut project, Some(1))
        .expect("the CLI fallback should keep the check successful");

    assert!(result.success);
    assert!(result.diagnostics.is_empty());
    let metrics = executor.incremental_metrics();
    assert!(
        matches!(metrics.last_materialized_entries_considered, 12 | 13),
        "{metrics:?}"
    );
    assert_eq!(
        metrics.last_tree_entries_scanned,
        metrics.last_materialized_entries_considered
    );
    assert_eq!(
        metrics,
        IncrementalCheckMetrics {
            checks: 1,
            session_to_cli_fallbacks: 1,
            last_session_to_cli_fallback: true,
            last_requested_files: 1,
            last_materialized_entries_considered: metrics.last_materialized_entries_considered,
            last_tree_entries_scanned: metrics.last_materialized_entries_considered,
            last_full_rebuild: true,
            ..Default::default()
        }
    );

    let _ = fs::remove_dir_all(&case_dir);
}
