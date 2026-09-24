//! `vize lib pull --with-examples`.

use std::fs;

use serde_json::{Value, json};

use super::super::fs_ops::sha256_hex;
use super::fixture::{Project, ui_v1};
use super::lockfile;

const EXAMPLE: &str = "families/rating/examples/rating-basic.vue";
const EXAMPLE_SOURCE: &str = "<!-- Five-star rating. -->\n<script setup lang=\"ts\">\nimport { Rating } from \"../rating.ts\";\n</script>\n\n<template>\n  <Rating aria-label=\"Score\" />\n</template>\n";

/// ui@1.0.0 whose `rating` item ships one example.
fn registry_with_example(project: &Project) {
    let dir = project.registry("ui");
    ui_v1(&dir);
    let target = dir.join("files").join(EXAMPLE);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, EXAMPLE_SOURCE).unwrap();
    let manifest_path = dir.join("registry.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    for item in manifest["items"].as_array_mut().unwrap() {
        let examples = if item["name"] == "rating" {
            json!([{
                "path": EXAMPLE,
                "title": "Basic",
                "description": "Five-star rating.",
                "sha256": sha256_hex(EXAMPLE_SOURCE.as_bytes()).as_str(),
                "size": EXAMPLE_SOURCE.len(),
            }])
        } else {
            json!([])
        };
        item["examples"] = examples;
    }
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

#[test]
fn examples_are_pulled_only_on_request_and_tracked_in_the_lockfile() {
    let project = Project::new();
    registry_with_example(&project);
    let example_path = "src/components/vize/families/rating/examples/rating-basic.vue";

    project.run_with(&["ui"], &["pull", "rating"]).unwrap();
    assert!(!project.path(example_path).exists());
    assert!(!lockfile(&project).get("ui", "rating").unwrap().examples);

    let output = project
        .run_with(&["ui"], &["pull", "rating", "--with-examples"])
        .unwrap();
    assert!(
        output.contains("+ families/rating/examples/rating-basic.vue (create)"),
        "{output}"
    );
    assert_eq!(project.read(example_path), EXAMPLE_SOURCE);
    let lock = lockfile(&project);
    let rating = lock.get("ui", "rating").unwrap();
    assert!(rating.examples);
    assert!(rating.files.contains_key(EXAMPLE));
    assert!(
        !lock.get("ui", "state").unwrap().examples,
        "dependencies stay example-free"
    );

    // Later updates keep pulling the examples without the flag.
    fs::remove_file(project.path(example_path)).unwrap();
    project.run_with(&["ui"], &["update", "rating"]).unwrap();
    assert!(project.path(example_path).is_file());

    project.run_with(&["ui"], &["remove", "rating"]).unwrap();
    assert!(!project.path(example_path).exists());
}

#[test]
fn tampered_or_colliding_examples_are_rejected() {
    let project = Project::new();
    registry_with_example(&project);
    fs::write(
        project.registry("ui").join("files").join(EXAMPLE),
        "tampered",
    )
    .unwrap();
    let error = project
        .run_with(&["ui"], &["pull", "rating", "--with-examples"])
        .unwrap_err();
    assert!(
        error
            .message()
            .contains("does not match its registry digest"),
        "{error}"
    );

    let manifest_path = project.registry("ui").join("registry.json");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    fs::write(
        &manifest_path,
        manifest.replace(EXAMPLE, "families/rating/rating.ts"),
    )
    .unwrap();
    let collision = project.run_with(&["ui"], &["list"]).unwrap_err();
    assert!(
        collision.message().contains("duplicate file"),
        "{collision}"
    );
}
