// CrossFile preset: Overview (default)
// General provide/inject and emit patterns

import type { Preset } from "./crossfile";

export const OVERVIEW_PRESET: Omit<Preset, "icon"> = {
  id: "default",
  name: "Overview",
  description: "General cross-file analysis patterns",
  files: {
    "App.vue": `<script setup lang="ts">
import { provide, ref } from 'vue'
import ParentComponent from './ParentComponent.vue'

// Provide theme to all descendants
const theme = ref<'light' | 'dark'>('dark')
provide('theme', theme)
provide('user', { name: 'John', id: 1 })

function handleUpdate(value: number) {
  console.log('Updated:', value)
}
</script>

<template>
  <div id="app" class="app-container">
    <ParentComponent
      title="Dashboard"
      @update="handleUpdate"
    />
  </div>
</template>`,

    "ParentComponent.vue": `<script setup lang="ts">
import { inject, ref, onMounted } from 'vue'
import ChildComponent from './ChildComponent.vue'

const props = defineProps<{
  title: string
}>()

const emit = defineEmits<{
  update: [value: number]
  'unused-event': []
}>()

const theme = inject<Ref<'light' | 'dark'>>('theme')

// ISSUE: Destructuring inject loses reactivity!
const { name } = inject('user') as { name: string; id: number }

const width = ref(0)
onMounted(() => {
  width.value = window.innerWidth
})
</script>

<template>
  <div :class="['parent', theme]">
    <h2>{{ title }}</h2>
    <p>User: {{ name }}</p>
    <ChildComponent
      :theme="theme"
      custom-attr="value"
      @change="emit('update', $event)"
    />
  </div>
</template>`,

    "ChildComponent.vue": `<script setup lang="ts">
import { ref, toRefs } from 'vue'

const props = defineProps<{
  theme?: string
}>()

const { theme } = toRefs(props)

const emit = defineEmits<{
  change: [value: number]
}>()

const items = ref([
  { id: 1, name: 'Item 1' },
  { id: 2, name: 'Item 2' },
])

function handleClick(item: { id: number; name: string }) {
  emit('change', item.id)
}
</script>

<template>
  <!-- ISSUE: Multiple root elements without v-bind="$attrs" -->
  <div class="child-header">
    <span>Theme: {{ theme }}</span>
  </div>
  <ul class="child-list">
    <li v-for="item in items" :key="item.id" @click="handleClick(item)">
      {{ item.name }}
    </li>
  </ul>
</template>`,
  },
};
