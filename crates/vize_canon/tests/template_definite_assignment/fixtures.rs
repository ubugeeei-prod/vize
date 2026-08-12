pub(crate) const SUPPORT_TS: &str = r#"export class Paginator {
  constructor(public readonly key: string) {}
  reload(): void {}
}
export declare function fetchTags(): Promise<string[]>;
export declare const flag: boolean;
"#;

pub(crate) const WATCH_IMMEDIATE_SFC: &str = r#"<script setup lang="ts">
import { ref, watch } from 'vue'
import { Paginator } from './support'
const tab = ref('list')
let paginator: Paginator
watch(tab, (newTab) => {
  paginator = new Paginator(newTab)
}, { immediate: true })
</script>

<template>
  <div>{{ paginator.key }}</div>
</template>
"#;

pub(crate) const CONDITIONAL_AWAIT_SFC: &str = r#"<script setup lang="ts">
import { fetchTags, flag } from './support'
let followedTags: string[]
if (flag) {
  followedTags = await fetchTags()
}
</script>

<template>
  <div>{{ followedTags.length }}</div>
</template>
"#;

pub(crate) const CONDITIONAL_ASSIGN_SFC: &str = r#"<script setup lang="ts">
import { Paginator, flag } from './support'
let paginator: Paginator
if (flag) {
  paginator = new Paginator('a')
}
</script>

<template>
  <div>{{ paginator.key }}</div>
</template>
"#;

pub(crate) const ASYNC_CALLBACK_SFC: &str = r#"<script setup lang="ts">
import { onMounted } from 'vue'
import { fetchTags } from './support'
let followedTags: string[]
onMounted(async () => {
  followedTags = await fetchTags()
})
</script>

<template>
  <div>{{ followedTags.length }}</div>
</template>
"#;

pub(crate) const WATCH_NON_IMMEDIATE_SFC: &str = r#"<script setup lang="ts">
import { ref, watch } from 'vue'
import { Paginator } from './support'
const tab = ref('list')
let paginator: Paginator
watch(tab, (newTab) => {
  paginator = new Paginator(newTab)
})
</script>

<template>
  <div>{{ paginator.key }}</div>
</template>
"#;

pub(crate) const EARLY_RETURN_SFC: &str = r#"<script setup lang="ts">
import { Paginator, flag } from './support'
let paginator: Paginator
function init() {
  if (!flag) return
  paginator = new Paginator('a')
}
init()
</script>

<template>
  <div>{{ paginator.key }}</div>
</template>
"#;

pub(crate) const NEVER_ASSIGNED_SFC: &str = r#"<script setup lang="ts">
import { Paginator } from './support'
let paginator: Paginator
</script>

<template>
  <div>{{ paginator.key }}</div>
</template>
"#;

pub(crate) const DIRECT_INIT_SFC: &str = r#"<script setup lang="ts">
import { Paginator } from './support'
const paginator = new Paginator('a')
</script>

<template>
  <div>{{ paginator.key }}</div>
</template>
"#;

pub(crate) const TOP_LEVEL_AWAIT_ASSIGN_SFC: &str = r#"<script setup lang="ts">
import { fetchTags } from './support'
let followedTags: string[]
followedTags = await fetchTags()
</script>

<template>
  <div>{{ followedTags.length }}</div>
</template>
"#;

pub(crate) const REASSIGNMENT_SFC: &str = r#"<script setup lang="ts">
import { Paginator } from './support'
let paginator: Paginator = new Paginator('a')
paginator = new Paginator('b')
</script>

<template>
  <div>{{ paginator.key }}</div>
</template>
"#;

pub(crate) const DEFINITE_ASSERTION_SFC: &str = r#"<script setup lang="ts">
import { Paginator, flag } from './support'
let paginator!: Paginator
if (flag) {
  paginator = new Paginator('a')
}
</script>

<template>
  <div>{{ paginator.key }}</div>
</template>
"#;

/// JSX in the setup body: `<span />` is a type assertion under `lang="ts"`, so
/// the deferred-binding scan has to parse this one as TSX to see `paginator`.
pub(crate) const TSX_CONDITIONAL_SFC: &str = r#"<script setup lang="tsx">
import { Paginator, flag } from './support'
const icon = () => <span class="icon" />
let paginator: Paginator
if (flag) {
  paginator = new Paginator('a')
}
</script>

<template>
  <div>{{ paginator.key }}<component :is="icon" /></div>
</template>
"#;

pub(crate) const SCRIPT_CONDITIONAL_SFC: &str = r#"<script setup lang="ts">
import { Paginator, flag } from './support'
let paginator: Paginator
if (flag) {
  paginator = new Paginator('a')
}
const key = paginator.key
</script>

<template>
  <div>{{ key }}</div>
</template>
"#;

pub(crate) const SCRIPT_ASYNC_CALLBACK_SFC: &str = r#"<script setup lang="ts">
import { onMounted } from 'vue'
import { fetchTags } from './support'
let followedTags: string[]
onMounted(async () => {
  followedTags = await fetchTags()
})
const count = followedTags.length
</script>

<template>
  <div>{{ count }}</div>
</template>
"#;

pub(crate) const SCRIPT_NEVER_ASSIGNED_SFC: &str = r#"<script setup lang="ts">
import { Paginator } from './support'
let paginator: Paginator
const key = paginator.key
</script>

<template>
  <div>{{ key }}</div>
</template>
"#;

pub(crate) const SCRIPT_WATCH_IMMEDIATE_SFC: &str = r#"<script setup lang="ts">
import { ref, watch } from 'vue'
import { Paginator } from './support'
const tab = ref('list')
let paginator: Paginator
watch(tab, (newTab) => {
  paginator = new Paginator(newTab)
}, { immediate: true })
const key = paginator.key
</script>

<template>
  <div>{{ key }}</div>
</template>
"#;

pub(crate) const TEMPLATE_STILL_CHECKED_SFC: &str = r#"<script setup lang="ts">
import { Paginator, flag } from './support'
let paginator: Paginator
if (flag) {
  paginator = new Paginator('a')
}
</script>

<template>
  <div>{{ paginator.nope }}</div>
</template>
"#;

pub(crate) const TEMPLATE_VFOR_STILL_CHECKED_SFC: &str = r#"<script setup lang="ts">
import { fetchTags, flag } from './support'
let followedTags: string[]
if (flag) {
  followedTags = await fetchTags()
}
</script>

<template>
  <div v-for="tag in followedTags" :key="tag">{{ tag.nope }}</div>
</template>
"#;

pub(crate) const OPTIONS_API_CONDITIONAL_SFC: &str = r#"<script lang="ts">
import { defineComponent } from 'vue'
import { Paginator, flag } from './support'

export default defineComponent({
  setup() {
    let paginator: Paginator
    if (flag) {
      paginator = new Paginator('a')
    }
    return { key: paginator.key }
  },
})
</script>

<template>
  <div>options api</div>
</template>
"#;

pub(crate) const PLAIN_SCRIPT_SFC: &str = r#"<script lang="ts">
import { Paginator, flag } from './support'

let paginator: Paginator
if (flag) {
  paginator = new Paginator('a')
}
export default { name: 'PlainScriptConditional' }
</script>

<template>
  <div>{{ paginator.key }}</div>
</template>
"#;

pub(crate) const FIXTURES: &[(&str, &str)] = &[
    ("src/WatchImmediate.vue", WATCH_IMMEDIATE_SFC),
    ("src/ConditionalAwait.vue", CONDITIONAL_AWAIT_SFC),
    ("src/ConditionalAssign.vue", CONDITIONAL_ASSIGN_SFC),
    ("src/AsyncCallback.vue", ASYNC_CALLBACK_SFC),
    ("src/WatchNonImmediate.vue", WATCH_NON_IMMEDIATE_SFC),
    ("src/EarlyReturn.vue", EARLY_RETURN_SFC),
    ("src/NeverAssigned.vue", NEVER_ASSIGNED_SFC),
    ("src/DirectInit.vue", DIRECT_INIT_SFC),
    ("src/TopLevelAwaitAssign.vue", TOP_LEVEL_AWAIT_ASSIGN_SFC),
    ("src/Reassignment.vue", REASSIGNMENT_SFC),
    ("src/DefiniteAssertion.vue", DEFINITE_ASSERTION_SFC),
    ("src/ScriptConditional.vue", SCRIPT_CONDITIONAL_SFC),
    ("src/ScriptAsyncCallback.vue", SCRIPT_ASYNC_CALLBACK_SFC),
    ("src/ScriptNeverAssigned.vue", SCRIPT_NEVER_ASSIGNED_SFC),
    ("src/ScriptWatchImmediate.vue", SCRIPT_WATCH_IMMEDIATE_SFC),
    ("src/TemplateStillChecked.vue", TEMPLATE_STILL_CHECKED_SFC),
    (
        "src/TemplateVForStillChecked.vue",
        TEMPLATE_VFOR_STILL_CHECKED_SFC,
    ),
    ("src/OptionsApiConditional.vue", OPTIONS_API_CONDITIONAL_SFC),
];
