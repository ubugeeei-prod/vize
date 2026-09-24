//! status / diff / update / remove over a v1 -> v2 fixture upgrade.

use super::fixture::{Project, RATING_KEYS_V2, RATING_TS_V2, RATING_VUE_V1, ui_v1, ui_v2};
use super::{ID_TS, RATING_TS, RATING_VUE, STATE_TS, lockfile};

const RATING_KEYS: &str = "src/components/vize/families/rating/rating-keys.ts";

fn pulled_v1() -> Project {
    let project = Project::new();
    ui_v1(&project.registry("v1"));
    ui_v2(&project.registry("v2"));
    project.run_with(&["v1"], &["pull", "rating"]).unwrap();
    project
}

#[test]
fn status_reports_local_edits_missing_files_and_upstream_versions() {
    let project = pulled_v1();

    let clean = project.json(&["v1"], &["status"]);
    assert_eq!(clean["items"][1]["name"], "rating");
    assert_eq!(clean["items"][1]["state"], "clean");
    assert_eq!(clean["items"][1]["upstream"], "up-to-date");

    project.write(RATING_VUE, "<template>mine</template>\n");
    std::fs::remove_file(project.path(ID_TS)).unwrap();
    let dirty = project.json(&["v2"], &["status"]);
    let items = dirty["items"].as_array().unwrap();
    assert_eq!(items[0]["name"], "id");
    assert_eq!(items[0]["state"], "missing");
    assert_eq!(items[1]["state"], "modified");
    assert_eq!(items[1]["upstream"], "update-available");
    assert_eq!(items[1]["registryVersion"], "2.0.0");
    assert_eq!(items[2]["upstream"], "up-to-date");

    let human = project.run_with(&["v2"], &["status"]).unwrap();
    assert!(
        human.contains("ui:rating 1.0.0 modified, update available (2.0.0)"),
        "{human}"
    );
    assert!(
        human.contains("  modified families/rating/rating.vue"),
        "{human}"
    );
}

#[test]
fn diff_renders_local_against_the_registry_version() {
    let project = pulled_v1();
    project.write(RATING_VUE, "<template>mine</template>\n");

    let diff = project.run_with(&["v2"], &["diff", "rating"]).unwrap();

    assert!(
        diff.contains("--- a/families/rating/rating.ts (local)"),
        "{diff}"
    );
    assert!(
        diff.contains("+++ b/families/rating/rating.ts (@vizejs/ui@2.0.0)"),
        "{diff}"
    );
    assert!(diff.contains("-export const size = 5;"), "{diff}");
    assert!(diff.contains("+export const size = 10;"), "{diff}");
    assert!(
        diff.contains("+++ b/families/rating/rating-keys.ts"),
        "{diff}"
    );
    assert!(diff.contains("-<template>mine</template>"), "{diff}");

    let report = project.json(&["v2"], &["diff", "rating"]);
    let files = report["files"].as_array().unwrap();
    let vue = files
        .iter()
        .find(|file| file["path"] == "families/rating/rating.vue")
        .unwrap();
    assert_eq!(vue["local"], "modified");
    assert_eq!(vue["upstreamChanged"], false);

    let same = project.run_with(&["v1"], &["diff", "state"]).unwrap();
    assert!(same.contains("ui:state matches @vizejs/ui@1.0.0"), "{same}");
}

#[test]
fn update_applies_upstream_changes_and_keeps_local_edits() {
    let project = pulled_v1();
    // Local edit to a file upstream did not change: must survive the update.
    project.write(RATING_VUE, "<template>mine</template>\n");

    let output = project.run_with(&["v2"], &["update"]).unwrap();

    assert!(
        output.contains("update ui:rating 1.0.0 -> 2.0.0"),
        "{output}"
    );
    assert!(
        output.contains("~ families/rating/rating.ts (update)"),
        "{output}"
    );
    assert!(
        output.contains("+ families/rating/rating-keys.ts (create)"),
        "{output}"
    );
    assert!(
        output.contains("* families/rating/rating.vue (keep-local)"),
        "{output}"
    );
    assert_eq!(project.read(RATING_TS), RATING_TS_V2);
    assert_eq!(project.read(RATING_KEYS), RATING_KEYS_V2);
    assert_eq!(project.read(RATING_VUE), "<template>mine</template>\n");
    assert!(
        project
            .path("src/components/vize/foundations/locale/locale.ts")
            .is_file()
    );

    let lock = lockfile(&project);
    let rating = lock.get("ui", "rating").unwrap();
    assert_eq!(rating.version, "2.0.0");
    assert!(rating.direct);
    assert!(!lock.get("ui", "locale").unwrap().direct);
    assert_eq!(
        rating
            .files
            .get("families/rating/rating.vue")
            .map(|sha| sha.as_str()),
        Some(super::super::fs_ops::sha256_hex(RATING_VUE_V1.as_bytes()).as_str()),
        "the merge base stays the pristine upstream digest"
    );
}

#[test]
fn update_refuses_conflicting_edits_unless_forced() {
    let project = pulled_v1();
    project.write(RATING_TS, "export const size = 7;\n");

    let preview = project
        .run_with(&["v2"], &["update", "rating", "--dry-run"])
        .unwrap();
    assert!(
        preview.contains("! families/rating/rating.ts (conflict)"),
        "{preview}"
    );
    assert!(preview.contains("conflicts (need --force)"), "{preview}");

    let error = project
        .run_with(&["v2"], &["update", "rating"])
        .unwrap_err();
    assert!(error.message().contains("re-run with --force"), "{error}");
    assert_eq!(project.read(RATING_TS), "export const size = 7;\n");
    assert!(
        !project.path(RATING_KEYS).exists(),
        "nothing is written on refusal"
    );
    assert_eq!(
        lockfile(&project).get("ui", "rating").unwrap().version,
        "1.0.0"
    );

    project
        .run_with(&["v2"], &["update", "rating", "--force"])
        .unwrap();
    assert_eq!(project.read(RATING_TS), RATING_TS_V2);
}

#[test]
fn update_deletes_pristine_files_dropped_upstream() {
    let project = Project::new();
    ui_v2(&project.registry("v2"));
    ui_v1(&project.registry("v1"));
    project.run_with(&["v2"], &["pull", "rating"]).unwrap();
    assert!(project.path(RATING_KEYS).is_file());

    let output = project.run_with(&["v1"], &["update", "rating"]).unwrap();

    assert!(
        output.contains("- families/rating/rating-keys.ts (delete)"),
        "{output}"
    );
    assert!(!project.path(RATING_KEYS).exists());
}

#[test]
fn remove_deletes_items_and_orphaned_dependencies() {
    let project = pulled_v1();

    let blocked = project.run_with(&["v1"], &["remove", "id"]).unwrap_err();
    assert!(
        blocked.message().contains("required by rating, state"),
        "{blocked}"
    );

    project.write(RATING_VUE, "<template>mine</template>\n");
    let refused = project
        .run_with(&["v1"], &["remove", "rating"])
        .unwrap_err();
    assert!(
        refused
            .message()
            .contains("ui:rating families/rating/rating.vue"),
        "{refused}"
    );
    assert!(project.path(RATING_TS).is_file());

    let preview = project
        .run_with(&["v1"], &["remove", "rating", "--dry-run"])
        .unwrap();
    assert!(
        preview.contains("would remove ui:state (orphaned dependency)"),
        "{preview}"
    );

    project
        .run_with(&["v1"], &["remove", "rating", "--force"])
        .unwrap();
    assert!(!project.path(RATING_VUE).exists());
    assert!(!project.path(STATE_TS).exists());
    assert!(!project.path(ID_TS).exists());
    assert!(
        !project.path("src/components/vize").exists(),
        "empty directories are pruned"
    );
    assert!(!project.path("vize-lib.lock.json").exists());
}
