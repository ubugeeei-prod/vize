//! End-to-end `vize lib` tests over fixture registries (no network).

mod fixture;
mod lifecycle;
mod sources;
mod v2;

use fixture::{Project, RATING_TS_V1, RATING_VUE_V1, composable_v1, ui_v1};

use super::lockfile::Lockfile;

const RATING_TS: &str = "src/components/vize/families/rating/rating.ts";
const RATING_VUE: &str = "src/components/vize/families/rating/rating.vue";
const STATE_TS: &str = "src/components/vize/foundations/state/state.ts";
const ID_TS: &str = "src/components/vize/foundations/id/id.ts";

fn lockfile(project: &Project) -> Lockfile {
    Lockfile::read(&project.root, &project.path("vize-lib.lock.json")).unwrap()
}

#[test]
fn list_search_and_info_read_the_registry() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));
    composable_v1(&project.registry("composable"));

    let listed = project.json(&["ui", "composable"], &["list"]);
    let names: Vec<&str> = listed["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        ["id", "rating", "locale", "state", "locale", "use-toggle"]
    );

    let found = project.json(&["ui", "composable"], &["search", "star"]);
    assert_eq!(found["items"][0]["name"], "rating");

    let info = project.json(&["ui"], &["info", "star rating"]);
    assert_eq!(info["name"], "rating");
    assert_eq!(info["version"], "1.0.0");
    assert_eq!(
        info["registryDependencies"],
        serde_json::json!(["id", "state"])
    );

    let human = project
        .run_with(&["composable"], &["info", "useToggle"])
        .unwrap();
    assert!(human.contains("composable:use-toggle"), "{human}");
}

#[test]
fn pull_copies_the_item_and_its_dependency_closure() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));

    let output = project.run_with(&["ui"], &["pull", "rating"]).unwrap();

    assert!(
        output.contains("pull ui:rating 1.0.0 into src/components/vize"),
        "{output}"
    );
    assert_eq!(project.read(RATING_TS), RATING_TS_V1);
    assert_eq!(project.read(RATING_VUE), RATING_VUE_V1);
    assert!(project.path(STATE_TS).is_file());
    assert!(project.path(ID_TS).is_file());
    assert!(
        !project
            .path("src/components/vize/foundations/locale")
            .exists()
    );

    let lock = lockfile(&project);
    let entries: Vec<(&str, bool)> = lock
        .items
        .iter()
        .map(|item| (item.name.as_str(), item.direct))
        .collect();
    assert_eq!(entries, [("id", false), ("rating", true), ("state", false)]);
    let rating = lock.get("ui", "rating").unwrap();
    assert_eq!(rating.version, "1.0.0");
    assert_eq!(rating.package, "@vizejs/ui");
    assert_eq!(rating.dir, "src/components/vize");
    assert_eq!(
        rating
            .files
            .get("families/rating/rating.ts")
            .map(|sha| sha.as_str()),
        Some(super::fs_ops::sha256_hex(RATING_TS_V1.as_bytes()).as_str())
    );

    let again = project.run_with(&["ui"], &["pull", "rating"]).unwrap();
    assert!(again.contains("already up to date"), "{again}");
}

#[test]
fn pull_dry_run_and_json_write_nothing() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));

    let report = project.json(&["ui"], &["pull", "rating", "--dry-run"]);

    assert_eq!(report["dryRun"], true);
    assert_eq!(report["applied"], false);
    assert_eq!(report["items"][0]["name"], "rating");
    assert_eq!(report["items"][0]["files"][0]["action"], "create");
    assert!(!project.path(RATING_TS).exists());
    assert!(!project.path("vize-lib.lock.json").exists());
}

#[test]
fn pull_uses_dir_flag_then_config_then_registry_default() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));
    composable_v1(&project.registry("composable"));
    project.write(
        "vize.config.json",
        r#"{ "lib": { "composableDir": "src/use" } }"#,
    );

    project
        .run_with(&["ui", "composable"], &["pull", "ui:id", "--dir", "src/ui"])
        .unwrap();
    project
        .run_with(&["ui", "composable"], &["pull", "useToggle"])
        .unwrap();

    assert!(project.path("src/ui/foundations/id/id.ts").is_file());
    assert!(project.path("src/use/use-toggle.ts").is_file());
    let error = project
        .run_with(&["ui"], &["pull", "rating", "--dir", "src/elsewhere"])
        .unwrap_err();
    assert!(
        error.message().contains("already pulled into src/ui"),
        "{error}"
    );
    let escape = project
        .run_with(&["composable"], &["pull", "locale", "--dir", "../outside"])
        .unwrap_err();
    assert!(
        escape.message().contains("inside the project root"),
        "{escape}"
    );
}

#[cfg(unix)]
#[test]
fn pull_rejects_symlinked_target_directory_without_writing_outside_project() {
    use std::os::unix::fs::symlink;

    let project = Project::new();
    ui_v1(&project.registry("ui"));
    let outside = project.registries.join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::create_dir_all(project.path("src")).unwrap();
    symlink(&outside, project.path("src/components")).unwrap();

    let error = project.run_with(&["ui"], &["pull", "id"]).unwrap_err();

    assert!(error.message().contains("symbolic link"), "{error}");
    assert!(!outside.join("vize/foundations/id/id.ts").exists());
    assert!(!project.path("vize-lib.lock.json").exists());
}

#[cfg(unix)]
#[test]
fn pull_rejects_symlinked_target_file_even_with_overwrite() {
    use std::os::unix::fs::symlink;

    let project = Project::new();
    ui_v1(&project.registry("ui"));
    let outside = project.registries.join("outside.ts");
    std::fs::write(&outside, "keep me").unwrap();
    std::fs::create_dir_all(project.path("src/components/vize/foundations/id")).unwrap();
    symlink(&outside, project.path(ID_TS)).unwrap();

    let error = project
        .run_with(&["ui"], &["pull", "id", "--overwrite"])
        .unwrap_err();

    assert!(error.message().contains("symbolic link"), "{error}");
    assert_eq!(std::fs::read_to_string(outside).unwrap(), "keep me");
    assert!(!project.path("vize-lib.lock.json").exists());
}

#[cfg(unix)]
#[test]
fn remove_rejects_symlinked_pulled_file_without_deleting_target() {
    use std::os::unix::fs::symlink;

    let project = Project::new();
    ui_v1(&project.registry("ui"));
    project.run_with(&["ui"], &["pull", "id"]).unwrap();
    let outside = project.registries.join("outside.ts");
    std::fs::write(&outside, fixture::ID_V1).unwrap();
    std::fs::remove_file(project.path(ID_TS)).unwrap();
    symlink(&outside, project.path(ID_TS)).unwrap();

    let error = project.run_with(&["ui"], &["remove", "id"]).unwrap_err();

    assert!(error.message().contains("symbolic link"), "{error}");
    assert_eq!(std::fs::read_to_string(outside).unwrap(), fixture::ID_V1);
    assert!(project.path("vize-lib.lock.json").exists());
}

#[cfg(unix)]
#[test]
fn pull_rejects_symlinked_lockfile_before_writing_item() {
    use std::os::unix::fs::symlink;

    let project = Project::new();
    ui_v1(&project.registry("ui"));
    let outside = project.registries.join("outside.lock.json");
    std::fs::write(&outside, "keep me").unwrap();
    symlink(&outside, project.path("vize-lib.lock.json")).unwrap();

    let error = project.run_with(&["ui"], &["pull", "id"]).unwrap_err();

    assert!(error.message().contains("symbolic link"), "{error}");
    assert_eq!(std::fs::read_to_string(outside).unwrap(), "keep me");
    assert!(!project.path(ID_TS).exists());
}

#[test]
fn pull_rejects_lockfile_configured_outside_project() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));
    project.write(
        "vize.config.json",
        r#"{ "lib": { "lockfile": "../outside.lock.json" } }"#,
    );

    let error = project.run_with(&["ui"], &["pull", "id"]).unwrap_err();

    assert!(
        error.message().contains("inside the project root"),
        "{error}"
    );
    assert!(!project.path(ID_TS).exists());
    assert!(!project.registries.join("outside.lock.json").exists());
}

#[test]
fn ambiguous_names_need_a_kind_prefix() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));
    composable_v1(&project.registry("composable"));

    let error = project
        .run_with(&["ui", "composable"], &["pull", "locale"])
        .unwrap_err();
    assert!(
        error.message().contains("ui:locale, composable:locale"),
        "{error}"
    );

    project
        .run_with(&["ui", "composable"], &["pull", "composable:locale"])
        .unwrap();
    assert!(project.path("src/composables/vize/locale.ts").is_file());
}

#[test]
fn pull_refuses_to_clobber_untracked_files_without_overwrite() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));
    project.write(ID_TS, "// my own id helper\n");

    let error = project.run_with(&["ui"], &["pull", "id"]).unwrap_err();
    assert!(
        error.message().contains("foundations/id/id.ts (conflict)"),
        "{error}"
    );
    assert_eq!(project.read(ID_TS), "// my own id helper\n");
    assert!(!project.path("vize-lib.lock.json").exists());

    project
        .run_with(&["ui"], &["pull", "id", "--overwrite"])
        .unwrap();
    assert_eq!(project.read(ID_TS), fixture::ID_V1);
}

#[test]
fn tampered_registry_files_are_rejected() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));
    std::fs::write(
        project.registry("ui").join("files/foundations/id/id.ts"),
        "tampered",
    )
    .unwrap();

    let error = project.run_with(&["ui"], &["pull", "id"]).unwrap_err();

    assert!(
        error
            .message()
            .contains("does not match its registry digest"),
        "{error}"
    );
    assert!(!project.path(ID_TS).exists());
}
