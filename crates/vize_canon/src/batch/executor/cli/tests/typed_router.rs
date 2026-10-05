use std::fs;

use super::super::partition_virtual_files;
use crate::batch::VirtualProject;

fn project(enabled: bool, page: &str, shared: bool) -> (tempfile::TempDir, VirtualProject) {
    let root = tempfile::tempdir().unwrap();
    let plugins = if enabled {
        r#"["vue-router/volar/sfc-typed-router"]"#
    } else {
        "[]"
    };
    fs::write(
        root.path().join("tsconfig.json"),
        format!(r#"{{"vueCompilerOptions":{{"plugins":{plugins}}}}}"#),
    )
    .unwrap();
    if shared {
        fs::write(root.path().join("shared.ts"), "export const value = 1;").unwrap();
    }
    let common = if shared {
        "import { value } from './shared';"
    } else {
        "const value = 1;"
    };
    for index in 0..4 {
        fs::write(root.path().join(format!("Page{index}.vue")), format!(
            "<script setup lang=\"ts\">{common} {}</script><template>{{{{ value }}}}</template>",
            if index == 0 { page } else { "" },
        )).unwrap();
    }
    let mut project = VirtualProject::new(root.path()).unwrap();
    project.set_tsconfig_path(Some(root.path().join("tsconfig.json")));
    let mut files: Vec<_> = fs::read_dir(root.path())
        .unwrap()
        .map(|file| file.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension != "json")
        })
        .collect();
    files.sort();
    project.register_paths(&files).unwrap();
    project.materialize().unwrap();
    (root, project)
}

#[test]
fn only_actual_configured_route_helper_import_joins_native_programs() {
    let route = "const route = useRoute(); void route;";
    let (_root, enabled) = project(true, route, true);
    assert!(enabled.has_typed_router_imports());
    assert!(partition_virtual_files(&enabled, 2).shards.is_empty());
    let (_root, disabled) = project(false, route, true);
    assert!(!disabled.has_typed_router_imports());
    assert_eq!(partition_virtual_files(&disabled, 2).shards.len(), 2);
    for (page, expected_shards) in [
        ("const unrelated = 1; void unrelated;", 2),
        ("const route = useRoute('/known'); void route;", 2),
        // The existing closed leaf domain declines '<' before this new guard.
        ("const route = useRoute<'/known'>(); void route;", 0),
        (
            "const useRoute = () => 1; const route = useRoute(); void route;",
            2,
        ),
        (
            "const definePage = (value: unknown) => value; definePage({});",
            2,
        ),
    ] {
        let (_root, project) = project(true, page, true);
        assert!(!project.has_typed_router_imports(), "{page}");
        assert_eq!(
            partition_virtual_files(&project, 2).shards.len(),
            expected_shards,
            "{page}"
        );
    }
}

#[test]
fn generated_global_visibility_also_joins_independent_components_without_leaves() {
    let route = "const route = useRoute(); void route;";
    let (_root, enabled) = project(true, route, false);
    assert!(enabled.has_typed_router_imports());
    assert!(partition_virtual_files(&enabled, 2).shards.is_empty());
    let (_root, disabled) = project(false, route, false);
    assert!(!disabled.has_typed_router_imports());
    assert_eq!(partition_virtual_files(&disabled, 2).shards.len(), 2);
}
