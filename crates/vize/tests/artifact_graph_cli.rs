use std::{fs, process::Command};

#[test]
fn graph_command_executes_compiler_lint_and_typecheck_from_one_snapshot() {
    let directory = tempfile::tempdir().unwrap();
    let sfc = directory.path().join("App.vue");
    let tsx = directory.path().join("Card.tsx");
    fs::write(
        &sfc,
        r#"<script setup lang="ts">
const ready = true
</script>
<template><main v-if="ready">ready</main></template>"#,
    )
    .unwrap();
    fs::write(
        &tsx,
        r#"const ready = true; export const Card = () => ready ? <p>yes</p> : <p>no</p>;"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .arg("graph")
        .arg(&sfc)
        .arg(&tsx)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema"], "vize.artifact-graph.execution");
    assert_eq!(report["snapshotSourceCount"], 2);
    let files = report["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    assert_ne!(files[0]["sourceId"], files[1]["sourceId"]);

    for file in files {
        assert!(file["compilerBytes"].as_u64().unwrap() > 0);
        let is_vue = file["path"]
            .as_str()
            .is_some_and(|path| path.ends_with(".vue"));
        assert_eq!(file["virtualTsBytes"].as_u64().is_some(), is_vue);
        let products = file["products"].as_array().unwrap();
        for required in [
            "croquis.document",
            "rendu.hir",
            "backend.dom-module",
            "patina.document-report",
        ] {
            assert!(
                products
                    .iter()
                    .any(|product| product["product"] == required),
                "missing {required} in {products:?}"
            );
        }
        assert!(
            products
                .iter()
                .all(|product| product["product"] != "flow.graph"),
            "compiler/lint/typecheck must not fabricate an unrequested Flow product"
        );
        assert_eq!(
            products
                .iter()
                .filter(|product| product["product"] == "croquis.document")
                .count(),
            1
        );
        assert_eq!(
            products
                .iter()
                .any(|product| product["product"] == "canon.sfc-typecheck"),
            is_vue
        );
        assert!(
            products
                .iter()
                .all(|product| product["sourceId"] == file["sourceId"])
        );
    }

    let tsx_products = files[1]["products"].as_array().unwrap();
    assert!(
        tsx_products
            .iter()
            .all(|product| product["product"] != "relief.syntax")
    );
    assert!(
        tsx_products
            .iter()
            .all(|product| product["product"] != "relief.transformed")
    );

    let counters = report["counters"].as_array().unwrap();
    let semantics = counters
        .iter()
        .find(|counter| counter["product"] == "croquis.document")
        .unwrap();
    assert_eq!(semantics["executions"], 2);
    assert_eq!(semantics["queries"], 3);
}

#[test]
fn graph_command_accepts_a_project_only_cross_source_request() {
    let directory = tempfile::tempdir().unwrap();
    let app = directory.path().join("App.vue");
    let card = directory.path().join("Card.tsx");
    fs::write(&app, "<template><Card /></template>").unwrap();
    fs::write(&card, "export const Card = () => <article />;").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args([
            "graph",
            "--project",
            "--no-compiler",
            "--no-lint",
            "--no-typecheck",
        ])
        .arg(&app)
        .arg(&card)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["snapshotSourceCount"], 2);
    assert_eq!(report["files"].as_array().unwrap().len(), 1);
    let products = report["files"][0]["products"].as_array().unwrap();
    assert!(
        products
            .iter()
            .any(|product| product["product"] == "croquis.project")
    );
    let semantic_sources: std::collections::BTreeSet<_> = products
        .iter()
        .filter(|product| product["product"] == "croquis.semantics")
        .map(|product| product["sourceId"].as_u64().unwrap())
        .collect();
    assert_eq!(semantic_sources.len(), 2);
}

#[test]
fn graph_command_routes_vapor_through_rendu() {
    let directory = tempfile::tempdir().unwrap();
    let sfc = directory.path().join("App.vue");
    fs::write(
        &sfc,
        r#"<template><main v-if="ready">ready</main></template>"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["graph", "--target", "vapor", "--no-lint", "--no-typecheck"])
        .arg(&sfc)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let file = &report["files"][0];
    assert!(file["compilerBytes"].is_null());
    assert!(file["vaporBlocks"].as_u64().unwrap() > 0);
    let products = file["products"].as_array().unwrap();
    for required in ["rendu.hir", "backend.vapor-plan"] {
        assert!(
            products
                .iter()
                .any(|product| product["product"] == required),
            "missing {required} in {products:?}"
        );
    }
    assert!(
        products
            .iter()
            .all(|product| product["product"] != "backend.dom-module")
    );
}
