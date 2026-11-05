//! #11918: the compiler and checker must share owning-tsconfig conditions.
use super::{RuntimeProp, compile, compile_source, expected, write};
use std::path::Path;

const COMPONENT: &str = "<script setup lang=\"ts\">import type { FooProps } from 'lib'; defineProps<FooProps>();</script>";

fn project(exports: &str) -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    write(
        project.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler","customConditions":["source"]}}"#,
    );
    write(project.path(), "node_modules/lib/package.json", exports);
    write(
        project.path(),
        "node_modules/lib/src/index.ts",
        "export interface FooProps { bar?: Record<string, unknown> }",
    );
    write(
        project.path(),
        "node_modules/lib/src/development.ts",
        "export interface FooProps { bar?: string }",
    );
    write(
        project.path(),
        "node_modules/lib/dist/index.js",
        "export {};",
    );
    project
}

fn string_prop() -> Vec<RuntimeProp> {
    let mut props = expected();
    props[0].constructor = Some("String".into());
    props
}

fn unresolved(root: &Path, source: &str, package: &str) {
    let error = compile_source(root, source).expect_err("inactive or blocked types must not leak");
    assert_eq!(error.code.as_deref(), Some("V_SFC_TYPE_RESOLUTION"));
    assert_eq!(
        error.message,
        vize_carton::cstr!("Cannot resolve imported macro type `FooProps` from `{package}`.")
    );
}

#[test]
fn configured_source_condition_resolves_root_and_nested_export_shapes() {
    for manifest in [
        r#"{"types":"./dist/index.d.ts","exports":{".":{"source":"./src/index.ts","default":"./dist/index.js"}}}"#,
        r#"{"types":"./dist/index.d.ts","exports":{"source":"./src/index.ts","default":"./dist/index.js"}}"#,
        r#"{"exports":{".":{"import":{"source":"./src/index.ts","default":"./dist/index.js"}}}}"#,
        r#"{"exports":{".":{"source":["./missing.ts","./src/index.ts"]}}}"#,
    ] {
        let project = project(manifest);
        assert_eq!(
            compile(project.path(), "app/Probe.vue", "lib", false),
            expected()
        );
    }
}

#[test]
fn inherited_conditions_and_named_custom_conditions_reach_source() {
    let project = project(
        r#"{"exports":{".":{"development":"./src/development.ts","default":"./dist/index.js"}}}"#,
    );
    write(
        project.path(),
        "tsconfig.json",
        r#"{"extends":"./config/shared.json"}"#,
    );
    write(
        project.path(),
        "config/shared.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler","customConditions":["development"]}}"#,
    );
    assert_eq!(
        compile(project.path(), "app/Probe.vue", "lib", false),
        string_prop()
    );
}

#[test]
fn package_key_order_precedes_the_order_of_custom_conditions() {
    let project = project(
        r#"{"exports":{".":{"development":"./src/development.ts","source":"./src/index.ts"}}}"#,
    );
    write(
        project.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler","customConditions":["source","development"]}}"#,
    );
    assert_eq!(
        compile(project.path(), "app/Probe.vue", "lib", false),
        string_prop()
    );
}

#[test]
fn wildcard_source_exports_keep_exact_and_null_subpath_boundaries() {
    let project = project(
        r#"{"exports":{"./*":{"source":"./src/*.ts"},"./exact":{"source":"./src/development.ts"},"./private/*":null}}"#,
    );
    assert_eq!(
        compile(project.path(), "app/Probe.vue", "lib/index", false),
        expected()
    );
    assert_eq!(
        compile(project.path(), "app/Exact.vue", "lib/exact", false),
        string_prop()
    );
    write(
        project.path(),
        "node_modules/lib/private/secret.d.ts",
        "export interface FooProps { secret: string }",
    );
    unresolved(
        project.path(),
        "<script setup lang=\"ts\">import type { FooProps } from 'lib/private/secret'; defineProps<FooProps>();</script>",
        "lib/private/secret",
    );
}

#[test]
fn condition_edits_replace_an_existing_cached_target_and_empty_override_disables_it() {
    let project = project(
        r#"{"exports":{".":{"source":"./src/index.ts","development":"./src/development.ts","default":"./dist/index.js"}}}"#,
    );
    assert_eq!(
        compile(project.path(), "app/Probe.vue", "lib", false),
        expected()
    );
    write(
        project.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler","customConditions":["development"]}}"#,
    );
    assert_eq!(
        compile(project.path(), "app/Probe.vue", "lib", false),
        string_prop()
    );
    write(
        project.path(),
        "config/shared.json",
        r#"{"compilerOptions":{"customConditions":["source"]}}"#,
    );
    write(
        project.path(),
        "tsconfig.json",
        r#"{"extends":"./config/shared.json","compilerOptions":{"moduleResolution":"bundler","customConditions":[]}}"#,
    );
    unresolved(project.path(), COMPONENT, "lib");
    assert!(
        project
            .path()
            .join("node_modules/lib/src/index.ts")
            .is_file()
    );
}

#[test]
fn unconfigured_source_and_explicit_null_condition_do_not_expose_physical_types() {
    let project =
        project(r#"{"exports":{".":{"source":"./src/index.ts","default":"./dist/index.js"}}}"#);
    write(
        project.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
    );
    unresolved(project.path(), COMPONENT, "lib");
    write(
        project.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler","customConditions":["source"]}}"#,
    );
    write(
        project.path(),
        "node_modules/lib/package.json",
        r#"{"types":"./src/index.ts","exports":{".":{"source":null,"default":"./dist/index.js"}}}"#,
    );
    unresolved(project.path(), COMPONENT, "lib");
}

#[test]
fn active_condition_lookup_does_not_borrow_inactive_or_unexported_types() {
    for manifest in [
        r#"{"types":"./src/index.ts","exports":{".":{"inactive":{"types":"./src/index.ts"}}}}"#,
        r#"{"types":"./src/index.ts","exports":{"./private":{"source":"./src/index.ts"}}}"#,
        r#"{"types":"./src/index.ts","exports":{".":{"source":"./missing.ts"}}}"#,
        r#"{"types":"./src/index.ts","exports":{".":{"source":[null,"./src/index.ts"]}}}"#,
    ] {
        let project = project(manifest);
        unresolved(project.path(), COMPONENT, "lib");
    }
}

#[test]
fn pinned_source_condition_project_retains_its_complete_ssr_module() {
    const ROOT: &str =
        "../../../../tests/_fixtures/differential/compiler/macro-export-conditions-11918/";
    let project = tempfile::tempdir().unwrap();
    let source = include_str!(
        "../../../../tests/_fixtures/differential/compiler/macro-export-conditions-11918/Layout.vue.txt"
    );
    for (name, bytes) in [
        (
            "tsconfig.json",
            include_str!(
                "../../../../tests/_fixtures/differential/compiler/macro-export-conditions-11918/tsconfig.json.txt"
            ),
        ),
        (
            "node_modules/lib/package.json",
            include_str!(
                "../../../../tests/_fixtures/differential/compiler/macro-export-conditions-11918/lib.package.json.txt"
            ),
        ),
        (
            "node_modules/lib/src/index.ts",
            include_str!(
                "../../../../tests/_fixtures/differential/compiler/macro-export-conditions-11918/lib.index.ts.txt"
            ),
        ),
        (
            "node_modules/lib/dist/index.js",
            include_str!(
                "../../../../tests/_fixtures/differential/compiler/macro-export-conditions-11918/lib.runtime.js.txt"
            ),
        ),
    ] {
        write(project.path(), name, bytes);
    }
    let filename = project.path().join("Layout.vue");
    write(project.path(), "Layout.vue", source);
    let parse = vize_atelier_sfc::SfcParseOptions {
        filename: filename.to_string_lossy().as_ref().into(),
        ..Default::default()
    };
    let descriptor = vize_atelier_sfc::parse_sfc(source, parse.clone()).unwrap();
    let mut options = vize_atelier_sfc::SfcCompileOptions {
        parse,
        ..Default::default()
    };
    options.template.ssr = true;
    let result = vize_atelier_sfc::compile_sfc(&descriptor, options).unwrap();
    assert_eq!(result.errors.len(), 0, "{ROOT}");
    assert_eq!(
        result.code.as_str(),
        include_str!(
            "../../../../tests/_fixtures/differential/compiler/macro-export-conditions-11918/module.expected.txt"
        )
    );
}
