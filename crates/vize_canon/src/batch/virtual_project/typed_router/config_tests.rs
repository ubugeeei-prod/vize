#![expect(
    clippy::disallowed_types,
    reason = "configuration fixtures use std strings"
)]

use super::*;
use std::fs;
use tempfile::TempDir;

fn project(config: &str) -> (TempDir, VirtualProject) {
    let root = TempDir::new().unwrap();
    fs::write(root.path().join("tsconfig.json"), config).unwrap();
    let mut project = VirtualProject::new(root.path()).unwrap();
    project.set_tsconfig_path(Some(root.path().join("tsconfig.json")));
    (root, project)
}

#[test]
fn only_explicit_known_plugin_enables_typed_pages() {
    for config in [
        "{}",
        r#"{"vueCompilerOptions":{"plugins":[]}}"#,
        r#"{"vueCompilerOptions":{"plugins":["other-plugin"]}}"#,
        r#"{"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-route-blocks"]}}"#,
        r#"{"vueCompilerOptions":{"plugins":[["vue-router/volar/sfc-typed-router",{}]]}}"#,
        r#"{"vueCompilerOptions":{"plugins":[{"name":"vue-router/volar/sfc-typed-router","options":{"rootDir":7}}]}}"#,
    ] {
        let (_root, project) = project(config);
        assert!(project.typed_router.root.is_none(), "{config}");
    }
    let (_root, project) =
        project(r#"{"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#);
    assert_eq!(
        project.typed_router.root.as_ref(),
        Some(&project.project_root)
    );
}

#[test]
fn plugin_root_overrides_effective_compiler_root() {
    let (_root, project) = project(
        r#"{"compilerOptions":{"rootDir":"src"},"vueCompilerOptions":{"plugins":[{"name":"vue-router/volar/sfc-typed-router","options":{"rootDir":"pages"}}]}}"#,
    );
    assert_eq!(
        project.typed_router.root,
        Some(project.project_root.join("pages"))
    );
    let (_root, project) = project(
        r#"{"compilerOptions":{"rootDir":"src"},"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#,
    );
    assert_eq!(
        project.typed_router.root,
        Some(project.project_root.join("src"))
    );
}

#[test]
fn extends_plugins_follow_replacement_and_clear_rules() {
    let (root, mut project) = project(r#"{"extends":"./base.json"}"#);
    fs::write(
        root.path().join("base.json"),
        r#"{"compilerOptions":{"rootDir":"src"},"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#,
    ).unwrap();
    project.refresh_compiler_configuration();
    assert_eq!(
        project.typed_router.root,
        Some(project.project_root.join("src"))
    );
    fs::write(
        root.path().join("tsconfig.json"),
        r#"{"extends":"./base.json","vueCompilerOptions":{"plugins":[]}}"#,
    )
    .unwrap();
    project.refresh_compiler_configuration();
    assert!(project.typed_router.root.is_none());
}

#[test]
fn file_identity_is_relative_and_literal_safe_without_route_name_guessing() {
    let root = TempDir::new().unwrap();
    let path = root.path().join("src/pages/users/[id=int].vue");
    assert_eq!(
        file_literal(root.path(), &path).unwrap(),
        "\"src/pages/users/[id=int].vue\""
    );
    let escaped = root.path().join("src/pages/a'\"\n\u{2028}.vue");
    let literal = file_literal(root.path(), &escaped).unwrap();
    assert_eq!(
        serde_json::from_str::<std::string::String>(&literal).unwrap(),
        "src/pages/a'\"\n\u{2028}.vue"
    );
    assert!(literal.contains("\\u2028"));
    assert_eq!(
        file_literal(
            &root.path().join("src/pages"),
            &root.path().join("components/Outside.vue")
        )
        .unwrap(),
        "\"../../components/Outside.vue\""
    );
    assert!(file_literal(Path::new("relative"), &path).is_none());
}

#[test]
fn registered_import_need_is_exact_and_removed_on_replacement_or_delete() {
    let (_root, mut project) =
        project(r#"{"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#);
    let path = project.project_root.join("src/pages/users/[id=int].vue");
    project.register_vue_file(&path, "<script setup lang=\"ts\">import { useRoute } from 'vue-router'; const route = useRoute(); void route;</script>").unwrap();
    assert!(project.has_typed_router_imports());
    assert_eq!(project.typed_router.import_files.len(), 1);
    project
        .register_vue_file(
            &path,
            "<script setup lang=\"ts\">const value = 1; void value;</script>",
        )
        .unwrap();
    assert!(!project.has_typed_router_imports());
    project
        .register_vue_file(&path, "<template>{{ $route.params.id }}</template>")
        .unwrap();
    assert!(project.has_typed_router_imports());
    let file = project
        .virtual_files_sorted()
        .into_iter()
        .find(|file| file.original_path == path)
        .unwrap();
    assert!(
        file.content
            .contains("ReturnType<typeof import('vue-router').useRoute<")
    );
    assert!(!file.content.contains("const $route: __Global<'$route'"));
    project.remove_registered_source(&path);
    assert!(!project.has_typed_router_imports());
}

#[test]
fn disabled_and_configured_but_unused_files_remain_byte_exact() {
    let (_left_root, mut left) = project("{}");
    let (_right_root, mut right) =
        project(r#"{"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#);
    let path = left.project_root.join("src/components/Unrelated.vue");
    let source =
        "<script setup lang=\"ts\">const count = 1;</script><template>{{ count }}</template>";
    left.register_vue_file(&path, source).unwrap();
    // Unused plugin configuration introduces no filename or route-helper bytes.
    let right_path = right.project_root.join("src/components/Unrelated.vue");
    right.register_vue_file(&right_path, source).unwrap();
    let l = left.virtual_files_sorted()[0].content.as_str();
    let r = right.virtual_files_sorted()[0].content.as_str();
    assert_eq!(l, r);
    assert!(!left.has_typed_router_imports() && !right.has_typed_router_imports());
}
