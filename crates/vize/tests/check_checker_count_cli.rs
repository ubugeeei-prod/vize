//! #7325: explicit checker width reaches each Corsa CLI process.
#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

fn write_project(root: &Path) -> std::io::Result<()> {
    fs::write(
        root.join("package.json"),
        r#"{"name":"checker-width-test","private":true}"#,
    )?;
    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"Bundler"},"include":["*.vue"]}"#,
    )?;
    for name in ["First.vue", "Second.vue"] {
        fs::write(
            root.join(name),
            include_str!("fixtures/import-registration/Child.vue"),
        )?;
    }
    fs::write(
        root.join("corsa.sh"),
        r#"#!/bin/sh
previous=''
for arg in "$@"; do
  if [ "$previous" = '--checkers' ]; then
    printf '%s\n' "$arg" >> "$VIZE_TEST_CHECKER_LOG"
  fi
  previous="$arg"
done
exit 0
"#,
    )?;
    fs::set_permissions(root.join("corsa.sh"), fs::Permissions::from_mode(0o755))
}

#[test]
fn cli_checker_count_overrides_environment_for_single_and_sharded_programs() {
    for (explicit, environment, servers, expected) in [
        (None, None, "1", "1"),
        (None, Some("6"), "1", "6"),
        (Some("4"), Some("1"), "1", "4"),
        (Some("4"), Some("1"), "2", "4"),
    ] {
        let case = tempfile::tempdir().unwrap();
        let root = case.path();
        write_project(root).unwrap();
        let log = root.join("checkers.log");
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command
            .current_dir(root)
            .env_remove("VIZE_CHECKERS")
            .env("VIZE_TEST_CHECKER_LOG", &log)
            .args([
                "check",
                "--no-config",
                "--format",
                "json",
                "--servers",
                servers,
                "--corsa-path",
            ])
            .arg(root.join("corsa.sh"));
        if let Some(value) = environment {
            command.env("VIZE_CHECKERS", value);
        }
        if let Some(value) = explicit {
            command.args(["--checkers", value]);
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "{output:?}");
        let counts = fs::read_to_string(log).unwrap();
        let expected_counts = vec![expected; if servers == "2" { 2 } else { 1 }];
        assert_eq!(
            counts.lines().collect::<Vec<_>>(),
            expected_counts,
            "each disjoint shard receives the selected worker count"
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["errorCount"], 0);
    }
}

#[test]
fn invalid_checker_counts_fail_before_starting_corsa() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path();
    write_project(root).unwrap();
    let log = root.join("checkers.log");
    for value in ["0", "-1", "many"] {
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root)
            .env("VIZE_TEST_CHECKER_LOG", &log)
            .args(["check", "--checkers"])
            .arg(value)
            .arg("--corsa-path")
            .arg(root.join("corsa.sh"))
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(!log.exists());
    }
}
