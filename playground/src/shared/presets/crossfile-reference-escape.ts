// CrossFile preset: Reference Escape
// Reactive references escaping scope (Rust-like tracking)

import type { Preset } from "./crossfile";

export const REFERENCE_ESCAPE_PRESET: Omit<Preset, "icon"> = {
  id: "reference-escape",
  name: "Reference Escape",
  description: "Reactive references escaping scope (Rust-like tracking)",
  files: {
    "App.vue": `<script setup lang="ts">
import { reactive, ref, provide } from 'vue'
import ChildComponent from './ChildComponent.vue'
import { useExternalStore } from './stores/external'

// === REFERENCE ESCAPE PATTERNS ===

const state = reactive({
  user: { name: 'Alice', permissions: ['read'] },
  items: [] as string[]
})

// \u274C ESCAPE: Passing reactive object to external function
// The external function may store a reference
useExternalStore().registerState(state)

// \u274C ESCAPE: Assigning to window/global
;(window as any).appState = state

// \u274C ESCAPE: Returning from setup to be used elsewhere
// (This is often intentional via provide, but needs awareness)
provide('state', state)

function addItem(item: string) {
  state.items.push(item)
}
</script>

<template>
  <div>
    <h1>Reference Escape Tracking</h1>
    <p>User: {{ state.user.name }}</p>
    <ChildComponent :state="state" @add="addItem" />
  </div>
</template>`,

    "ChildComponent.vue": `<script setup lang="ts">
import { inject, watch, onUnmounted } from 'vue'

const props = defineProps<{
  state: { user: { name: string }; items: string[] }
}>()

const emit = defineEmits<{
  add: [item: string]
}>()

// \u274C ESCAPE: Storing prop reference in external location
let cachedState: typeof props.state | null = null
function cacheState() {
  cachedState = props.state  // Reference escapes!
}

// \u274C ESCAPE: setTimeout/setInterval with reactive reference
setTimeout(() => {
  // This closure captures props.state
  console.log(props.state.user.name)
}, 1000)

// \u274C ESCAPE: Event listener with reactive reference
function setupListener() {
  document.addEventListener('click', () => {
    // Reference escapes to global event listener!
    console.log(props.state.items.length)
  })
}

// \u2713 CORRECT: Use local copy or computed if needed
import { computed, readonly } from 'vue'
const userName = computed(() => props.state.user.name)
const readonlyState = readonly(props.state)  // Prevent accidental mutations
</script>

<template>
  <div>
    <h2>Child Component</h2>
    <p>User: {{ userName }}</p>
    <button @click="emit('add', 'new item')">Add Item</button>
  </div>
</template>`,

    "stores/external.ts": `import { reactive } from 'vue'

interface State {
  user: { name: string; permissions: string[] }
  items: string[]
}

// This simulates an external store that holds references
class ExternalStore {
  // Using object type to store states by key
  private states: { [key: string]: State } = {}

  // \u274C This stores a reference to reactive object
  registerState(state: State) {
    // The reactive object is now stored externally
    // Mutations here affect the original!
    this.states['main'] = state

    // \u274C DANGER: External code can mutate your reactive state
    setTimeout(() => {
      state.user.name = 'Modified externally!'
    }, 5000)
  }

  getState(key: string) {
    return this.states[key]
  }
}

// Singleton - state persists across component lifecycle
const store = new ExternalStore()

export function useExternalStore() {
  return store
}
`,

    "SafePattern.vue": `<script setup lang="ts">
import { reactive, toRaw, readonly, shallowRef, markRaw, onUnmounted } from 'vue'

// === SAFE PATTERNS FOR REFERENCE MANAGEMENT ===

const state = reactive({
  data: { value: 1 }
})

// \u2713 SAFE: Pass raw copy to external APIs
function sendToAnalytics() {
  const raw = toRaw(state)
  const copy = structuredClone(raw)
  // analytics.track(copy)  // Safe - no reactive reference
}

// \u2713 SAFE: Use readonly for external exposure
const publicState = readonly(state)

// \u2713 SAFE: Use markRaw for data that shouldn't be reactive
const heavyObject = markRaw({
  largeArray: new Array(10000).fill(0),
  canvas: null as HTMLCanvasElement | null
})

// \u2713 SAFE: Proper cleanup for external references
let cleanupFn: (() => void) | null = null

function setupExternalListener() {
  const handler = () => {
    // Use state here
  }
  document.addEventListener('scroll', handler)
  cleanupFn = () => document.removeEventListener('scroll', handler)
}

onUnmounted(() => {
  cleanupFn?.()
})
</script>

<template>
  <div>
    <h2>Safe Reference Patterns</h2>
    <p>Value: {{ state.data.value }}</p>
  </div>
</template>`,
  },
};
