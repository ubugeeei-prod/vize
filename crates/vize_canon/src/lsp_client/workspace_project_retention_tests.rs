use super::*;
use crate::{batch::virtual_project::MaterializedFileDelta, file_uri::path_to_file_uri};
use std::{fs, path::PathBuf};

fn project() -> (tempfile::TempDir, CorsaProjectClient, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    fs::write(root.join("__vize_vue_modules.d.ts"), "export {};\n").unwrap();
    let config = root.join("tsconfig.json");
    fs::write(&config, "{\"files\":[]}\n").unwrap();
    let mut client = CorsaProjectClient::empty_for_test(root);
    client.explicit_project_config = Some(config.clone());
    (directory, client, config)
}

fn overlay(client: &mut CorsaProjectClient, path: &Path, source: &str) -> String {
    fs::write(path, source).unwrap();
    let uri = path_to_file_uri(path);
    client.document_texts.insert(uri.clone(), source.into());
    uri
}

#[test]
fn topology_refresh_retains_only_unchanged_canonical_bytes_and_identity() {
    let (_directory, mut client, config) = project();
    let root = client.project_root.clone();
    let kept = overlay(
        &mut client,
        &root.join("kept.vue.ts"),
        "export const kept = 1;\n",
    );
    let changed = root.join("changed.vue.ts");
    let created = root.join("recreated.vue.ts");
    let deleted = root.join("deleted.vue.ts");
    for path in [&changed, &created, &deleted] {
        overlay(&mut client, path, "export const previous = 1;\n");
    }
    let drifted = root.join("drifted.vue.ts");
    overlay(&mut client, &drifted, "export const previous = 1;\n");
    fs::write(&drifted, "export const changed = 2;\n").unwrap();
    let absent = root.join("absent.vue.ts");
    overlay(&mut client, &absent, "export {};\n");
    fs::remove_file(absent).unwrap();
    let foreign = tempfile::tempdir().unwrap();
    overlay(
        &mut client,
        &foreign.path().join("foreign.ts"),
        "export {};\n",
    );
    client
        .document_texts
        .insert("untitled:untitled".into(), "export {};\n".into());

    let delta = MaterializedFileDelta {
        changed: vec![changed],
        created: vec![created],
        deleted: vec![deleted],
        topology_changed: true,
    };
    assert_eq!(
        client.unchanged_canon_overlays(&root, Some(&config), &delta),
        FxHashMap::from_iter([(kept, "export const kept = 1;\n".into())])
    );
}

#[test]
fn root_configuration_mode_and_non_canon_transitions_drop_all_prior_overlays() {
    let (_directory, mut client, config) = project();
    let root = client.project_root.clone();
    let uri = overlay(&mut client, &root.join("current.vue.ts"), "export {};\n");
    let delta = MaterializedFileDelta::default();
    let expected = FxHashMap::from_iter([(uri, "export {};\n".into())]);
    assert_eq!(
        client.unchanged_canon_overlays(&root, Some(&config), &delta),
        expected
    );
    let other = tempfile::tempdir().unwrap();
    assert_eq!(
        client.unchanged_canon_overlays(other.path(), Some(&config), &delta),
        FxHashMap::default()
    );
    assert_eq!(
        client.unchanged_canon_overlays(&root, Some(&root.join("other.json")), &delta),
        FxHashMap::default()
    );
    client.explicit_project_config = None;
    assert_eq!(
        client.unchanged_canon_overlays(&root, Some(&config), &delta),
        expected
    );
    assert_eq!(
        client.unchanged_canon_overlays(&root, Some(&root.join("other.json")), &delta),
        FxHashMap::default()
    );
    client.explicit_project_config = Some(config.clone());
    client.materialized_project_session = true;
    assert_eq!(
        client.unchanged_canon_overlays(&root, Some(&config), &delta),
        FxHashMap::default()
    );
    client.materialized_project_session = false;
    fs::remove_file(root.join("__vize_vue_modules.d.ts")).unwrap();
    assert_eq!(
        client.unchanged_canon_overlays(&root, Some(&config), &delta),
        FxHashMap::default()
    );
}

#[cfg(unix)]
#[test]
fn matching_bytes_at_a_symlink_identity_do_not_retain_a_foreign_overlay() {
    let (_directory, mut client, config) = project();
    let root = client.project_root.clone();
    let foreign = tempfile::tempdir().unwrap();
    let original = foreign.path().join("original.ts");
    fs::write(&original, "export {};\n").unwrap();
    let link = root.join("link.ts");
    std::os::unix::fs::symlink(original, &link).unwrap();
    client
        .document_texts
        .insert(path_to_file_uri(&link), "export {};\n".into());
    assert_eq!(
        client.unchanged_canon_overlays(&root, Some(&config), &MaterializedFileDelta::default()),
        FxHashMap::default()
    );
}
