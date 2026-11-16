//! A solution-style root (`files: []` plus `references`, the create-vue
//! layout) gives each SFC the options of the referenced project that owns it,
//! as `tsc`, `vue-tsc` and Vue's compiler do (#13090).

use std::path::Path;

use super::{compile, compile_source, expected, write};

const PROPS: &str = "export interface FooProps { bar?: Record<string, unknown> }";

fn solution(root: &Path, app: &str) {
    write(root, "src/models/FooProps.ts", PROPS);
    // The node project comes first and also declares `@/*`: only ownership,
    // not reference order or the presence of `paths`, selects the app.
    write(
        root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./tsconfig.node.json"},{"path":"./tsconfig.app.json"}]}"#,
    );
    write(
        root,
        "tsconfig.node.json",
        r#"{"compilerOptions":{"paths":{"@/*":["./node/*"]}},"include":["vite.config.*"]}"#,
    );
    write(root, "tsconfig.app.json", app);
}

#[test]
fn referenced_project_owning_the_sfc_supplies_its_paths_alias() {
    let project = tempfile::tempdir().expect("project");
    solution(
        project.path(),
        r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}},"include":["src/**/*.ts","src/**/*.vue"]}"#,
    );
    assert_eq!(
        compile(project.path(), "src/Alias.vue", "@/models/FooProps", false),
        expected()
    );
    assert_eq!(
        compile(
            project.path(),
            "src/Relative.vue",
            "./models/FooProps",
            true
        ),
        expected()
    );
}

#[test]
fn referenced_project_inherits_paths_anchored_at_the_declaring_config() {
    let project = tempfile::tempdir().expect("project");
    // No `baseUrl` anywhere: the inherited target resolves against the
    // directory of the config that declares `paths`, not the referencing one.
    write(
        project.path(),
        "config/tsconfig.base.json",
        r#"{"compilerOptions":{"paths":{"@/*":["../src/*"]}},"include":["../src/**/*"]}"#,
    );
    solution(
        project.path(),
        r#"{"extends":"./config/tsconfig.base.json","compilerOptions":{"strict":true}}"#,
    );
    assert_eq!(
        compile(
            project.path(),
            "src/Inherited.vue",
            "@/models/FooProps",
            false
        ),
        expected()
    );
}

#[test]
fn a_source_no_referenced_project_owns_keeps_the_solution_root() {
    let project = tempfile::tempdir().expect("project");
    solution(
        project.path(),
        r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}},"include":["src/**/*.ts"]}"#,
    );
    write(project.path(), "app/models/FooProps.ts", PROPS);
    let error = compile_source(project.path(), "<script setup lang=\"ts\">import type { FooProps } from '@/models/FooProps'; defineProps<FooProps>();</script>").expect_err("an unowned SFC has no alias");
    assert_eq!(error.code.as_deref(), Some("V_SFC_TYPE_RESOLUTION"));
}
