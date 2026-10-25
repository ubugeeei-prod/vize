use super::CorsaExecutor;
use crate::batch::VirtualProject;
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
use vize_carton::{String, cstr};

const INVALID: &str = r#"<script setup lang="ts" generic="T extends">
const independent: number = 'bad';
void independent;
const genericValue: T | undefined = undefined;
void genericValue;
</script>
<template><div /></template>
"#;

fn required_project(root: &Path) -> VirtualProject {
    fs::create_dir_all(root.join("node_modules")).unwrap();
    let source = root.join("App.vue");
    fs::write(&source, INVALID).unwrap();
    let mut project = VirtualProject::new(root).unwrap();
    project.register_path(&source).unwrap();
    assert!(project.requires_project_session());
    project
}

fn refusing_session_with_successful_cli(root: &Path) -> (CorsaExecutor, std::path::PathBuf) {
    let transcript = root.join("whole-argv.jsonl");
    let executable = root.join("tsgo");
    // Every ordinary CLI invocation would succeed. Both session transports
    // close immediately, so any accidental CLI fallback masks the real failure.
    let script = cstr!(
        "#!/usr/bin/env node\nconst fs = require('node:fs');\nfs.appendFileSync({}, JSON.stringify(process.argv.slice(2)) + '\\n');\nif (process.argv.includes('--api')) process.stdout.write('api unavailable');\n",
        serde_json::to_string(&transcript).unwrap()
    );
    fs::write(&executable, script.as_bytes()).unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    (
        CorsaExecutor::with_corsa_path(root, Some(&executable)).unwrap(),
        transcript,
    )
}

fn assert_no_program_cli(transcript: &Path) {
    let calls: Vec<Vec<String>> = fs::read_to_string(transcript)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        !calls.is_empty(),
        "the actual session must have been attempted"
    );
    assert!(
        calls.len() <= 2,
        "only one session's existing msgpack/JSON transport negotiation may run; an outer retry is forbidden: {calls:?}"
    );
    for args in calls {
        assert!(
            args.iter()
                .all(|arg| !matches!(arg.as_str(), "--project" | "--checkers" | "--incremental")),
            "a required session must not degrade to an incomplete program CLI: {args:?}"
        );
    }
}

#[test]
fn required_session_refusal_never_runs_single_or_sharded_program_cli() {
    let _fallback_guard = super::super::fallback::FALLBACK_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for servers in [Some(1), Some(2), None] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let project = required_project(&root);
        let (executor, transcript) = refusing_session_with_successful_cli(&root);
        assert!(executor.check_with_servers(&project, servers).is_err());
        assert_no_program_cli(&transcript);
    }
}

#[test]
fn required_incremental_session_refusal_is_not_retried_or_counted_as_cli_fallback() {
    let _fallback_guard = super::super::fallback::FALLBACK_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let mut project = required_project(&root);
    let (executor, transcript) = refusing_session_with_successful_cli(&root);
    assert!(
        executor
            .check_incremental_session(&mut project, Some(2))
            .is_err()
    );
    assert_no_program_cli(&transcript);
    let metrics = executor.incremental_metrics();
    assert_eq!(metrics.checks, 1);
    assert_eq!(metrics.session_to_cli_fallbacks, 0);
    assert!(!metrics.last_session_to_cli_fallback);
}
