//! Isolating the two script blocks must not stop vize reporting the collisions
//! that are genuinely illegal, nor start reporting the shadowing Vue allows.
//!
//! Every expectation below was recorded from a `vue-tsc` 3.3.4 / `vue`
//! 3.6.0-beta.10 run over byte-identical fixtures (issue #4151).

#[path = "support/script_block_project.rs"]
mod project;

/// Two declarations of one name inside the classic block.
const DUPLICATE_IN_CLASSIC: &str = r#"<script lang="ts">
const kinds = ['a', 'b'] as const;

export type Kind = typeof kinds[number];
export type Kind = string;
</script>

<script lang="ts" setup>
const props = defineProps<{ kind: Kind }>();
</script>

<template>
  <div>{{ props.kind }}</div>
</template>
"#;

/// The same collision repaired by renaming the second declaration.
const REPAIRED_CLASSIC: &str = r#"<script lang="ts">
const kinds = ['a', 'b'] as const;

export type Kind = typeof kinds[number];
export type KindName = string;
</script>

<script lang="ts" setup>
const props = defineProps<{ kind: Kind; name: KindName }>();
</script>

<template>
  <div>{{ props.kind }}{{ props.name }}</div>
</template>
"#;

/// Two declarations of one name inside the setup block.
const DUPLICATE_IN_SETUP: &str = r#"<script lang="ts">
const kinds = ['a', 'b'] as const;
</script>

<script lang="ts" setup>
type Local = typeof kinds[number];
type Local = string;

const value: Local = 'a';
</script>

<template>
  <div>{{ value }}</div>
</template>
"#;

/// Both blocks *export* one name, so both land at module scope.
const DUPLICATE_EXPORT_ACROSS_BLOCKS: &str = r#"<script lang="ts">
const kinds = ['a', 'b'] as const;

export type Kind = typeof kinds[number];
</script>

<script lang="ts" setup>
export type Kind = string;

const value: Kind = 'a';
</script>

<template>
  <div>{{ value }}</div>
</template>
"#;

/// The classic block cannot see setup-scope values.
const INVALID_CROSS_BLOCK_REF: &str = r#"<script lang="ts">
export type FromSetup = typeof setupOnlyValue;
</script>

<script lang="ts" setup>
const setupOnlyValue = 1;
const used = setupOnlyValue;
</script>

<template>
  <div>{{ used }}</div>
</template>
"#;

/// A setup-*local* alias shadows the classic export rather than colliding.
const SHADOWING_TYPE_ALIAS: &str = r#"<script lang="ts">
const kinds = ['a', 'b'] as const;

export type Kind = typeof kinds[number];
</script>

<script lang="ts" setup>
type Kind = string;

const value: Kind = 'a';
</script>

<template>
  <div>{{ value }}</div>
</template>
"#;

/// The same shadowing through an `interface`, over a still-used classic value.
const SHADOWING_INTERFACE: &str = r#"<script lang="ts">
const sizes = ['sm', 'lg'] as const;

export type Size = typeof sizes[number];
export const DEFAULT_SIZE: Size = 'sm';
</script>

<script lang="ts" setup>
interface Size {
  width: number;
}

const box: Size = { width: 1 };
</script>

<template>
  <div>{{ box.width }}{{ DEFAULT_SIZE }}</div>
</template>
"#;

/// The shadowed classic declaration is a class, whose name lives in both the
/// type and the value space.
const SHADOWING_CLASS: &str = r#"<script lang="ts">
export class Shape {
  size = 1;
}
</script>

<script lang="ts" setup>
type Shape = { size: number };

const value: Shape = { size: 2 };
</script>

<template>
  <div>{{ value.size }}</div>
</template>
"#;

/// The classic declaration of the same name is a `const`, which only occupies
/// the value space. The setup `type` therefore shadows nothing and still has to
/// reach module scope, where the generated `Emits`/`Slots` aliases reference it.
const CLASSIC_VALUE_AND_SETUP_TYPE: &str = r#"<script lang="ts">
export const Item = 1;
</script>

<script lang="ts" setup>
type Item = { id: number };

defineSlots<{ default(props: { item: Item }): unknown }>();
const emit = defineEmits<{ (ev: 'pick', item: Item): void }>();

const current: Item = { id: Item };

function pick() {
  emit('pick', current);
}
</script>

<template>
  <div @click="pick">{{ current.id }}</div>
</template>
"#;

/// A parent handing that child a wrongly typed `@pick` handler: the setup
/// `type Item` has to reach module scope for the generated `Emits` alias to
/// keep checking the component's contract.
const CLASSIC_VALUE_CONSUMER: &str = r#"<script setup lang="ts">
import Child from './ClassicValueAndSetupType.vue';

function wrong(item: string) {
  void item;
}
</script>

<template>
  <Child @pick="wrong" />
</template>
"#;

/// One local name imported by both blocks is a real duplicate binding.
const SHARED_IMPORT_NAME: &str = r#"<script lang="ts">
import { ref } from 'vue';

export const seed = ref(1);
</script>

<script lang="ts" setup>
import { ref } from 'vue';

const local = ref(seed.value);
</script>

<template>
  <div>{{ local }}</div>
</template>
"#;

/// A wrong value assigned to a shared classic type: one error, no duplicate.
const BAD_ASSIGNMENT: &str = r#"<script lang="ts">
const kinds = ['a', 'b'] as const;

export type Kind = typeof kinds[number];
</script>

<script lang="ts" setup>
const value: Kind = 'c';
</script>

<template>
  <div>{{ value }}</div>
</template>
"#;

#[test]
fn genuine_collisions_and_legal_shadowing_match_vue_tsc() {
    assert_eq!(
        project::check(&[
            ("src/DuplicateInClassic.vue", DUPLICATE_IN_CLASSIC),
            ("src/DuplicateInSetup.vue", DUPLICATE_IN_SETUP),
            (
                "src/DuplicateExportAcrossBlocks.vue",
                DUPLICATE_EXPORT_ACROSS_BLOCKS
            ),
            ("src/InvalidCrossBlockRef.vue", INVALID_CROSS_BLOCK_REF),
            ("src/ShadowingTypeAlias.vue", SHADOWING_TYPE_ALIAS),
            ("src/ShadowingInterface.vue", SHADOWING_INTERFACE),
            ("src/ShadowingClass.vue", SHADOWING_CLASS),
            (
                "src/ClassicValueAndSetupType.vue",
                CLASSIC_VALUE_AND_SETUP_TYPE
            ),
            ("src/SharedImportName.vue", SHARED_IMPORT_NAME),
            ("src/BadAssignment.vue", BAD_ASSIGNMENT),
        ]),
        [
            "src/BadAssignment.vue(8,7): error TS2322: Type '\"c\"' is not assignable to type '\"a\" | \"b\"'.",
            "src/DuplicateExportAcrossBlocks.vue(4,13): error TS2300: Duplicate identifier 'Kind'.",
            "src/DuplicateExportAcrossBlocks.vue(8,13): error TS2300: Duplicate identifier 'Kind'.",
            "src/DuplicateInClassic.vue(4,13): error TS2300: Duplicate identifier 'Kind'.",
            "src/DuplicateInClassic.vue(5,13): error TS2300: Duplicate identifier 'Kind'.",
            "src/DuplicateInSetup.vue(6,6): error TS2300: Duplicate identifier 'Local'.",
            "src/DuplicateInSetup.vue(7,6): error TS2300: Duplicate identifier 'Local'.",
            "src/InvalidCrossBlockRef.vue(2,32): error TS2304: Cannot find name 'setupOnlyValue'.",
            "src/SharedImportName.vue(2,10): error TS2300: Duplicate identifier 'ref'.",
            "src/SharedImportName.vue(8,10): error TS2300: Duplicate identifier 'ref'.",
        ]
    );
}

#[test]
fn a_setup_type_over_a_classic_value_keeps_checking_the_component_contract() {
    assert_eq!(
        project::check(&[
            (
                "src/ClassicValueAndSetupType.vue",
                CLASSIC_VALUE_AND_SETUP_TYPE
            ),
            ("src/ClassicValueConsumer.vue", CLASSIC_VALUE_CONSUMER),
        ]),
        [
            "src/ClassicValueConsumer.vue(10,11): error TS2322: Type '(item: string) => void' is not assignable to type '(item: Item) => any'.\nTypes of parameters 'item' and 'item' are incompatible.\nType 'Item' is not assignable to type 'string'.",
        ]
    );
}

#[test]
fn renaming_the_second_classic_declaration_repairs_the_duplicate() {
    assert_eq!(
        project::check(&[("src/DuplicateInClassic.vue", REPAIRED_CLASSIC)]),
        [] as [String; 0]
    );
}
