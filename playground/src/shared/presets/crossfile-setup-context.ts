// CrossFile preset: Setup Context
// Vue APIs called outside setup (CSRP/Memory Leak)

import type { Preset } from "./crossfile";

export const SETUP_CONTEXT_PRESET: Omit<Preset, "icon"> = {
  id: "setup-context",
  name: "Setup Context",
  description: "Vue APIs called outside setup (CSRP/Memory Leak)",
  files: {
    "App.vue": `<script setup lang="ts">
import ComponentWithLeaks from './ComponentWithLeaks.vue'
import SafeComponent from './SafeComponent.vue'
</script>

<template>
  <div>
    <h1>Setup Context Violations</h1>
    <p>CSRP = Cross-request State Pollution (SSR)</p>
    <p>Memory Leaks from watchers created outside setup</p>
    <ComponentWithLeaks />
    <SafeComponent />
  </div>
</template>`,

    "ComponentWithLeaks.vue": `<script setup lang="ts">
import { ref, watch, onMounted, computed, provide, inject } from 'vue'
import { createGlobalState } from './utils/state'
</script>

<script lang="ts">
// \u26A0\uFE0F WARNING: Module-level Vue APIs cause issues!

import { ref, reactive, watch, computed, provide } from 'vue'

// \u274C CSRP Risk: Module-level reactive state is shared across requests in SSR
const globalCounter = ref(0)

// \u274C CSRP Risk: Module-level reactive object
const sharedState = reactive({
  users: [],
  settings: {}
})

// \u274C Memory Leak: Watch created outside setup is never cleaned up
watch(globalCounter, (val) => {
  console.log('Counter changed:', val)
})

// \u274C Memory Leak: Computed outside setup
const doubledCounter = computed(() => globalCounter.value * 2)

// \u274C Invalid: Provide outside setup
// provide('counter', globalCounter)  // This would throw!

export default {
  name: 'ComponentWithLeaks'
}
</script>

<template>
  <div class="warning-box">
    <h2>Component with Issues</h2>
    <p>Global counter: {{ globalCounter }}</p>
    <p>This component has CSRP risks and memory leaks!</p>
  </div>
</template>`,

    "SafeComponent.vue": `<script setup lang="ts">
import { ref, reactive, watch, computed, provide, onUnmounted } from 'vue'

// \u2713 CORRECT: All Vue APIs inside setup context

// \u2713 Component-scoped reactive state
const counter = ref(0)
const state = reactive({
  items: [] as string[]
})

// \u2713 Watch inside setup - auto-cleaned up
watch(counter, (val) => {
  console.log('Counter changed:', val)
})

// \u2713 Computed inside setup
const doubled = computed(() => counter.value * 2)

// \u2713 Provide inside setup
provide('counter', counter)

// \u2713 If you need manual cleanup
const customEffect = () => {
  // some side effect
}
onUnmounted(() => {
  // cleanup
})

function increment() {
  counter.value++
}
</script>

<template>
  <div class="safe-box">
    <h2>Safe Component</h2>
    <p>Counter: {{ counter }} (doubled: {{ doubled }})</p>
    <button @click="increment">Increment</button>
    <p>All Vue APIs properly scoped to setup context</p>
  </div>
</template>`,

    "utils/state.ts": `import { ref, reactive, computed, watch } from 'vue'

// \u274C DANGEROUS: Factory function that creates reactive state at module level
// Each import shares the same state!

// This file demonstrates why you should NOT do this:

const moduleState = reactive({
  value: 0
})

// \u274C Module-level watch - memory leak!
watch(() => moduleState.value, (v) => console.log(v))

// \u2713 CORRECT: Factory function that creates fresh state per call
export function createGlobalState() {
  const state = reactive({
    value: 0
  })

  // This watch will only be created when the function is called
  // inside a setup context, ensuring proper cleanup
  return {
    state,
    increment: () => state.value++
  }
}

// \u2713 CORRECT: Use VueUse's createGlobalState for shared state
// import { createGlobalState } from '@vueuse/core'
// export const useGlobalState = createGlobalState(() => reactive({ count: 0 }))
`,
  },
};
