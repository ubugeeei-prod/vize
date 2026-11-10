//! The TSX instance contract's authored-key set (#11654).
//!
//! `$ComponentInstanceFor` keeps the authored JSX keys that the component
//! accepts: its props, their kebab-case spellings, declared emit listeners,
//! fallthrough attributes, DOM listeners and Vue's public input props. Listing
//! that set directly (instead of taking `keyof` of the full input-props
//! intersection, which distributes over every DOM listener key) must keep
//! every exact TSX and template input diagnostic, and a function returning
//! several SFCs must keep its prop checks.
use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::String;

type Row = (String, Option<u32>, String);

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ someValue: string; count?: number }>()
defineEmits<{ select: [value: string] }>()
</script>

<template><div>{{ someValue }}</div></template>
"#;

const ANY_PROPS: &str = r#"<script setup lang="ts">
defineProps<any>()
</script>

<template><div /></template>
"#;

const ANY_EMITS: &str = r#"<script setup lang="ts">
defineProps<{ label: string }>()
defineEmits<any>()
</script>

<template><div>{{ label }}</div></template>
"#;

const PROBE: &str = r#"import Child from "./Child.vue";
import AnyProps from "./AnyProps.vue";
import AnyEmits from "./AnyEmits.vue";

export const valid = <Child someValue="a" count={1} />;
export const kebab = <Child some-value="a" />;
export const owned = <Child someValue="a" class="c" style="color: red" data-x="1" aria-label="l" key="k" />;
export const declaredListener = <Child someValue="a" onSelect={(value: string) => value} />;
export const domListener = <Child someValue="a" onClick={(event: MouseEvent) => event} />;
export const extra = <Child someValue="a" unknownProp="x" />;
export const wrongType = <Child someValue={1} />;
export const missing = <Child />;
export const wrongListener = <Child someValue="a" onSelect={(value: number) => value} />;
export const anyProps = <AnyProps anything={1} />;
export const anyEmits = <AnyEmits label="a" anything={1} />;
export const anyEmitsWrongType = <AnyEmits label={1} />;
"#;

const SAME_A: &str = r#"<script setup lang="ts">
defineProps<{ block: { id: string; type: string }; h: number }>()
</script>

<template><p>{{ block.id }} {{ h }}</p></template>
"#;

const SAME_B: &str = r#"<script setup lang="ts">
defineProps<{ block: { id: string; type: string }; h: number }>()
</script>

<template><span>{{ block.type }} {{ h }}</span></template>
"#;

const UNION_HOST: &str = r#"<script setup lang="ts">
import SameA from './SameA.vue'
import SameB from './SameB.vue'
import Child from './Child.vue'

const props = defineProps<{ block: { id: string; type: string }; h: number }>()

function getComponent(type: string) {
  switch (type) {
    case 'a': return SameA
    case 'b': return SameB
    case 'child': return Child
    default: return null
  }
}

const pair = props.h > 0 ? SameA : SameB
const listed = [SameA, SameB]
void listed
</script>

<template>
  <component :is="getComponent(block.type)" :block="block" :h="h" />
  <component :is="pair" :block="block" :h="'tall'" />
  <component :is="pair" :h="h" />
</template>
"#;

fn write_tsconfig(project_root: &std::path::Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
}

fn diagnostics(name: &str, files: &[(&str, &str)]) -> Option<Vec<Row>> {
    resolve_test_tsgo_binary()?;
    let project_root = super::real_vue::create_project_case(name, files);
    write_tsconfig(&project_root);
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    snapshot
}

fn row(file: &str, code: u32, text: &str) -> Row {
    (file.into(), Some(code), text.into())
}

#[test]
fn tsx_input_keys_keep_exact_attribute_checks() {
    let Some(snapshot) = diagnostics(
        "instance-input-keys-tsx",
        &[
            ("src/Child.vue", CHILD),
            ("src/AnyProps.vue", ANY_PROPS),
            ("src/AnyEmits.vue", ANY_EMITS),
            ("src/Probe.tsx", PROBE),
        ],
    ) else {
        return;
    };
    assert_eq!(
        snapshot,
        vec![
            row(
                "src/Probe.tsx",
                2322,
                "10:43:error Type '{ someValue: string; unknownProp: string; }' is not assignable to type 'IntrinsicAttributes & Pick<$AuthoredKeyWitness<{ someValue: unknown; }>, \"someValue\"> & Partial<Props> & Partial<$KebabProps<Props>> & ... 7 more ... & ComponentCustomProps'.\nProperty 'unknownProp' does not exist on type 'IntrinsicAttributes & Pick<$AuthoredKeyWitness<{ someValue: unknown; }>, \"someValue\"> & Partial<Props> & Partial<$KebabProps<Props>> & ... 7 more ... & ComponentCustomProps'.",
            ),
            row(
                "src/Probe.tsx",
                2322,
                "11:33:error Type 'number' is not assignable to type 'string'.",
            ),
            row(
                "src/Probe.tsx",
                2322,
                "12:25:error Type '{}' is not assignable to type 'IntrinsicAttributes & Props & $EmitPropsFromOptions<{ select: (value: string) => any; }>'.\nProperty 'someValue' is missing in type '{}' but required in type 'Props'.",
            ),
            row(
                "src/Probe.tsx",
                2322,
                "13:51:error Type '(value: number) => number' is not assignable to type '(value: string) => any'.\nTypes of parameters 'value' and 'value' are incompatible.\nType 'string' is not assignable to type 'number'.",
            ),
            row(
                "src/Probe.tsx",
                2322,
                "15:45:error Type '{ label: string; anything: number; }' is not assignable to type 'IntrinsicAttributes & Pick<$AuthoredKeyWitness<{ label: unknown; }>, \"label\"> & Partial<Props> & Partial<$KebabProps<Props>> & ... 7 more ... & ComponentCustomProps'.\nProperty 'anything' does not exist on type 'IntrinsicAttributes & Pick<$AuthoredKeyWitness<{ label: unknown; }>, \"label\"> & Partial<Props> & Partial<$KebabProps<Props>> & ... 7 more ... & ComponentCustomProps'.",
            ),
            row(
                "src/Probe.tsx",
                2322,
                "16:44:error Type 'number' is not assignable to type 'string'.",
            ),
        ]
    );
}

#[test]
fn component_unions_keep_prop_checks() {
    let Some(snapshot) = diagnostics(
        "instance-input-keys-union",
        &[
            ("src/Child.vue", CHILD),
            ("src/SameA.vue", SAME_A),
            ("src/SameB.vue", SAME_B),
            ("src/UnionHost.vue", UNION_HOST),
        ],
    ) else {
        return;
    };
    assert_eq!(
        snapshot,
        vec![
            row(
                "src/UnionHost.vue",
                2322,
                "24:41:error Type 'string' is not assignable to type 'number'.",
            ),
            row(
                "src/UnionHost.vue",
                2345,
                "25:4:error Argument of type '{ h: number; }' is not assignable to parameter of type '$ComponentCheckProps<Props, $PublicComponentAttrs & Record<string, unknown>>'.\nProperty 'block' is missing in type '{ h: number; }' but required in type '{ readonly block: { id: string; type: string; }; readonly h: number; }'.",
            ),
        ]
    );
}
