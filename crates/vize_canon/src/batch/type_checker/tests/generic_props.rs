use super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

mod declaration_emit;
mod emit_only;
mod inherited_boolean;
mod inline_callback_context_props;
mod inline_callback_guarded_props;
mod inline_callback_props;
mod inline_callback_union_props;

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

#[test]
fn preserves_optional_object_spreads_after_with_defaults() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "issue-2810-generic-with-defaults-object-spread",
        &[(
            "src/Panel.vue",
            r#"<script setup lang="ts" generic="T">
interface Props {
  value?: T;
  options?: { label?: string };
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false
});
</script>

<template>
  <div v-bind="{ ...props.options }" />
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
        "optional object props must remain spreadable after unrelated defaults: {snapshot:#?}"
    );
    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn imported_define_emits_type_marks_generic_parameter_as_used() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = create_project_case(
        "issue-2808-imported-generic-emits",
        &[
            (
                "src/types.ts",
                r#"export interface ExampleValue {
  value: number;
}

export interface ExampleProps {
  firstValue?: ExampleValue | ExampleValue[];
  secondValue?: ExampleValue | ExampleValue[];
  additionalNumberValue?: number;
  disabled?: boolean;
  reversed?: boolean;
  mode?: "a" | "b" | "c";
}

export type ExampleEmits<
  T extends ExampleValue | ExampleValue[] = ExampleValue | ExampleValue[]
> = {
  change: [value: T];
};
"#,
            ),
            (
                "src/Example.vue",
                r#"<script
  setup
  lang="ts"
  generic="
    T extends ExampleValue | ExampleValue[] = ExampleValue | ExampleValue[]
  "
>
  import {
    type ExampleEmits,
    type ExampleProps,
    type ExampleValue
  } from "./types";

  const props = withDefaults(defineProps<ExampleProps>(), {
    firstValue: undefined,
    secondValue: undefined,
    additionalNumberValue: 0,
    disabled: false,
    reversed: false,
    mode: "b"
  });

  const emit = defineEmits<ExampleEmits<T>>();
  emit("change", props.firstValue as T);
  // @ts-expect-error number is outside the ExampleEmits<T> payload constraint
  emit("change", 1);
</script>

<template>
  <slot :emit :props />
</template>
"#,
            ),
        ],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "target": "ES2023",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noEmit": true,
    "noUnusedParameters": true,
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "lib": ["ES2023", "DOM", "DOM.Iterable"],
    "skipLibCheck": true
  },
  "include": ["src/**/*.vue", "src/**/*.ts"]
}"#,
    )
    .unwrap();

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.is_empty(),
        "an imported defineEmits type argument must mark T as used: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
