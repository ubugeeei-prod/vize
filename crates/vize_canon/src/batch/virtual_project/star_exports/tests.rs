//! Same registered source owns admission, projected destination and refresh.
#![expect(
    clippy::disallowed_macros,
    reason = "complete fixture and map packets use std formatting"
)]

use std::{fs, path::Path};

use vize_carton::{String, ToCompactString};

use super::super::VirtualProject;

mod materialized;

const EMPTY: &str =
    "<script setup lang=\"ts\">const value = 1;</script><template>{{ value }}</template>";
const BARREL: &str = "export * from './Panel.vue';\n";
const BASELINE: &str = "export * from './Panel.vue.ts';\n";
const ERASED: &str = "export {} from './Panel.vue.ts';\n";
const BASE_MAP: &str =
    "ImportSourceMap { adjustments: [OffsetAdjustment { original_offset: 15, adjustment: 3 }] }";
const EMPTY_MAP: &str = "ImportSourceMap { adjustments: [OffsetAdjustment { original_offset: 8, adjustment: 1 }, OffsetAdjustment { original_offset: 15, adjustment: 3 }], replacement_ranges: [ReplacementRange { original_start: 7, original_end: 8, virtual_start: 7, virtual_end: 9 }] }";

fn project(root: &Path) -> VirtualProject {
    let mut project = VirtualProject::new(root).unwrap();
    project.scope_batch_namespace(&root.join("cache"));
    project
}

fn register(project: &mut VirtualProject, name: &str, source: &str) {
    let path = project.project_root().join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, source).unwrap();
    project.register_path(&path).unwrap();
}

/// Complete owned generated bodies, maps and original source, without a hash
/// replacing the bytes. Includes any TSX forwarding companion.
fn owner_packet(project: &VirtualProject, name: &str) -> serde_json::Value {
    let original = project.project_root().join(name);
    let owner = project.source_artifacts.get(&original).unwrap();
    let files = owner
        .virtual_paths
        .iter()
        .map(|path| {
            let file = project.virtual_files.get(path).unwrap();
            serde_json::json!({
                "path": path, "original": file.original_path, "code": file.content,
                "map": format!("{:?}", file.source_map),
                "authored": project.original_contents.get(path),
                "preRewrite": project.pre_rewrite_code.get(path),
            })
        })
        .collect::<Vec<_>>();
    let baseline = owner.star_projection.as_ref().map(|base| {
        serde_json::json!({
            "code": base.code, "map": format!("{:?}", base.source_map),
            "stars": format!("{:?}", base.stars),
        })
    });
    serde_json::json!({"original": original, "empty": owner.empty_module_exports,
        "files": files, "baseline": baseline, "passthrough": owner.passthrough_paths})
}

fn assert_barrel(project: &VirtualProject, name: &str, code: &str, map: &str) {
    let file = project
        .find_by_original(&project.project_root().join(name))
        .unwrap();
    assert_eq!(file.content.as_str(), code);
    assert_eq!(format!("{:?}", file.source_map.import_map), map);
    assert_eq!(
        project
            .original_contents
            .get(&file.virtual_path)
            .unwrap()
            .as_str(),
        BARREL
    );
    assert!(file.source_map.sfc_map.is_none());
}

#[test]
fn known_empty_source_refreshes_named_empty_removed_and_recreated_ownership() {
    let root = tempfile::tempdir().unwrap();
    let mut project = project(root.path());
    fs::write(project.project_root().join("Panel.vue"), EMPTY).unwrap();
    register(&mut project, "index.ts", BARREL);
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", BASELINE, BASE_MAP);
    let baseline = project.source_artifacts[&project.project_root().join("index.ts")]
        .star_projection
        .as_ref()
        .unwrap();
    let baseline_packet = (baseline.code.clone(), baseline.source_map.clone());
    register(&mut project, "Panel.vue", EMPTY);
    let empty = owner_packet(&project, "Panel.vue");
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", ERASED, EMPTY_MAP);
    assert_eq!(owner_packet(&project, "Panel.vue"), empty);
    let file = project
        .find_by_original(&project.project_root().join("index.ts"))
        .unwrap();
    assert_eq!(
        (
            file.source_map.import_map.get_original_offset(7),
            file.source_map.import_map.get_original_offset(8)
        ),
        (7, 7)
    );
    for source in [
        "<script lang=\"ts\">export const Props = 1;</script><template />",
        "<script lang=\"ts\">export type Public = string;</script><template />",
    ] {
        register(&mut project, "Panel.vue", source);
        let named = owner_packet(&project, "Panel.vue");
        assert_eq!(named["empty"], false);
        project.finalize_module_imports();
        assert_barrel(&project, "index.ts", BASELINE, BASE_MAP);
        assert_eq!(owner_packet(&project, "Panel.vue"), named);
    }
    register(&mut project, "Panel.vue", EMPTY);
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", ERASED, EMPTY_MAP);
    assert_eq!(owner_packet(&project, "Panel.vue"), empty);
    let original = project.project_root().join("Panel.vue");
    let paths = project.source_artifacts[&original].virtual_paths.clone();
    assert_eq!(project.remove_registered_source(&original), paths);
    assert!(project.find_by_original(&original).is_none());
    assert!(!project.source_artifacts.contains_key(&original));
    assert!(
        !paths
            .iter()
            .any(|path| project.virtual_files.contains_key(path))
    );
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", BASELINE, BASE_MAP);
    register(&mut project, "Panel.vue", EMPTY);
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", ERASED, EMPTY_MAP);
    assert_eq!(owner_packet(&project, "Panel.vue"), empty);
    let retained = project.source_artifacts[&project.project_root().join("index.ts")]
        .star_projection
        .as_ref()
        .unwrap();
    assert_eq!(
        (&retained.code, &retained.source_map),
        (&baseline_packet.0, &baseline_packet.1)
    );
}

#[test]
fn only_same_owner_current_mirror_can_admit_an_explicit_relative_edge() {
    let root = tempfile::tempdir().unwrap();
    let mut project = project(root.path());
    register(&mut project, "Panel.vue", EMPTY);
    register(
        &mut project,
        "nested/index.ts",
        "export * from '../Panel.vue';\n",
    );
    let importer = project
        .find_by_original(&project.project_root().join("nested/index.ts"))
        .unwrap();
    assert!(project.known_empty_vue_target(importer, "../Panel.vue", "../Panel.vue.ts"));
    for (authored, projected) in [
        ("../Panel", "../Panel.vue.ts"),
        ("@panel", "../Panel.vue.ts"),
        ("package/Panel.vue", "../Panel.vue.ts"),
        ("../Missing.vue", "../Panel.vue.ts"),
        ("../Panel.vue", "../Foreign.vue.ts"),
        ("../Panel.vue", "../Panel.vue.tsx"),
        ("../Panel.vue.ts", "../Panel.vue.ts"),
    ] {
        assert!(!project.known_empty_vue_target(importer, authored, projected));
    }
    project.finalize_module_imports();
    assert_eq!(
        project
            .find_by_original(&project.project_root().join("nested/index.ts"))
            .unwrap()
            .content
            .as_str(),
        "export {} from '../Panel.vue.ts';\n"
    );
    register(&mut project, "Panel.vue.ts", "export const Props = 17;\n");
    let importer = project
        .find_by_original(&project.project_root().join("nested/index.ts"))
        .unwrap();
    assert!(project.known_empty_vue_target(importer, "../Panel.vue", "../Panel.vue.ts"));
    // The existing collision alias keeps an ordinary authored companion separate.
    // Removing the physical SFC before re-registering creates a real overwritten
    // mirror; stale source metadata must not authenticate that foreign body.
    fs::remove_file(project.project_root().join("Panel.vue")).unwrap();
    register(&mut project, "Panel.vue.ts", "export const Props = 17;\n");
    let importer = project
        .find_by_original(&project.project_root().join("nested/index.ts"))
        .unwrap();
    assert!(!project.known_empty_vue_target(importer, "../Panel.vue", "../Panel.vue.ts"));
    project.finalize_module_imports();
    assert_eq!(
        project
            .find_by_original(&project.project_root().join("nested/index.ts"))
            .unwrap()
            .content
            .as_str(),
        "export * from '../Panel.vue.ts';\n"
    );
    assert_eq!(
        project
            .find_by_original(&project.project_root().join("Panel.vue.ts"))
            .unwrap()
            .content
            .as_str(),
        "export const Props = 17;\n"
    );
}

#[test]
fn default_only_and_batch_registration_order_preserve_all_direct_consumers() {
    let source = "<script lang=\"ts\">export default { name: 'Panel' };</script><template />";
    let consumer = "import Panel, { type Props, type Emits, type Slots } from './Panel.vue';\nexport * as namespace from './Panel.vue';\nexport { default as PublicPanel } from './Panel.vue';\n";
    let expected = "import Panel, { type Props, type Emits, type Slots } from './Panel.vue.ts';\nexport * as namespace from './Panel.vue.ts';\nexport { default as PublicPanel } from './Panel.vue.ts';\n";
    let mut outcomes = Vec::new();
    for target_first in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let mut project = project(root.path());
        fs::write(project.project_root().join("Panel.vue"), source).unwrap();
        if target_first {
            register(&mut project, "Panel.vue", source);
        }
        register(&mut project, "index.ts", BARREL);
        register(&mut project, "direct.ts", consumer);
        if !target_first {
            register(&mut project, "Panel.vue", source);
        }
        let target = owner_packet(&project, "Panel.vue");
        let direct = owner_packet(&project, "direct.ts");
        assert_eq!(target["empty"], true);
        project.finalize_module_imports();
        assert_barrel(&project, "index.ts", ERASED, EMPTY_MAP);
        assert_eq!(owner_packet(&project, "Panel.vue"), target);
        assert_eq!(owner_packet(&project, "direct.ts"), direct);
        let file = project
            .find_by_original(&project.project_root().join("direct.ts"))
            .unwrap();
        assert_eq!(file.content.as_str(), expected);
        outcomes.push((file.content.clone(), file.source_map.import_map.clone()));
    }
    assert_eq!(outcomes[0], outcomes[1]);
}

#[test]
fn unproven_sfc_modules_and_plain_scripts_keep_the_whole_baseline() {
    for source in [
        "<script lang=\"ts\">export { default } from './external';</script><template />",
        "<script lang=\"ts\">export * from './external';</script><template />",
        "<script lang=\"ts\">export const broken = ;</script><template />",
        "<script src=\"./external.ts\" lang=\"ts\"></script><template />",
        "<script lang=\"coffee\">value = 1</script><template />",
        "<script setup lang=\"ts\">export const Props = 1;</script><template />",
    ] {
        let root = tempfile::tempdir().unwrap();
        let mut project = project(root.path());
        register(&mut project, "external.ts", "export default {};\n");
        register(&mut project, "Panel.vue", source);
        register(&mut project, "index.ts", BARREL);
        let target = owner_packet(&project, "Panel.vue");
        assert_eq!(target["empty"], false, "{source}");
        project.finalize_module_imports();
        assert_barrel(&project, "index.ts", BASELINE, BASE_MAP);
        assert_eq!(owner_packet(&project, "Panel.vue"), target);
    }
    let root = tempfile::tempdir().unwrap();
    let mut project = project(root.path());
    let path = project.project_root().join("Panel.vue");
    fs::write(&path, "export default {};\n").unwrap();
    project
        .register_script_file(&path, "export default {};\n", oxc_span::SourceType::ts())
        .unwrap();
    register(&mut project, "index.ts", BARREL);
    let plain = owner_packet(&project, "Panel.vue");
    assert_eq!(plain["empty"], false);
    project.finalize_module_imports();
    assert_barrel(&project, "index.ts", BASELINE, BASE_MAP);
    assert_eq!(owner_packet(&project, "Panel.vue"), plain);
}

#[test]
fn project_config_routes_and_explicit_modes_do_not_grant_alias_credit() {
    for resolution in ["bundler", "node16", "nodenext"] {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("tsconfig.json"), format!(
            r#"{{"compilerOptions":{{"moduleResolution":"{resolution}","baseUrl":".","paths":{{"@/*":["*"],"@/Panel.vue":["Foreign.vue"]}}}}}}"#)).unwrap();
        let mut project = project(root.path());
        project.set_tsconfig_path(Some(project.project_root().join("tsconfig.json")));
        register(&mut project, "Panel.vue", EMPTY);
        register(
            &mut project,
            "Foreign.vue",
            "<script>export const Props = 1;</script>",
        );
        let authored = "export * from '@/Panel.vue';\nexport * from 'pkg/Panel.vue';\nexport * from './Panel';\n";
        register(&mut project, "routes.ts", authored);
        let before = owner_packet(&project, "routes.ts");
        project.finalize_module_imports();
        assert_eq!(owner_packet(&project, "routes.ts"), before);
        for mode in ["import", "require"] {
            let authored: String = format!(
                "export type * from './Panel.vue' with {{ 'resolution-mode': '{mode}' }};\n"
            )
            .to_compact_string();
            let expected = format!(
                "export type {{}} from './Panel.vue.ts' with {{ 'resolution-mode': '{mode}' }};\n"
            );
            register(&mut project, "explicit.ts", &authored);
            project.finalize_module_imports();
            assert_eq!(
                project
                    .find_by_original(&project.project_root().join("explicit.ts"))
                    .unwrap()
                    .content
                    .as_str(),
                expected
            );
        }
    }
}
