use super::{
    TSCONFIG, build, host_source, package_fixture, package_manifest, request_path,
    selected_companion,
};

#[test]
fn same_project_hosts_keep_one_union_snapshot_across_sequential_opens() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("workspace");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::write(app.join("tsconfig.json"), TSCONFIG).unwrap();
    let mut hosts = Vec::new();
    for (lane, prop, ty, value) in [
        ("alpha", "alpha", "string", "'ok'"),
        ("bravo", "bravo", "number", "1"),
    ] {
        let lane_root = app.join("src").join(lane);
        let host = lane_root.join("Host.vue");
        let package = lane_root.join("node_modules/@scope/ui");
        std::fs::create_dir_all(package.join("src")).unwrap();
        std::fs::write(package.join("package.json"), package_manifest(prop, ty)).unwrap();
        std::fs::write(
            package.join("src/Internal.vue"),
            format!("<script setup lang=\"ts\">defineProps<{{ {prop}: {ty} }}>()</script>\n"),
        )
        .unwrap();
        std::fs::write(
            package.join("src/Conditional.vue"),
            format!(
                "<script setup lang=\"ts\">import Internal from '#internal'; void Internal; defineProps<{{ {prop}: {ty} }}>()</script>\n"
            ),
        )
        .unwrap();
        std::fs::write(
            package.join("src/Fallback.vue"),
            "<script setup lang=\"ts\">defineProps<{ fallback: Date }>()</script>\n",
        )
        .unwrap();
        hosts.push((host, host_source(prop, value)));
    }

    let alpha = build(&hosts[0].0, &hosts[0].1);
    let alpha_request = request_path(&alpha);
    let alpha_companion = selected_companion(&alpha_request);
    assert!(alpha_request.is_file());
    assert!(alpha_companion.is_file());

    let bravo = build(&hosts[1].0, &hosts[1].1);
    let bravo_request = request_path(&bravo);
    let bravo_companion = selected_companion(&bravo_request);
    assert!(bravo_request.is_file());
    assert!(bravo_companion.is_file());
    assert!(
        alpha_request.is_file(),
        "second host pruned the first query: alpha={} alpha_root={:?} bravo={} bravo_root={:?}",
        alpha_request.display(),
        alpha.session_project_root,
        bravo_request.display(),
        bravo.session_project_root,
    );
    assert!(
        alpha_companion.is_file(),
        "second host pruned the first importer-local package shadow"
    );

    let alpha_again = build(&hosts[0].0, &hosts[0].1);
    assert_eq!(request_path(&alpha_again), alpha_request);
    assert!(
        bravo_request.is_file(),
        "reverse open pruned the second query"
    );
    assert!(
        bravo_companion.is_file(),
        "reverse open pruned the second package shadow"
    );
}

#[test]
fn same_mtime_package_source_edit_refreshes_every_shadow_companion() {
    let fixture = package_fixture("refresh", "alpha", "string");
    let source = host_source("alpha", "'ok'");
    let initial = build(&fixture.host, &source);
    let companion = selected_companion(&request_path(&initial));
    let before = std::fs::read_to_string(&companion).unwrap();
    let component = fixture.package.join("src/Conditional.vue");
    let modified = std::fs::metadata(&component).unwrap().modified().unwrap();
    let changed = std::fs::read_to_string(&component)
        .unwrap()
        .replace("alpha", "omega");
    assert_eq!(
        changed.len(),
        std::fs::read_to_string(&component).unwrap().len()
    );
    std::fs::write(&component, changed).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&component)
        .unwrap()
        .set_modified(modified)
        .unwrap();

    let refreshed = build(&fixture.host, &source);
    let after = std::fs::read_to_string(selected_companion(&request_path(&refreshed))).unwrap();
    assert_ne!(before, after);
    assert!(after.contains("omega"));
    assert!(
        refreshed.materialized_changes.changed.contains(&companion),
        "the package companion change must be forwarded to the live Corsa session: {:?}",
        refreshed.materialized_changes
    );
}
