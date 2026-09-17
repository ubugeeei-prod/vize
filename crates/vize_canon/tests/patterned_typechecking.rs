use std::path::Path;
use vize_canon::{
    BatchTypeChecker, BatchTypeCheckerTrait, SfcBlockType, SfcTypeCheckOptions, type_check_sfc,
};

#[path = "patterned_typechecking/component_props.rs"]
mod component_props;

fn write(root: &Path, name: &str, source: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}

fn project() -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    write(
        project.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"bundler","noEmit":true,"skipLibCheck":true},"include":["src/**/*"]}"#,
    );
    write(
        project.path(),
        "node_modules/vue/package.json",
        r#"{"name":"vue","types":"index.d.ts"}"#,
    );
    write(
        project.path(),
        "node_modules/vue/index.d.ts",
        "export interface Ref<T = unknown> { value: T }\nexport function ref<T>(value: T): Ref<T>;\nexport interface VNodeProps {}\nexport interface AllowedComponentProps {}\nexport interface ComponentCustomProps {}",
    );
    write(
        project.path(),
        "src/types.ts",
        "export type Result<T> = { kind: 'ok'; rows: T[] } | { kind: 'err'; message: string };",
    );
    write(
        project.path(),
        "src/Child.vue",
        r#"<script setup lang="ts">defineProps<{ count: number }>()</script><template><div/></template>"#,
    );
    project
}

const APP: &str = r#"<script setup lang="ts">
import type { Result } from './types';
import Child from './Child.vue';
const result = {} as Result<number>;
</script>
<template v-match="result">
  <template v-when="{ kind: 'ok', const rows, ...const rest } as whole if (rows.length &gt; 0)">
    {{ whole.rows.length }} {{ result.rows.length }} {{ rest }}
    <Child v-for="rows in rows" :count="rows" />
  </template>
  <p v-when="_">Fallback</p>
</template>"#;

#[test]
fn root_pattern_generates_typed_scopes_and_authored_mappings() {
    let mut options = SfcTypeCheckOptions::new("App.vue").with_virtual_ts();
    let off = type_check_sfc(APP, &options);
    assert!(off.has_errors());
    assert!(off.virtual_ts.is_none());
    options.experimental_patterned_template = true;
    let on = type_check_sfc(APP, &options);
    assert!(on.diagnostics.is_empty(), "{:?}", on.diagnostics);
    let code = on.virtual_ts.unwrap();
    insta::assert_snapshot!("patterned_root_virtual_ts", code);
    assert_eq!(
        on.virtual_ts_helpers,
        Some(include_str!(
            "../src/virtual_ts/helpers/pattern_matching.d.ts"
        ))
    );
    for name in ["rows", "rest", "whole"] {
        assert!(
            on.virtual_ts_mappings.iter().any(|mapping| {
                APP.get(mapping.src_range.clone()) == Some(name)
                    && code.get(mapping.gen_range.clone()) == Some(name)
            }),
            "missing authored binding mapping for {name}: {code}"
        );
    }
}

#[test]
fn native_checker_narrows_imported_generics_and_replays_component_scopes() {
    let project = project();
    write(project.path(), "src/App.vue", APP);
    let mut checker = BatchTypeChecker::new(project.path()).unwrap();
    checker.set_experimental_patterned_template(true);
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    assert!(result.success, "{result:#?}");
}

#[test]
fn missing_coverage_and_invalid_member_access_are_authored_errors() {
    let project = project();
    write(
        project.path(),
        "src/App.vue",
        r#"<script setup lang="ts">
import type { Result } from './types';
const result = {} as Result<number>;
</script>
<template v-match="result">
  <p v-when="{ kind: 'ok', const rows }">{{ rows.toUpperCase() }}</p>
</template>"#,
    );
    let mut checker = BatchTypeChecker::new(project.path()).unwrap();
    checker.set_experimental_patterned_template(true);
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.file.ends_with("App.vue"))
        .collect();
    assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code == Some(2339) && d.line == 5)
    );
    assert!(diagnostics.iter().any(|d| d.code == Some(2322)
        && d.line == 4
        && d.column == 19
        && d.block_type == Some(SfcBlockType::Template)));
}

#[test]
fn unreachable_is_warning_but_guarded_coverage_is_an_error() {
    let project = project();
    write(
        project.path(),
        "src/Unreachable.vue",
        r#"<script setup lang="ts">const state = 'ok' as 'ok' | 'err';</script>
<template v-match="state">
<p v-when="'ok'">First</p>
<p v-when="'ok'">Unreachable</p>
<p v-when="'err'">Last</p>
</template>"#,
    );
    write(
        project.path(),
        "src/Guarded.vue",
        r#"<script setup lang="ts">const state = 'ok' as 'ok' | 'err'; const guard = true as boolean;</script>
<template v-match="state">
<p v-when="'ok' if (guard)">Guarded</p>
<p v-when="'err'">Last</p>
</template>"#,
    );
    let mut checker = BatchTypeChecker::new(project.path()).unwrap();
    checker.set_experimental_patterned_template(true);
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    assert_eq!(result.diagnostics.len(), 2, "{:#?}", result.diagnostics);
    let mut actual: Vec<_> = result
        .diagnostics
        .iter()
        .map(|d| {
            (
                d.file.file_name().unwrap().to_str().unwrap(),
                d.code,
                d.severity,
                d.line,
            )
        })
        .collect();
    actual.sort_unstable();
    assert_eq!(
        actual,
        [
            ("Guarded.vue", Some(2322), 1, 1),
            ("Unreachable.vue", Some(2322), 2, 3)
        ]
    );
}

#[test]
fn warning_only_native_check_succeeds() {
    let project = project();
    write(
        project.path(),
        "src/Warning.vue",
        r#"<script setup lang="ts">const value = 'a' as 'a' | 'b';</script>
<template v-match="value"><p v-when="'a'"/><p v-when="'a'"/><p v-when="'b'"/></template>"#,
    );
    let mut checker = BatchTypeChecker::new(project.path()).unwrap();
    checker.set_experimental_patterned_template(true);
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    assert!(result.success, "{result:#?}");
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.diagnostics[0].severity, 2);
}

#[test]
fn pattern_warnings_do_not_hide_backend_errors() {
    let project = project();
    write(
        project.path(),
        "src/Warning.vue",
        r#"<script setup lang="ts">const value = 'a' as 'a' | 'b';</script>
<template v-match="value"><p v-when="'a'"/><p v-when="'a'"/><p v-when="'b'"/></template>"#,
    );
    write(
        project.path(),
        "src/error.ts",
        "export const value: number = 'wrong';",
    );
    let mut checker = BatchTypeChecker::new(project.path()).unwrap();
    checker.set_experimental_patterned_template(true);
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    assert!(!result.success, "{result:#?}");
    assert_ne!(result.exit_code, 0);
    let mut actual: Vec<_> = result
        .diagnostics
        .iter()
        .map(|d| {
            (
                d.file.file_name().unwrap().to_str().unwrap(),
                d.code,
                d.severity,
            )
        })
        .collect();
    actual.sort_unstable();
    assert_eq!(
        actual,
        [("Warning.vue", Some(2322), 2), ("error.ts", Some(2322), 1)]
    );
}

#[test]
fn native_scope_matrix_keeps_bindings_and_control_flow() {
    let cases = [
        (
            "Value",
            "const state = 'ok' as 'ok' | 'err'; const expected = 'ok' as const;",
            r#"<template v-match="state"><p v-when="expected as expected">{{ expected.toUpperCase() }}</p><p v-when="'err'"/></template>"#,
        ),
        (
            "Guard",
            "const state = undefined as string | undefined;",
            r#"<template v-match="state"><p v-when="const text if (text !== undefined)">{{ text.toUpperCase() }}</p><p v-when="_"/></template>"#,
        ),
        (
            "ParentGuard",
            "const result = undefined as Result<number> | undefined;",
            r#"<template><div v-if="result"><template v-match="result.kind"><p v-when="'ok'"/><p v-when="'err'"/></template></div></template>"#,
        ),
        (
            "Loop",
            "const rows = [] as Result<number>[];",
            r#"<template><template v-for="row in rows"><template v-match="row"><p v-when="{ kind: 'ok', const rows }">{{ rows[0].toFixed() }}{{ row.rows.length }}</p><p v-when="{ kind: 'err', const message }">{{ message.toUpperCase() }}</p></template></template></template>"#,
        ),
        (
            "Nested",
            "const result = {} as Result<'a' | 'b'>;",
            r#"<template v-match="result"><template v-when="{ kind: 'ok', const rows }"><template v-match="rows[0]"><p v-when="'a'"/><p v-when="'b'"/></template></template><p v-when="{ kind: 'err', const message }">{{ message }}</p></template>"#,
        ),
        (
            "Rest",
            "const tuple = [] as unknown as readonly [number, string, boolean];",
            r#"<template v-match="tuple"><p v-when="[const head, ...const tail]">{{ head.toFixed() }}{{ tail[0].toUpperCase() }}{{ tail[1].valueOf() }}</p></template>"#,
        ),
        (
            "Matrix",
            "const tuple = [] as unknown as ['a' | 'b', 1 | 2];",
            r#"<template v-match="tuple"><p v-when="['a', 1]"/><p v-when="['a', 2]"/><p v-when="['b', 1]"/><p v-when="['b', 2]"/></template>"#,
        ),
        (
            "Slots",
            "const Host = {} as { readonly __vizeSlots?: { default: (props: { row: Result<number> }) => unknown } };",
            r#"<template><Host v-slot="{ row }"><template v-match="row"><p v-when="{ kind: 'ok', const rows }">{{ rows[0].toFixed() }}</p><p v-when="{ kind: 'err', const message }">{{ message.toUpperCase() }}</p></template></Host></template>"#,
        ),
    ];
    let project = project();
    for (name, script, template) in cases {
        let source = vize_s0::cstr!(
            "<script setup lang=\"ts\">import type {{ Result }} from './types'; {script}</script>{template}"
        );
        write(project.path(), &vize_s0::cstr!("src/{name}.vue"), &source);
    }
    let mut checker = BatchTypeChecker::new(project.path()).unwrap();
    checker.set_experimental_patterned_template(true);
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    assert!(result.diagnostics.is_empty(), "{result:#?}");
    assert!(result.success, "unmapped backend failure: {result:#?}");
}

#[test]
fn nested_flag_off_is_not_silently_unchecked() {
    let source = r#"<script setup>const subject = true;</script><template><div v-match="subject"><p v-when="_"/></div></template>"#;
    let result = type_check_sfc(source, &SfcTypeCheckOptions::new("Off.vue"));
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|d| d.message.as_str())
        .collect();
    assert_eq!(
        diagnostics,
        [
            "`v-match` / `v-when` require `experimentals.patternedTemplate`.",
            "`v-match` / `v-when` require `experimentals.patternedTemplate`.",
            "Undefined reference '_' in template expression",
        ]
    );
}
