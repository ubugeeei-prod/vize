//! A command-line host gets the check projection: its diagnostics are the
//! editor projection's, byte for byte.

use super::support::{copy_fixture, install_mapper_manifest, install_vue_package, workspace_root};
use super::{Command, Output, Path, PathBuf, TSGO_ENV};

const MODE_ENV: &str = "VIZE_CONTENT_MAPPER_MODE";
/// The projection cohort's own host. The shared conformance host carries
/// companion requirements (Vue package, JavaScript tsc) this cohort does not.
const PROJECTION_TSGO_ENV: &str = "VIZE_TEST_PROJECTION_CONTENT_MAPPER_TSGO";

/// A required CI cohort fails closed; an unconfigured local run remains optional.
fn projection_host() -> Option<PathBuf> {
    select_projection_host(
        std::env::var_os(PROJECTION_TSGO_ENV)
            .or_else(|| std::env::var_os(TSGO_ENV))
            .map(PathBuf::from),
        std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some(),
    )
}

fn select_projection_host(host: Option<PathBuf>, required: bool) -> Option<PathBuf> {
    let Some(host) = host else {
        assert!(
            !required,
            "unexecuted projection-mode cohort: VIZE_TEST_REQUIRE_TSGO is set and both {PROJECTION_TSGO_ENV} and {TSGO_ENV} are absent"
        );
        eprintln!(
            "skipping Content Mapper projection modes: neither {PROJECTION_TSGO_ENV} nor {TSGO_ENV} is set"
        );
        return None;
    };
    assert!(
        host.is_file(),
        "selected projection-mode Content Mapper host is not a file: {}",
        host.display()
    );
    Some(host)
}

fn check(tsgo: &Path, project: &Path, mode: Option<&str>) -> Output {
    let mut command = Command::new(tsgo);
    command.current_dir(project).args([
        "--runExternalCode",
        "--noEmit",
        "-p",
        "tsconfig.json",
        "--pretty",
        "false",
    ]);
    match mode {
        Some(mode) => command.env(MODE_ENV, mode),
        None => command.env_remove(MODE_ENV),
    };
    command.output().unwrap()
}

#[test]
fn check_projection_reports_the_editor_diagnostics() {
    let Some(tsgo) = projection_host() else {
        return;
    };
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    // Each fixture reports diagnostics, or it would compare nothing.
    for fixture in [
        "content_mapper_projection_modes",
        "content_mapper_diagnostic_parity",
        "content_mapper_unknown_options_10218",
    ] {
        let project = tempfile::Builder::new()
            .prefix("content-mapper-projection-modes-")
            .tempdir_in(&cases_root)
            .unwrap();
        copy_fixture(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(fixture),
            project.path(),
        );
        install_mapper_manifest(project.path());
        install_vue_package(project.path());

        let editor = check(&tsgo, project.path(), Some("editor"));
        let editor_stdout = std::str::from_utf8(&editor.stdout).unwrap();
        assert_ne!(
            editor_stdout.matches(": error TS").count(),
            0,
            "{fixture}: {editor_stdout}"
        );
        // `-p` alone selects the check projection; the override forces it.
        for mode in [None, Some("check")] {
            let check = check(&tsgo, project.path(), mode);
            assert_eq!(
                std::str::from_utf8(&check.stdout).unwrap(),
                editor_stdout,
                "{fixture} {mode:?}"
            );
            assert_eq!(check.status.code(), editor.status.code(), "{fixture}");
        }
    }
}

#[test]
fn optional_projection_host_without_a_provider_remains_unexecuted() {
    assert_eq!(select_projection_host(None, false), None);
}

#[test]
#[should_panic(expected = "unexecuted projection-mode cohort: VIZE_TEST_REQUIRE_TSGO is set")]
fn required_projection_host_without_a_provider_fails_closed() {
    let _ = select_projection_host(None, true);
}

#[test]
#[should_panic(expected = "selected projection-mode Content Mapper host is not a file")]
fn missing_projection_host_fails_before_the_parity_run() {
    let root = tempfile::tempdir().unwrap();
    let _ = select_projection_host(Some(root.path().join("missing-tsgo")), true);
}
