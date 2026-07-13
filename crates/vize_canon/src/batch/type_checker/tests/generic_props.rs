use super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn batch_type_checker_exposes_generic_props_inherited_from_pick() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-setup-props-inherited-pick-template",
        &[(
            "src/Foo.vue",
            r#"<script lang="ts">
interface BaseProps {
  required?: boolean;
}

export interface FooProps<T = string> extends Pick<BaseProps, "required"> {
  value?: T;
}
</script>

<script setup lang="ts" generic="T = string">
defineProps<FooProps<T>>();
</script>

<template>
  <input :required />
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/Foo.vue" && *code == Some(2304) && message.contains("required"))
        }),
        "inherited type-based props should be exposed to generic SFC templates, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn batch_type_checker_infers_generic_sfc_props_through_extracted_component_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-sfc-extracted-component-props",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts" generic="TValue = undefined">
defineProps<{ value?: TValue }>();
</script>
"#,
            ),
            (
                "src/usage.ts",
                r#"import Child from "./Child.vue";

type ComponentProps<T> = T extends new (...args: any) => { $props: infer P }
  ? NonNullable<P>
  : {};

declare function mount<T>(
  component: T,
  options?: { props?: ComponentProps<T> }
): void;

mount(Child, {
  props: {
    value: "one"
  }
});
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/usage.ts"
                && *code == Some(2322)
                && message.contains("Type 'string' is not assignable to type 'undefined'"))
        }),
        "generic SFC props should infer through extracted component props, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn preserves_generic_parameter_when_default_differs_from_constraint() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "issue-2811-generic-default-scope",
        &[(
            "src/Foo.vue",
            r#"<script setup lang="ts" generic="T extends string | number = string">
import { ref } from "vue";

interface Props {
  value?: T;
}

const props = defineProps<Props>();
const state = ref(props.value);

function setValue(value: T) {
  state.value = value;
}
</script>

<template>
  <slot :set-value :state />
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "generic defaults must not replace T inside the SFC declaration: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn preserves_optional_union_props_after_with_defaults() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "issue-2809-with-defaults-union",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
import { ref } from "vue";

interface FooProps {
  kind?: "one";
  value?: string;
}

interface BarProps {
  kind?: "many";
  value?: string[];
}

type ExampleProps = FooProps | BarProps;

const props = withDefaults(defineProps<ExampleProps>(), {
  kind: "one",
});

const value = ref(props.value);

function clear() {
  value.value = undefined;
}
</script>

<template>
  <button type="button" @click="clear">Clear</button>
</template>
"#,
        )],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "a defaulted discriminant must not require other union props: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
