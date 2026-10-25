//! These payloads are unchanged members of the two actual same-version packs.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use vize_carton::{FxHashMap, String};

use super::export_graph::{ExportGraph, Kind};

fn payloads(kind: &str, root: &Path) -> FxHashMap<PathBuf, String> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/typechecker/experimental-router-project-capability");
    let authority: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.join("artifacts.json")).unwrap()).unwrap();
    let provider = authority["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|value| value["kind"] == kind)
        .unwrap();
    assert_eq!(provider["name"], "vue-router");
    assert_eq!(provider["version"], "5.4.0");
    provider["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let path = entry["path"].as_str().unwrap();
            let bytes =
                std::fs::read(fixture.join(kind).join(vize_carton::cstr!("{path}.txt"))).unwrap();
            assert_eq!(bytes.len() as u64, entry["bytes"].as_u64().unwrap());
            assert_eq!(
                super::super::super::identity::encode_digest(Sha256::digest(&bytes)),
                entry["sha256"].as_str().unwrap()
            );
            (
                root.join(path),
                String::from(std::str::from_utf8(&bytes).unwrap()),
            )
        })
        .collect()
}

#[test]
fn simultaneous_real_same_version_packages_have_distinct_type_capabilities() {
    let published_root = Path::new("/project/published/node_modules/vue-router");
    let preview_root = Path::new("/project/preview/node_modules/vue-router");
    let published_files = payloads("published", published_root);
    let preview_files = payloads("preview", preview_root);
    let published_reads = RefCell::new(Vec::new());
    let preview_reads = RefCell::new(Vec::new());
    let mut published = ExportGraph::new(published_root, |path: &Path| {
        published_reads.borrow_mut().push(path.to_path_buf());
        Ok(published_files.get(path).cloned())
    });
    let mut preview = ExportGraph::new(preview_root, |path: &Path| {
        preview_reads.borrow_mut().push(path.to_path_buf());
        Ok(preview_files.get(path).cloned())
    });
    let published_entry = published_root.join("dist/experimental/index.d.ts");
    let preview_entry = preview_root.join("dist/experimental/index.d.ts");
    assert!(
        published
            .exported(&published_entry, "_RoutePropsForFilePath")
            .unwrap()
            .is_none()
    );
    let helper = preview
        .exported(&preview_entry, "_RoutePropsForFilePath")
        .unwrap()
        .unwrap();
    assert_eq!(helper.kind, Kind::Alias);
    assert_eq!(
        helper.sha256,
        "9d2b61dbab3330606dcc98dd0c78a9bb95ac5ea593eb0b764d0d43ab93912a6b"
    );
    assert_eq!(helper.parameters.len(), 1);
    assert!(helper.parameters.first().unwrap().string_constraint);
    let published_page = published
        .exported(&published_entry, "definePage")
        .unwrap()
        .unwrap();
    let preview_page = preview
        .exported(&preview_entry, "definePage")
        .unwrap()
        .unwrap();
    assert_eq!(published_page.kind, Kind::Function);
    assert_eq!(preview_page.kind, Kind::Function);
    assert_eq!(published_page.parameters.len(), 1);
    assert_eq!(preview_page.parameters.len(), 2);
    assert!(preview_page.parameters.first().unwrap().string_constraint);
    assert!(preview_page.parameters.last().unwrap().unconstrained);
    assert_eq!(
        published_page.path,
        published_root.join("dist/index-u4XkZVDG.d.ts")
    );
    assert_eq!(helper.path, preview_root.join("dist/index-BA_Jc-EQ.d.ts"));
    assert_eq!(
        preview_page.path,
        preview_root.join("dist/index-BA_Jc-EQ.d.ts")
    );
    assert_eq!(
        &preview_files.get(&helper.path).unwrap()
            [helper.start_byte as usize..helper.end_byte as usize],
        "type RoutePropsForFilePath<FilePath extends string> = [RouteNamesForFilePath<FilePath>] extends [never] ? RouteParamsGeneric : RouteMap[RouteNamesForFilePath<FilePath>]['params'];"
    );
    let counts = (published_reads.borrow().len(), preview_reads.borrow().len());
    assert!(
        published
            .exported(&published_entry, "_RoutePropsForFilePath")
            .unwrap()
            .is_none()
    );
    assert_eq!(
        preview
            .exported(&preview_entry, "_RoutePropsForFilePath")
            .unwrap(),
        Some(helper)
    );
    assert_eq!(
        (published_reads.borrow().len(), preview_reads.borrow().len()),
        counts,
        "warm symbol queries perform no reads"
    );
    assert_eq!(
        *published_reads.borrow(),
        [
            published_entry,
            published_root.join("dist/index-u4XkZVDG.ts"),
            published_root.join("dist/index-u4XkZVDG.tsx"),
            published_root.join("dist/index-u4XkZVDG.d.ts"),
        ]
    );
    assert_eq!(
        *preview_reads.borrow(),
        [
            preview_entry,
            preview_root.join("dist/index-BA_Jc-EQ.ts"),
            preview_root.join("dist/index-BA_Jc-EQ.tsx"),
            preview_root.join("dist/index-BA_Jc-EQ.d.ts"),
        ]
    );
}

#[test]
fn actual_type_reexport_and_declaration_removal_do_not_reuse_an_earlier_graph() {
    let root = Path::new("/project/preview/node_modules/vue-router");
    let mut files = payloads("preview", root);
    let entry = root.join("dist/experimental/index.d.ts");
    let original = files.get(&entry).unwrap().clone();
    let original_target = {
        let mut graph = ExportGraph::new(root, |path: &Path| Ok(files.get(path).cloned()));
        graph
            .exported(&entry, "_RoutePropsForFilePath")
            .unwrap()
            .unwrap()
    };
    files.insert(root.join("dist/experimental/original.d.ts"), original);
    files.insert(entry.clone(), "export * from './original.js';\n".into());
    let reexported = {
        let mut graph = ExportGraph::new(root, |path: &Path| Ok(files.get(path).cloned()));
        let helper = graph
            .exported(&entry, "_RoutePropsForFilePath")
            .unwrap()
            .unwrap();
        assert_eq!(graph.counts.parsed_files, 3);
        helper
    };
    assert_eq!(
        reexported, original_target,
        "the alias still reaches the actual original type declaration"
    );
    files.remove(&original_target.path);
    let mut graph = ExportGraph::new(root, |path: &Path| Ok(files.get(path).cloned()));
    assert_eq!(
        graph
            .exported(&entry, "_RoutePropsForFilePath")
            .unwrap_err()
            .code,
        "missing-reexport-target"
    );
}

#[test]
fn a_real_source_payload_shadowing_the_declaration_is_not_a_dts_winner() {
    let root = Path::new("/project/preview/node_modules/vue-router");
    let mut files = payloads("preview", root);
    let entry = root.join("dist/experimental/index.d.ts");
    let target = {
        let mut graph = ExportGraph::new(root, |path: &Path| Ok(files.get(path).cloned()));
        graph
            .exported(&entry, "_RoutePropsForFilePath")
            .unwrap()
            .unwrap()
            .path
    };
    let source_name = target.to_str().unwrap().strip_suffix(".d.ts").unwrap();
    files.insert(
        PathBuf::from(vize_carton::cstr!("{source_name}.ts")),
        files.get(&target).unwrap().clone(),
    );
    let mut graph = ExportGraph::new(root, |path: &Path| Ok(files.get(path).cloned()));
    assert_eq!(
        graph
            .exported(&entry, "_RoutePropsForFilePath")
            .unwrap_err()
            .code,
        "source-shadows-declaration"
    );
}
