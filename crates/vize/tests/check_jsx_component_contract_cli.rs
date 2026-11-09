#![cfg(test)]
use crate::support::corsa_requirement;
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
        "data-id",
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
export const view = <Counter count={1} is-opened data-id={1} onClick={(event: string) => event.toUpperCase()} />;
"#,
        "not assignable",
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
    data-role="counter"
    aria-label="Counter"
    onClick={(event) => event.preventDefault()}
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

#[test]
fn check_sfc_constructor_assignability_reads_the_public_instance() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "fixtures/constructor_assignability/check.expected.json"
    ))
    .unwrap();
    let project = JsxCheckProject::new("ctor", corsa_path, false);
    project.write(
        "src/Child.vue",
        r#"<script setup lang="ts">
defineProps<{ label: string }>();
</script>
<template><span>{{ label }}</span></template>
"#,
    );
    let parent = |annotation: &str| {
        format!(
            r#"<script setup lang="ts">
import {{ useTemplateRef }} from 'vue';
import Child from './Child.vue';
const child = useTemplateRef<InstanceType<typeof Child>>('child');
const read: {annotation} | null = child.value;
void read;
</script>
<template><Child ref="child" label="x" /></template>
"#
        )
    };
    let clean = r#"import Child from './Child.vue';
export const ctor: new () => { $props: { label: string } } = Child;
export const read = (child: InstanceType<typeof Child>): { $props: { label: string } } => child;
export const label = (child: InstanceType<typeof Child>) => child.$props.label satisfies string;
"#;
    // A wrong or missing public prop still fails through every route; the
    // no-argument constructor sees the instance's own `$props`, as Vue's
    // `DefineComponent` does, so the clean consumer reports nothing.
    let cases = [
        (
            "ctor wrong prop",
            "import Child from './Child.vue';\nexport const ctor: new () => { $props: { label: number } } = Child;\n",
            "{ $props: { label: string } }",
        ),
        (
            "ctor missing prop",
            "import Child from './Child.vue';\nexport const ctor: new () => { $props: { label: string; missing: string } } = Child;\n",
            "{ $props: { label: string } }",
        ),
        (
            "InstanceType wrong prop",
            "import Child from './Child.vue';\nexport const read = (child: InstanceType<typeof Child>): { $props: { label: number } } => child;\n",
            "{ $props: { label: string } }",
        ),
        (
            "InstanceType member",
            "import Child from './Child.vue';\nexport const read = (child: InstanceType<typeof Child>) => child.$props.label satisfies number;\n",
            "{ $props: { label: string } }",
        ),
        (
            "template ref wrong prop",
            clean,
            "{ $props: { label: number } }",
        ),
        (
            "template ref missing prop",
            clean,
            "{ $props: { label: string; missing: string } }",
        ),
        ("clean", clean, "{ $props: { label: string } }"),
    ];
    let mut actual = serde_json::Map::new();
    for (case, consumer, annotation) in cases {
        project.write("src/Consumer.tsx", consumer);
        project.write("src/Parent.vue", &parent(annotation));
        let output = project.check();
        let report: serde_json::Value =
            serde_json::from_str(&output.stdout).unwrap_or_else(|error| {
                panic!(
                    "{case}: invalid JSON ({error}):\n{}\n{}",
                    output.stdout, output.stderr
                )
            });
        actual.insert(
            case.into(),
            serde_json::json!({ "success": output.success, "files": report["files"] }),
        );
    }
    let actual = serde_json::Value::Object(actual);
    if actual != expected {
        // The whole packet, so it can be reviewed and pinned at once.
        eprintln!("{}", serde_json::to_string_pretty(&actual).unwrap());
    }
    assert_eq!(actual, expected);
}
