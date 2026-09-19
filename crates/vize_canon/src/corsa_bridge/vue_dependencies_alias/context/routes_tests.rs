//! Native package route and declaration ownership regressions.

use super::RouteDiscovery;
use crate::corsa_bridge::vue_dependencies_alias::AliasContext;
use vize_carton::FxHashMap;

const HOST_SOURCE: &str = "<template><div /></template>\n";
const CLASS_HOST_SOURCE: &str = r#"<script lang="ts">
import { Vue } from "vue-property-decorator";
export const DecoratorBase = Vue;
</script>
"#;

#[test]
fn generated_runtime_support_never_creates_editor_route_state() {
    let fixture = runtime_package_fixture();
    let settings =
        crate::batch::virtual_project::package_resolution::PackageResolutionSettings::default();
    let mut resolver = crate::PackageRouteResolver::default();
    let mut routes = FxHashMap::default();
    let mut reachability = FxHashMap::default();
    let mut bindings = Vec::new();
    let mut inputs = Vec::new();
    let aliases = Vec::new();
    let mut discovery = RouteDiscovery::new(
        &settings,
        &mut resolver,
        &mut routes,
        &mut reachability,
        &mut bindings,
        &mut inputs,
        &aliases,
    );

    for specifier in ["vue", "@vue/runtime-dom", "vite/client"] {
        assert!(!discovery.resolve(
            &fixture.host,
            specifier,
            crate::PackageResolutionMode::Import,
        ));
    }
    assert!(routes.is_empty());
    assert!(reachability.is_empty());
    assert!(bindings.is_empty());
    assert!(inputs.is_empty());
}

#[test]
fn generated_vue_helpers_do_not_expand_the_editor_project() {
    let fixture = runtime_package_fixture();
    let context = AliasContext::for_host(&fixture.host, HOST_SOURCE, &FxHashMap::default());

    assert!(context.aliases.is_empty());
    assert!(context.package_routes.is_empty());
    assert!(context.route_inputs.is_empty());
    assert_eq!(
        context
            .mirror
            .as_ref()
            .unwrap()
            .registered_original_paths_sorted(),
        vec![vize_carton::path::canonicalize_non_verbatim(&fixture.host)]
    );
}

#[test]
fn runtime_support_dependency_chain_stays_on_the_native_package_path() {
    let fixture = runtime_package_fixture();
    let context = AliasContext::for_host(&fixture.host, CLASS_HOST_SOURCE, &FxHashMap::default());

    assert!(context.aliases.is_empty());
    assert!(context.package_routes.is_empty());
    assert_eq!(
        context
            .mirror
            .as_ref()
            .unwrap()
            .registered_original_paths_sorted(),
        vec![vize_carton::path::canonicalize_non_verbatim(&fixture.host)]
    );
    assert!(
        context
            .route_inputs
            .iter()
            .any(|path| { path.ends_with("node_modules/vue-property-decorator/index.d.ts") })
    );
    assert!(
        context
            .route_inputs
            .iter()
            .any(|path| { path.ends_with("node_modules/vue-class-component/index.d.ts") })
    );
    assert!(
        !context
            .route_inputs
            .iter()
            .any(|path| path.ends_with("node_modules/vue/RuntimeOnly.vue"))
    );
}

#[test]
fn editor_paths_declaration_is_inferred_inside_the_mirror() {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("src/App.vue");
    let declaration = root.path().join("src/api/remote-search.d.ts");
    write(
        &root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}}}"#,
    );
    write(
        &host,
        "<script>import { transactionList } from '@/api/remote-search';\nexport default { methods: { load() { return transactionList() } } }\n</script>\n",
    );
    write(
        &declaration,
        "export declare function transactionList(): Promise<unknown>;\n",
    );

    let source = std::fs::read_to_string(&host).unwrap();
    let context = AliasContext::for_host(&host, &source, &FxHashMap::default());
    let mirror = context.mirror.as_ref().expect("paths require a mirror");
    let materialized = mirror
        .preferred_materialized_path_for_original(&declaration)
        .expect("reachable declaration must be mirrored");

    assert_eq!(materialized.file_name().unwrap(), "remote-search.d.ts");
    assert!(
        mirror.expected_materialized_files().contains(&materialized),
        "{}",
        materialized.display()
    );
    assert!(!mirror.is_declaration_root(&declaration));
}

struct RuntimePackageFixture {
    _root: tempfile::TempDir,
    host: std::path::PathBuf,
}

fn runtime_package_fixture() -> RuntimePackageFixture {
    let root = tempfile::tempdir().unwrap();
    let host = root.path().join("src/App.vue");
    let vue = root.path().join("node_modules/vue");
    write(&host, HOST_SOURCE);
    write(
        &root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
    );
    write(
        &vue.join("package.json"),
        r#"{"name":"vue","exports":"./index.ts"}"#,
    );
    write(
        &vue.join("index.ts"),
        "export { default as RuntimeOnly } from './RuntimeOnly.vue';\n",
    );
    write(&vue.join("RuntimeOnly.vue"), "<template />\n");
    let class_component = root.path().join("node_modules/vue-class-component");
    write(
        &class_component.join("package.json"),
        r#"{"name":"vue-class-component","types":"./index.d.ts"}"#,
    );
    write(
        &class_component.join("index.d.ts"),
        "import type { Component } from 'vue';\nexport declare const Vue: Component;\n",
    );
    let property_decorator = root.path().join("node_modules/vue-property-decorator");
    write(
        &property_decorator.join("package.json"),
        r#"{"name":"vue-property-decorator","types":"./index.d.ts"}"#,
    );
    write(
        &property_decorator.join("index.d.ts"),
        "export { Vue } from 'vue-class-component';\n",
    );
    RuntimePackageFixture { _root: root, host }
}

fn write(path: &std::path::Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
