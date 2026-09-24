//! Registry source resolution: installed packages, `--offline`, `npm pack`.

use std::fs;

use super::fixture::{Project, composable_v1, ui_v1, ui_v2};
use super::{ID_TS, lockfile};

#[test]
fn discovers_the_installed_package_registry() {
    let project = Project::new();
    ui_v1(&project.path("node_modules/@vizejs/ui/registry"));
    composable_v1(&project.path("node_modules/@vizejs/composable/registry"));

    let listed = project.json(&[], &["--offline", "list", "--kind", "ui"]);
    assert_eq!(listed["items"][0]["package"], "@vizejs/ui");

    project.run(&["--offline", "pull", "id"]).unwrap();
    assert!(project.path(ID_TS).is_file());
    assert_eq!(lockfile(&project).get("ui", "id").unwrap().version, "1.0.0");

    let status = project.json(&[], &["--offline", "status"]);
    assert_eq!(status["items"][0]["upstream"], "up-to-date");
}

#[test]
fn offline_mode_never_shells_out_to_npm() {
    let project = Project::new();
    ui_v1(&project.path("node_modules/@vizejs/ui/registry"));

    let missing = project
        .run(&["--offline", "pull", "composable:use-toggle"])
        .unwrap_err();
    assert!(
        missing
            .message()
            .contains("@vizejs/composable is not installed"),
        "{missing}"
    );

    let pinned = project
        .run(&["--offline", "pull", "ui:id@2.0.0"])
        .unwrap_err();
    assert!(
        pinned.message().contains("does not match @vizejs/ui@2.0.0"),
        "{pinned}"
    );
}

#[test]
fn explicit_registry_versions_must_match() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));

    let error = project
        .run_with(&["ui"], &["pull", "id@2.0.0"])
        .unwrap_err();

    assert!(
        error
            .message()
            .contains("--registry provides @vizejs/ui@1.0.0"),
        "{error}"
    );
}

#[test]
fn registry_argument_accepts_a_package_directory() {
    let project = Project::new();
    ui_v1(&project.registry("package/registry"));

    let listed = project.json(&["package"], &["list"]);

    assert_eq!(listed["items"][0]["name"], "id");
}

#[test]
fn rejects_registries_with_unsafe_paths_or_unknown_schema() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));
    let manifest_path = project.registry("ui").join("registry.json");
    let manifest = fs::read_to_string(&manifest_path).unwrap();

    fs::write(
        &manifest_path,
        manifest.replace("foundations/id/id.ts", "../../escape.ts"),
    )
    .unwrap();
    let unsafe_path = project.run_with(&["ui"], &["list"]).unwrap_err();
    assert!(
        unsafe_path.message().contains("unsafe registry path"),
        "{unsafe_path}"
    );

    fs::write(
        &manifest_path,
        manifest.replace("\"schemaVersion\": 1", "\"schemaVersion\": 9"),
    )
    .unwrap();
    let future = project.run_with(&["ui"], &["list"]).unwrap_err();
    assert!(
        future.message().contains("uses registry schema 9"),
        "{future}"
    );
}

#[cfg(unix)]
#[test]
fn packs_an_explicit_version_with_npm_when_not_installed() {
    use std::os::unix::fs::PermissionsExt;

    let project = Project::new();
    ui_v1(&project.path("node_modules/@vizejs/ui/registry"));
    // A fake npm: `npm pack <spec> --pack-destination <dir> --silent` tars a
    // prepared v2 package into <dir>, and records the spec it was asked for.
    let package = project.registry("packed/package");
    ui_v2(&package.join("registry"));
    let log = project.registry("npm.log");
    let script = project.registry("fake-npm.sh");
    fs::write(
        &script,
        vize_s0::cstr!(
            "#!/bin/sh\necho \"$2\" >> '{}'\ntar -czf \"$4/vizejs-ui-2.0.0.tgz\" -C '{}' package\n",
            log.display(),
            project.registry("packed").display()
        )
        .as_str(),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let npm = script.to_str().unwrap();

    project
        .run(&["--npm", npm, "pull", "ui:rating@2.0.0"])
        .unwrap();

    assert_eq!(fs::read_to_string(&log).unwrap(), "@vizejs/ui@2.0.0\n");
    assert_eq!(
        lockfile(&project).get("ui", "rating").unwrap().version,
        "2.0.0"
    );
    assert!(
        project
            .path("src/components/vize/families/rating/rating-keys.ts")
            .is_file()
    );

    // The installed version satisfies an unpinned request without npm.
    project.run(&["--npm", npm, "info", "id"]).unwrap();
    project.run(&["--npm", npm, "pull", "id"]).unwrap();
    assert_eq!(fs::read_to_string(&log).unwrap().lines().count(), 1);
}
