//! A command-line host gets the check projection: its diagnostics are the
//! editor projection's, byte for byte.

use super::support::{copy_fixture, install_mapper_manifest, install_vue_package, workspace_root};
use super::{Command, Output, Path, PathBuf, TSGO_ENV};

const MODE_ENV: &str = "VIZE_CONTENT_MAPPER_MODE";

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
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping Content Mapper projection modes: {TSGO_ENV} is not set");
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
