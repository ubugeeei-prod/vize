//! A classic `<script>` block and a `<script setup>` block share declarations
//! through Vue's module/setup visibility bridge, not by being merged into one
//! synthetic scope.
//!
//! Every expectation below was recorded from a `vue-tsc` 3.3.4 / `vue`
//! 3.6.0-beta.10 run over byte-identical fixtures (issue #4151).

#[path = "support/script_block_project.rs"]
mod project;

/// The Misskey `notifications.notification-config.vue` shape: the classic block
/// exports a type whose body depends on a classic-block value, and the setup
/// block consumes that type from `defineProps`/`defineEmits`.
const EXPORTED_TYPE: &str = r#"<script lang="ts">
const notificationConfigTypes = ['all', 'list', 'never'] as const;

export type NotificationConfig = {
  type: Exclude<typeof notificationConfigTypes[number], 'list'>;
} | {
  type: 'list';
  userListId: string;
};
</script>

<script lang="ts" setup>
const props = defineProps<{
  value: NotificationConfig;
  configurableTypes?: NotificationConfig['type'][];
}>();

const emit = defineEmits<{
  (ev: 'update', result: NotificationConfig): void;
}>();

const labels: Record<typeof notificationConfigTypes[number], string> = {
  all: 'all',
  list: 'list',
  never: 'never',
};

function save() {
  emit('update', props.value);
}
</script>

<template>
  <div>{{ labels[props.value.type] }}<button @click="save">save</button></div>
</template>
"#;

const EXPORTED_INTERFACE: &str = r#"<script lang="ts">
const COLORS = ['blue', 'red'] as const;

export interface BadgeOptions {
  color: typeof COLORS[number];
}
</script>

<script lang="ts" setup>
const props = defineProps<{ options: BadgeOptions }>();
const fallback: BadgeOptions = { color: COLORS[0] };
</script>

<template>
  <div>{{ props.options.color }}{{ fallback.color }}</div>
</template>
"#;

const EXPORTED_VALUE: &str = r#"<script lang="ts">
export const PAGE_SIZE = 20;

export enum DiffMode {
  Unified = 'unified',
  Split = 'split',
}

export class Cursor {
  line = 0;
}
</script>

<script lang="ts" setup>
const mode: DiffMode = DiffMode.Unified;
const cursor: Cursor = new Cursor();
const size = PAGE_SIZE;
</script>

<template>
  <div>{{ mode }}{{ cursor.line }}{{ size }}</div>
</template>
"#;

const AMBIENT_DECLARATION: &str = r#"<script lang="ts">
declare const __BUILD_ID__: string;

const stages = ['alpha', 'beta'] as const;

export declare type Stage = typeof stages[number];
</script>

<script lang="ts" setup>
const props = defineProps<{ stage: Stage }>();
const build = __BUILD_ID__;
</script>

<template>
  <div>{{ props.stage }}{{ build }}</div>
</template>
"#;

const DEFAULT_EXPORT: &str = r#"<script lang="ts">
import { defineComponent } from 'vue';

const levels = ['info', 'warn'] as const;

export type Level = typeof levels[number];

export default defineComponent({
  inheritAttrs: false,
});
</script>

<script lang="ts" setup>
const props = defineProps<{ level: Level }>();
</script>

<template>
  <div>{{ props.level }}</div>
</template>
"#;

const OPTIONS_API: &str = r#"<script lang="ts">
import { defineComponent } from 'vue';

const themes = ['light', 'dark'] as const;

export type Theme = typeof themes[number];

export default defineComponent({
  name: 'OptionsApiHost',
  inheritAttrs: false,
});
</script>

<script lang="ts" setup>
const props = defineProps<{ theme: Theme }>();
const isDark = props.theme === 'dark';
</script>

<template>
  <div>{{ isDark }}</div>
</template>
"#;

const SETUP_MACROS: &str = r#"<script lang="ts">
const sizes = ['sm', 'lg'] as const;

export type Size = typeof sizes[number];
export interface HostSlots {
  default(props: { size: Size }): unknown;
}
</script>

<script lang="ts" setup>
const props = withDefaults(defineProps<{ size?: Size }>(), { size: 'sm' });
const emit = defineEmits<{ (ev: 'resize', size: Size): void }>();
const model = defineModel<Size>({ default: 'sm' });
defineSlots<HostSlots>();
defineExpose({ size: props.size });

function bump() {
  emit('resize', props.size);
  model.value = props.size;
}
</script>

<template>
  <div @click="bump">{{ props.size }}{{ model }}</div>
</template>
"#;

const GENERIC_EXPORTED_TYPE: &str = r#"<script lang="ts">
const keys = ['a', 'b'] as const;

export type Boxed<T> = { key: typeof keys[number]; value: T };
</script>

<script lang="ts" setup>
const props = defineProps<{ boxed: Boxed<number> }>();
</script>

<template>
  <div>{{ props.boxed.value }}</div>
</template>
"#;

const CONSUMER_HEAD: &str = r#"import type { NotificationConfig } from './ExportedType.vue';
import type { BadgeOptions } from './ExportedInterface.vue';
import { PAGE_SIZE, DiffMode, Cursor } from './ExportedValue.vue';
import type { Stage } from './AmbientDeclaration.vue';
import type { Level } from './DefaultExport.vue';
import type { Size } from './SetupMacros.vue';
import type { Theme } from './OptionsApi.vue';
import type { Boxed } from './GenericExportedType.vue';
"#;

/// Three deliberately wrong assignments, each on a line the expectations pin.
const CONSUMER_TAIL_INVALID: &str = r#"
const okConfig: NotificationConfig = { type: 'all' };
const badConfig: NotificationConfig = { type: 'list' };
const okBadge: BadgeOptions = { color: 'red' };
const badBadge: BadgeOptions = { color: 'green' };
const okStage: Stage = 'alpha';
const badStage: Stage = 'gamma';
const okLevel: Level = 'info';
const okSize: Size = 'sm';
const okTheme: Theme = 'dark';
const okBoxed: Boxed<number> = { key: 'a', value: 1 };
const okCursor = new Cursor();
void okConfig; void badConfig; void okBadge; void badBadge; void okStage;
void badStage; void okLevel; void okSize; void okTheme; void okBoxed;
void okCursor; void PAGE_SIZE; void DiffMode;
"#;

/// The same file with each wrong assignment repaired in place.
const CONSUMER_TAIL_REPAIRED: &str = r#"
const okConfig: NotificationConfig = { type: 'all' };
const badConfig: NotificationConfig = { type: 'list', userListId: 'x' };
const okBadge: BadgeOptions = { color: 'red' };
const badBadge: BadgeOptions = { color: 'blue' };
const okStage: Stage = 'alpha';
const badStage: Stage = 'beta';
const okLevel: Level = 'info';
const okSize: Size = 'sm';
const okTheme: Theme = 'dark';
const okBoxed: Boxed<number> = { key: 'a', value: 1 };
const okCursor = new Cursor();
void okConfig; void badConfig; void okBadge; void badBadge; void okStage;
void badStage; void okLevel; void okSize; void okTheme; void okBoxed;
void okCursor; void PAGE_SIZE; void DiffMode;
"#;

fn components() -> Vec<(&'static str, &'static str)> {
    vec![
        ("src/ExportedType.vue", EXPORTED_TYPE),
        ("src/ExportedInterface.vue", EXPORTED_INTERFACE),
        ("src/ExportedValue.vue", EXPORTED_VALUE),
        ("src/AmbientDeclaration.vue", AMBIENT_DECLARATION),
        ("src/DefaultExport.vue", DEFAULT_EXPORT),
        ("src/OptionsApi.vue", OPTIONS_API),
        ("src/SetupMacros.vue", SETUP_MACROS),
        ("src/GenericExportedType.vue", GENERIC_EXPORTED_TYPE),
    ]
}

#[test]
fn cross_block_type_sharing_reports_exactly_what_vue_tsc_reports() {
    let consumer = format!("{CONSUMER_HEAD}{CONSUMER_TAIL_INVALID}");
    let mut files = components();
    files.push(("src/consumer.ts", consumer.as_str()));

    // Only the three authored assignment errors survive: sharing a
    // classic-block declaration with the setup block is not a duplicate.
    assert_eq!(
        project::check(&files),
        [
            "src/consumer.ts(11,7): error TS2322: Type '{ type: \"list\"; }' is not assignable to type 'NotificationConfig'.\nProperty 'userListId' is missing in type '{ type: \"list\"; }' but required in type '{ type: \"list\"; userListId: string; }'.",
            "src/consumer.ts(13,34): error TS2322: Type '\"green\"' is not assignable to type '\"blue\" | \"red\"'.",
            "src/consumer.ts(15,7): error TS2322: Type '\"gamma\"' is not assignable to type '\"alpha\" | \"beta\"'.",
        ]
    );
}

#[test]
fn repairing_the_consumer_leaves_no_diagnostics() {
    let consumer = format!("{CONSUMER_HEAD}{CONSUMER_TAIL_REPAIRED}");
    let mut files = components();
    files.push(("src/consumer.ts", consumer.as_str()));

    assert_eq!(project::check(&files), [] as [String; 0]);
}
