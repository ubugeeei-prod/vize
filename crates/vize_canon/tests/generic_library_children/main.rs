#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use level_test_support::script_block_project as project;

/// The shape vue-tsc emits for a generic SFC in a library (reka-ui's
/// `PinInputRoot`): a function whose return carries `__ctx`.
const LIBRARY: &str = include_str!(
    "../../../../tests/_fixtures/differential/typechecker/generic-library-unknown-props-11628/library.ts.txt"
);

/// Nuxt UI's PinInput forwards its own generic value to the library root.
/// vue-tsc infers the root's `Type` from the call; extracting its props
/// instead fixed `Type` to the constraint and reported TS2322.
#[test]
fn generic_library_children_infer_their_parameters_from_the_bound_props() {
    let forwarding = r#"<script setup lang="ts" generic="T extends 'text' | 'number'">
import { PinRoot } from './library'
type PinValue<Type extends 'text' | 'number'> = [Type] extends ['number'] ? number[] : string[]
const props = defineProps<{ modelValue?: PinValue<T>; type?: T }>()
</script>
<template><PinRoot :type="props.type" :model-value="props.modelValue" data-slot="root" /></template>"#;
    assert_eq!(
        project::check(&[("src/library.ts", LIBRARY), ("src/Pin.vue", forwarding)]),
        [] as [String; 0]
    );
    let wrong = r#"<script setup lang="ts">
import { PinRoot } from './library'
</script>
<template>
  <PinRoot type="number" :model-value="['a']" />
  <PinRoot :mask="'yes'" />
</template>"#;
    assert_eq!(
        project::check(&[("src/library.ts", LIBRARY), ("src/Wrong.vue", wrong)]),
        [
            "src/Wrong.vue(5,41): error TS2322: Type 'string' is not assignable to type 'number'.",
            "src/Wrong.vue(6,13): error TS2322: Type 'string' is not assignable to type 'boolean | undefined'.",
        ]
    );
}

/// An inline callback prop is contextually typed by the instantiated library
/// props (Nuxt UI's CommandPalette `:text-content="item => ..."`).
#[test]
fn inline_callbacks_of_generic_library_children_use_the_inferred_parameters() {
    let library = r#"export declare const List: <T>(
  props: import('vue').PublicProps & { options: T[]; textContent?: (option: T) => string } & (typeof globalThis extends { __VLS_PROPS_FALLBACK: infer P } ? P : {}),
  ctx?: any,
) => import('vue').VNode & {
  __ctx?: { props: { options: T[]; textContent?: (option: T) => string } };
};
"#;
    let component = r#"<script setup lang="ts">
import { List } from './library'
const items = [{ label: 'a' }]
const loose = items as any[]
</script>
<template>
  <List :options="loose" :text-content="item => String(item)" />
  <List :options="items" :text-content="item => item.label.toFixed()" />
</template>"#;
    assert_eq!(
        project::check(&[("src/library.ts", library), ("src/List.vue", component)]),
        [
            "src/List.vue(8,60): error TS2551: Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?",
        ]
    );
}

/// #11447: a listener payload comes from the instantiated `__ctx` props, as
/// vue-tsc infers it. It was `any`, so a `typeof` guard narrowed it to
/// `string` and the assignment to a generic model failed.
#[test]
fn listener_payloads_of_generic_library_children_use_the_inferred_parameters() {
    let library = r#"import type { VNode } from "vue";

export declare const Foo: <TBar>(
  props: NonNullable<Awaited<typeof setup>>["props"],
  setup?: Promise<{ props: { modelValue?: TBar; "onUpdate:modelValue"?: (value: TBar) => void } }>,
) => VNode & { __ctx?: NonNullable<Awaited<typeof setup>> };
"#;
    let probe = r#"<script setup lang="ts" generic="TBaz extends string">
import { Foo } from "./lib";

const model = defineModel<TBaz>({ required: true });
</script>

<template>
  <Foo
    :model-value="model"
    @update:model-value="
      (value) => {
        if (typeof value === 'string') model = value;
      }
    "
  />
</template>
"#;
    let variant = |from: &str, to: &str| {
        assert_eq!(probe.matches(from).count(), 1, "variant preimage");
        probe.replacen(from, to, 1)
    };
    let unconstrained = variant("TBaz extends string", "TBaz");
    let event = variant(
        "\"\n      (value) => {\n        if (typeof value === 'string') model = value;\n      }\n    \"",
        "\"if (typeof $event === 'string') model = $event\"",
    );
    let dotted = variant("<Foo\n", "<Parts.Foo\n").replacen(
        "import { Foo } from \"./lib\";",
        "import { Foo } from \"./lib\";\nconst Parts = { Foo };",
        1,
    );
    let poison = variant(
        "if (typeof value === 'string') model = value;",
        "const payload: never = value;",
    );
    let unknown = variant("(value) =>", "(value: unknown) =>");
    let plain = variant("from \"./lib\"", "from \"./plain\"");
    let plain_library = library.replacen(
        "VNode & { __ctx?: NonNullable<Awaited<typeof setup>> }",
        "VNode",
        1,
    );
    assert_eq!(
        project::check(&[
            ("src/lib.ts", library),
            ("src/plain.ts", &plain_library),
            ("src/Probe.vue", probe),
            ("src/Unconstrained.vue", &unconstrained),
            ("src/Event.vue", &event),
            ("src/Dotted.vue", &dotted),
            ("src/Poison.vue", &poison),
            ("src/Unknown.vue", &unknown),
            ("src/Plain.vue", &plain),
        ]),
        [
            "src/Plain.vue(12,40): error TS2322: Type 'string' is not assignable to type 'TBaz'.\n'string' is assignable to the constraint of type 'TBaz', but 'TBaz' could be instantiated with a different subtype of constraint 'string'.",
            "src/Poison.vue(12,15): error TS2322: Type 'TBaz' is not assignable to type 'never'.\nType 'string' is not assignable to type 'never'.",
            "src/Unknown.vue(12,40): error TS2322: Type 'string' is not assignable to type 'TBaz'.\n'string' is assignable to the constraint of type 'TBaz', but 'TBaz' could be instantiated with a different subtype of constraint 'string'.",
        ]
    );
}

/// Every listener of one library usage reads the same instantiated call.
#[test]
fn second_listeners_of_generic_library_children_keep_their_payloads() {
    let library = r#"import type { VNode } from "vue";

export declare const Foo: <TBar>(
  props: NonNullable<Awaited<typeof setup>>["props"],
  setup?: Promise<{ props: { modelValue?: TBar; "onUpdate:modelValue"?: (value: TBar) => void; onPick?: (value: TBar, index: number) => void } }>,
) => VNode & { __ctx?: NonNullable<Awaited<typeof setup>> };
"#;
    let probe = r#"<script setup lang="ts" generic="TBaz extends string">
import { Foo } from "./lib";

const model = defineModel<TBaz>({ required: true });
</script>

<template>
  <Foo
    :model-value="model"
    @update:model-value="(value) => { if (typeof value === 'string') model = value; }"
    @pick="(value, index) => { const n: number = index; if (typeof value === 'string') model = value; }"
  />
</template>
"#;
    let wrong = probe.replacen(
        "const n: number = index; if (typeof value === 'string') model = value;",
        "const n: string = index; const v: never = value;",
        1,
    );
    assert_eq!(
        project::check(&[
            ("src/lib.ts", library),
            ("src/Probe.vue", probe),
            ("src/Wrong.vue", &wrong),
        ]),
        [
            "src/Wrong.vue(11,38): error TS2322: Type 'number' is not assignable to type 'string'.",
            "src/Wrong.vue(11,63): error TS2322: Type 'TBaz' is not assignable to type 'never'.\nType 'string' is not assignable to type 'never'.",
        ]
    );
}

/// #11628: vize's checker declares vue-tsc's props fallback only where vue-tsc
/// would, so `checkUnknownProps` rejects an unknown attribute on a vue-tsc
/// generic library component exactly as vue-tsc does.
#[test]
fn checked_unknown_props_reach_generic_library_children() {
    let pin = include_str!(
        "../../../../tests/_fixtures/differential/typechecker/generic-library-unknown-props-11628/Pin.vue.txt"
    );
    let unknown = "src/Pin.vue(4,32): error TS2353: Object literal may only specify known properties, and '\"customAttr\"' does not exist in type 'VNodeProps & AllowedComponentProps & ComponentCustomProps & { type?: \"text\" | undefined; modelValue?: string[] | null | undefined; mask?: boolean | undefined; }'.";
    let strict = include_str!(
        "../../../../tests/_fixtures/differential/typechecker/generic-library-unknown-props-11628/tsconfig-strict.json.txt"
    );
    assert_eq!(
        project::check(&[
            ("tsconfig.json", strict),
            ("src/library.ts", LIBRARY),
            ("src/Pin.vue", pin)
        ]),
        [unknown]
    );
    let opted = include_str!(
        "../../../../tests/_fixtures/differential/typechecker/generic-library-unknown-props-11628/PinOptIn.vue.txt"
    );
    let opted_unknown = unknown.replace("(4,32)", "(5,32)");
    assert_eq!(
        project::check(&[("src/library.ts", LIBRARY), ("src/Pin.vue", opted)]),
        [opted_unknown.as_str()]
    );
    // vue-tsc's default leaves the attribute to fall through.
    let open = include_str!(
        "../../../../tests/_fixtures/differential/typechecker/generic-library-unknown-props-11628/tsconfig-open.json.txt"
    );
    assert_eq!(
        project::check(&[
            ("tsconfig.json", open),
            ("src/library.ts", LIBRARY),
            ("src/Pin.vue", pin)
        ]),
        [] as [String; 0]
    );
}
