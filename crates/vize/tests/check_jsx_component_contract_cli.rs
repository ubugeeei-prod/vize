#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/jsx_check_project.rs"]
mod jsx_check_project;

use std::path::{Path, PathBuf};

use jsx_check_project::{CheckOutput, JsxCheckProject};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn assert_broken(project: &JsxCheckProject, source: &str, expected: &str) -> CheckOutput {
    project.write("src/Consumer.tsx", source);
    let output = project.check();
    assert!(!output.success, "invalid TSX passed:\n{}", output.stdout);
    assert!(
        output.stdout.contains("src/Consumer.tsx") && output.stdout.contains(expected),
        "stdout:\n{}\nstderr:\n{}",
        output.stdout,
        output.stderr
    );
    output
}

fn assert_clean(output: CheckOutput) {
    assert!(
        output.success && output.stdout.contains(r#""errorCount": 0"#),
        "stdout:\n{}\nstderr:\n{}",
        output.stdout,
        output.stderr
    );
}

#[test]
fn check_tsx_enforces_imported_sfc_and_local_component_contracts() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = JsxCheckProject::new("tsx", corsa_path, false);
    project.write(
        "src/Counter.vue",
        r#"<script setup lang="ts">
defineProps<{ count: number; isOpened: boolean; 'data-id': number; label?: string }>();
defineEmits<{ select: [value: number] }>();
</script>
<template><span>{{ count }}</span></template>
"#,
    );
    project.write(
        "src/Generic.vue",
        r#"<script setup lang="ts" generic="T = string">
defineProps<{ value: T }>();
</script>
"#,
    );

    let wrong_value = assert_broken(
        &project,
        r#"import Counter from './Counter.vue';
export const view = <Counter count="wrong" is-opened data-id={1} />;
"#,
        "not assignable",
    );
    assert!(
        wrong_value.stdout.contains("error:2:30"),
        "{}",
        wrong_value.stdout
    );
    let missing_prop = assert_broken(
        &project,
        r#"import Counter from './Counter.vue';
export const view = <Counter count={1} data-id={1} />;
"#,
        "isOpened",
    );
    assert!(
        missing_prop.stdout.contains("error:2:22"),
        "{}",
        missing_prop.stdout
    );
    assert_broken(
        &project,
        r#"import Counter from './Counter.vue';
export const view = <Counter count={1} is-opened />;
"#,
        "dataId",
    );
    assert_broken(
        &project,
        r#"import Counter from './Counter.vue';
export const view = <Counter count={1} is-opened data-id={1} typo />;
"#,
        "typo",
    );
    assert_broken(
        &project,
        r#"import Counter from './Counter.vue';
const props = { count: 'wrong', isOpened: true, 'data-id': 1 };
export const view = <Counter {...props} />;
"#,
        "not assignable",
    );
    assert_broken(
        &project,
        r#"import Counter from './Counter.vue';
export const view = <Counter count={1} is-opened data-id={1} onSelect={(value) => value.toUpperCase()} />;
"#,
        "toUpperCase",
    );
    assert_broken(
        &project,
        r#"import Counter from './Counter.vue';
const Library = { Counter };
export const view = <Library.Counter count="wrong" is-opened data-id={1} />;
"#,
        "not assignable",
    );
    assert_broken(
        &project,
        r#"const Local = (_props: { count: number }) => null;
export const view = <Local count="wrong" />;
"#,
        "not assignable",
    );

    project.write(
        "src/Consumer.tsx",
        r#"import Counter from './Counter.vue';
import Generic from './Generic.vue';
const Library = { Counter };
const props = { count: 1, isOpened: true, 'data-id': 1 };
export const generic = <Generic value={123} />;
export const view = (
  <Library.Counter
    {...props}
    class="counter"
    style="color: red"
    onSelect={(value) => value.toFixed(0)}
  />
);
"#,
    );
    assert_clean(project.check());
}

#[test]
fn check_jsx_with_check_js_enforces_imported_sfc_props_and_repair() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project = JsxCheckProject::new("jsx", corsa_path, true);
    project.write(
        "src/Counter.vue",
        r#"<script setup lang="ts">
defineProps<{ count: number }>();
</script>
<template><span>{{ count }}</span></template>
"#,
    );
    project.write(
        "src/Consumer.jsx",
        r#"import Counter from './Counter.vue';
export const view = <Counter count="wrong" />;
"#,
    );
    let broken = project.check();
    assert!(!broken.success, "invalid JSX passed:\n{}", broken.stdout);
    assert!(
        broken.stdout.contains("src/Consumer.jsx")
            && broken.stdout.contains("error:2:30")
            && broken
                .stdout
                .contains("Type 'string' is not assignable to type 'number'"),
        "{}",
        broken.stdout
    );

    project.write(
        "src/Consumer.jsx",
        r#"import Counter from './Counter.vue';
export const view = <Counter count={1} />;
"#,
    );
    assert_clean(project.check());
}
