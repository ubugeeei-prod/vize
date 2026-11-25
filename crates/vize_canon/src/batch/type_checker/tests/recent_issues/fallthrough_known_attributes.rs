//! The known attributes of a component whose attributes fall through to its
//! root: its props, its emits, Vue's public attributes and, when the root's
//! attributes are inferred (`fallthroughAttributes`), the root's own
//! attributes and listeners. The unknown-prop and the unknown-event checks
//! read the same surface, and the configured `checkUnknownProps` (explicit or
//! through `strictTemplates`) closes the fallthrough tail.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

mod regressions;

const INPUT_ROOT: &str = r#"<script setup lang="ts">
const model = defineModel<string>();
</script>

<template>
  <input v-model="model" />
</template>
"#;

const BUTTON_ROOT: &str = r#"<script setup lang="ts">
defineProps<{ variant?: string }>();
</script>

<template>
  <button />
</template>
"#;

const NO_INHERIT_INPUT_ROOT: &str = r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
const model = defineModel<string>();
</script>

<template>
  <input v-model="model" />
</template>
"#;

const NO_INHERIT_BUTTON_ROOT: &str = r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
defineProps<{ variant?: string }>();
</script>

<template>
  <button />
</template>
"#;

const ATTRS_INPUT_ROOT: &str = r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
const model = defineModel<string>();
</script>

<template>
  <input v-model="model" v-bind="$attrs" />
</template>
"#;

const ATTRS_BUTTON_ROOT: &str = r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
defineProps<{ variant?: string }>();
</script>

<template>
  <button v-bind="$attrs" />
</template>
"#;

const COMPONENT_ROOT: &str = r#"<script setup lang="ts">
import Leaf from "./Leaf.vue";
defineProps<{ variant?: string }>();
</script>

<template>
  <Leaf />
</template>
"#;

const LISTENER: &str = r#"<script setup lang="ts">
import Foo from "./Foo.vue";
</script>

<template>
  <Foo @keydown="(event) => event.key" />
</template>
"#;

const LISTENER_PAYLOAD: &str = r#"<script setup lang="ts">
import Foo from "./Foo.vue";
</script>

<template>
  <Foo @keydown="(event) => event.missing" />
</template>
"#;

const ATTRIBUTE: &str = r#"<script setup lang="ts">
import Bar from "./Bar.vue";
</script>

<template>
  <Bar baz="qux" />
</template>
"#;

const BOUND_ATTRIBUTE: &str = r#"<script setup lang="ts">
import Bar from "./Bar.vue";
</script>

<template>
  <Bar :baz="1" />
</template>
"#;

const ROOT_PROP: &str = r#"<script setup lang="ts">
import Bar from "./Bar.vue";
</script>

<template>
  <Bar :leaf-size="1" />
</template>
"#;

const LEAF: &str = r#"<script setup lang="ts">
defineProps<{ leafSize?: number }>();
</script>

<template>
  <span />
</template>
"#;

type Diagnostic = (vize_carton::String, Option<u32>, vize_carton::String);

fn check(name: &str, vue_options: &str, files: &[(&str, &str)]) -> Vec<Diagnostic> {
    let root = super::real_vue::create_project_case(name, files);
    let tsconfig = vize_carton::cstr!(
        r#"{{
  "compilerOptions": {{
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext",
    "lib": ["ESNext", "DOM"]
  }},
  "vueCompilerOptions": {vue_options},
  "include": ["src/**/*.vue"]
}}"#
    );
    std::fs::write(root.join("tsconfig.json"), tsconfig.as_str()).unwrap();
    let snapshot = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(&root);
    snapshot.unwrap()
}

/// The listener and attribute probes against one pair of `Foo` (input root)
/// and `Bar` (button root) bodies.
fn probes(name: &str, vue_options: &str, foo: &str, bar: &str) -> Vec<Diagnostic> {
    check(
        name,
        vue_options,
        &[
            ("src/Foo.vue", foo),
            ("src/Bar.vue", bar),
            ("src/Leaf.vue", LEAF),
            ("src/Listener.vue", LISTENER),
            ("src/ListenerPayload.vue", LISTENER_PAYLOAD),
            ("src/Attribute.vue", ATTRIBUTE),
            ("src/BoundAttribute.vue", BOUND_ATTRIBUTE),
            ("src/RootProp.vue", ROOT_PROP),
        ],
    )
}

#[test]
fn component_known_attributes_follow_the_fallthrough_root() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let mut actual = Vec::new();
    for (name, options, foo, bar) in [
        ("loose", "{}", INPUT_ROOT, BUTTON_ROOT),
        (
            "strict",
            r#"{ "strictTemplates": true }"#,
            INPUT_ROOT,
            BUTTON_ROOT,
        ),
        (
            "strict-fallthrough",
            r#"{ "strictTemplates": true, "fallthroughAttributes": true }"#,
            INPUT_ROOT,
            BUTTON_ROOT,
        ),
        (
            "strict-no-fallthrough",
            r#"{ "strictTemplates": true, "fallthroughAttributes": false }"#,
            INPUT_ROOT,
            BUTTON_ROOT,
        ),
        (
            "strict-props-unchecked",
            r#"{ "strictTemplates": true, "checkUnknownProps": false, "checkUnknownEvents": false }"#,
            INPUT_ROOT,
            BUTTON_ROOT,
        ),
        (
            "explicit-unknown-checks",
            r#"{ "checkUnknownProps": true, "checkUnknownEvents": true, "fallthroughAttributes": true }"#,
            INPUT_ROOT,
            BUTTON_ROOT,
        ),
        (
            "strict-no-inherit",
            r#"{ "strictTemplates": true, "fallthroughAttributes": true }"#,
            NO_INHERIT_INPUT_ROOT,
            NO_INHERIT_BUTTON_ROOT,
        ),
        (
            "strict-bound-attrs",
            r#"{ "strictTemplates": true, "fallthroughAttributes": true }"#,
            ATTRS_INPUT_ROOT,
            ATTRS_BUTTON_ROOT,
        ),
        (
            "strict-component-root",
            r#"{ "strictTemplates": true, "fallthroughAttributes": true }"#,
            INPUT_ROOT,
            COMPONENT_ROOT,
        ),
    ] {
        for diagnostic in probes(
            vize_carton::cstr!("fallthrough-known-attributes-{name}").as_str(),
            options,
            foo,
            bar,
        ) {
            actual.push((name, diagnostic));
        }
    }
    let unknown = |file: &str, column: u32, key: &str| -> Diagnostic {
        let message = vize_carton::cstr!(
            "6:{column}:error Object literal may only specify known properties, and '\"{key}\"' does not exist in type '{UNKNOWN_ATTRIBUTE_TYPE}'."
        );
        (file.into(), Some(2353), message)
    };
    let listener = |file: &str, known: &str| -> Diagnostic {
        let message = vize_carton::cstr!(
            "6:9:error Object literal may only specify known properties, and '\"onKeydown\"' does not exist in type '{known}'."
        );
        (file.into(), Some(2353), message)
    };
    let payload = || -> Diagnostic {
        (
            "src/ListenerPayload.vue".into(),
            Some(2339),
            "6:35:error Property 'missing' does not exist on type 'KeyboardEvent'.".into(),
        )
    };
    // `baz` is no attribute of `Bar` or of any element; `leafSize` is a prop of
    // `Bar`'s root only when that root is the `Leaf` component.
    let attributes = || {
        vec![
            unknown("src/Attribute.vue", 8, "baz"),
            unknown("src/BoundAttribute.vue", 9, "baz"),
            unknown("src/RootProp.vue", 9, "leafSize"),
        ]
    };
    let emits = r#"{ "onUpdate:modelValue"?: unknown; }"#;
    let no_root = vize_carton::cstr!("{emits} & {{}}");
    let mut expected = Vec::new();
    let mut expect = |name: &'static str, rows: Vec<Diagnostic>| {
        expected.extend(rows.into_iter().map(|row| (name, row)));
    };
    // Unchecked unknown props and events: only the root-typed payload.
    expect("loose", vec![payload()]);
    // The root's attributes are not inferred: an attribute no element knows
    // and a root listener are unknown, as in vue-tsc. The payload is still
    // typed from the root.
    let mut strict = attributes();
    strict.extend([
        listener("src/Listener.vue", emits),
        payload(),
        listener("src/ListenerPayload.vue", emits),
    ]);
    expect("strict", strict);
    // The root's listeners are known events; an attribute the root does not
    // declare is still unknown.
    let mut inferred = attributes();
    inferred.push(payload());
    expect("strict-fallthrough", inferred.clone());
    let mut explicit_false = attributes();
    explicit_false.extend([
        listener("src/Listener.vue", emits),
        listener("src/ListenerPayload.vue", emits),
    ]);
    expect("strict-no-fallthrough", explicit_false);
    expect("strict-props-unchecked", vec![payload()]);
    expect("explicit-unknown-checks", inferred.clone());
    // Nothing falls through, so the root contributes nothing.
    let mut no_inherit = attributes();
    no_inherit.extend([
        listener("src/Listener.vue", no_root.as_str()),
        listener("src/ListenerPayload.vue", no_root.as_str()),
    ]);
    expect("strict-no-inherit", no_inherit);
    // `v-bind="$attrs"` names the root explicitly.
    expect("strict-bound-attrs", inferred);
    // A component root contributes its own props.
    expect(
        "strict-component-root",
        vec![
            unknown("src/Attribute.vue", 8, "baz"),
            unknown("src/BoundAttribute.vue", 9, "baz"),
            payload(),
        ],
    );
    // Compare each case's diagnostics as a set, in case order.
    let order = |name: &str| {
        CASES
            .iter()
            .position(|case| *case == name)
            .expect("known case")
    };
    for rows in [&mut actual, &mut expected] {
        rows.sort_by(|left, right| (order(left.0), &left.1).cmp(&(order(right.0), &right.1)));
    }
    assert_eq!(actual, expected);
}

const OPTIONS_PROPS: &str = r#"<script lang="ts">
import { defineComponent } from "vue";
export default defineComponent({
  props: { level: { type: Number, required: true } },
});
</script>

<template>
  <i />
</template>
"#;

const OPTIONS_USAGE: &str = r#"<script setup lang="ts">
import Bar from "./Bar.vue";
import Opts from "./Opts.vue";
</script>

<template>
  <Opts :level="1" />
  <Bar baz="qux" />
</template>
"#;

/// This batch checker leaves Options API projection off, as
/// `typeChecker.optionsApi: false` does. The props of an Options API
/// component are then not extracted, so they are unknown rather than empty:
/// its declared `level` is no unknown prop. A component whose props are known
/// still reports an attribute it does not declare.
#[test]
fn unextracted_options_api_props_keep_the_component_contract_open() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    assert_eq!(
        check(
            "fallthrough-known-attributes-unextracted-options",
            r#"{ "strictTemplates": true }"#,
            &[
                ("src/Opts.vue", OPTIONS_PROPS),
                ("src/Bar.vue", BUTTON_ROOT),
                ("src/Usage.vue", OPTIONS_USAGE),
            ],
        ),
        vec![(
            "src/Usage.vue".into(),
            Some(2353),
            vize_carton::cstr!(
                "8:8:error Object literal may only specify known properties, and '\"baz\"' does not exist in type '{UNKNOWN_ATTRIBUTE_TYPE}'."
            ),
        )],
    );
}

macro_rules! reported {
    ($file:literal) => {
        include_str!(concat!(
            "../../../../../../../tests/_fixtures/differential/typechecker/component-fallthrough-attributes-13685/",
            $file
        ))
    };
}

/// The two reported projects, byte for byte: a root listener under
/// `fallthroughAttributes` is a known event typed by the root, and an
/// attribute neither the component nor its root declares is unknown.
#[test]
fn reported_projects_type_attributes_from_the_fallthrough_root() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    for (name, tsconfig, foo, app, expected) in [
        (
            "fallthrough-listener",
            reported!("tsconfig-fallthrough.json.txt"),
            reported!("InputRoot.vue.txt"),
            reported!("Listener.vue.txt"),
            Vec::new(),
        ),
        (
            "unknown-attribute",
            reported!("tsconfig.json.txt"),
            reported!("ButtonRoot.vue.txt"),
            reported!("Attribute.vue.txt"),
            vec![(
                "App.vue".into(),
                Some(2353),
                vize_carton::cstr!(
                    "6:8:error Object literal may only specify known properties, and '\"baz\"' does not exist in type '{UNKNOWN_ATTRIBUTE_TYPE}'."
                ),
            )],
        ),
    ] {
        let root = super::real_vue::create_project_case(
            vize_carton::cstr!("component-fallthrough-attributes-{name}").as_str(),
            &[
                ("tsconfig.json", tsconfig),
                ("Foo.vue", foo),
                ("App.vue", app),
            ],
        );
        let snapshot = snapshot_project_diagnostics(&root);
        let _ = std::fs::remove_dir_all(&root);
        let expected: Vec<Diagnostic> = expected;
        assert_eq!(snapshot.unwrap(), expected, "{name}");
    }
}

const CASES: [&str; 9] = [
    "loose",
    "strict",
    "strict-fallthrough",
    "strict-no-fallthrough",
    "strict-props-unchecked",
    "explicit-unknown-checks",
    "strict-no-inherit",
    "strict-bound-attrs",
    "strict-component-root",
];

/// The type the unknown-prop check prints for `Bar`: its props, Vue's public
/// attributes and every attribute a native element declares.
const UNKNOWN_ATTRIBUTE_TYPE: &str = "$ComponentCheckProps<Props, $PublicComponentAttrs & { abbr?: unknown; about?: unknown; \"accent-height\"?: unknown; accentHeight?: unknown; accept?: unknown; acceptcharset?: unknown; accesskey?: unknown; accumulate?: unknown; action?: unknown; additive?: unknown; align?: unknown; ... 671 more ...; zoomAndPan?: unknown...";
