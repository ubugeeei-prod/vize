// CrossFile preset: Reactivity Loss
// Patterns that break Vue reactivity

import type { Preset } from "./crossfile";

export const REACTIVITY_PRESET: Omit<Preset, "icon"> = {
  id: "reactivity-loss",
  name: "Reactivity Loss",
  description: "Patterns that break Vue reactivity",
  files: {
    "App.vue": `<script setup lang="ts">
import { reactive, ref, provide } from 'vue'
import ChildComponent from './ChildComponent.vue'

// === Correct Usage ===
const state = reactive({
  count: 0,
  user: { name: 'Alice', age: 25 }
})

// === ANTI-PATTERNS: Reactivity Loss ===

// 1. Destructuring reactive object breaks reactivity
const { count, user } = state  // \u274C count is now a plain number

// 2. Spreading reactive object breaks reactivity
const copiedState = { ...state }  // \u274C No longer reactive

// 3. Reassigning reactive variable breaks reactivity
let dynamicState = reactive({ value: 1 })
dynamicState = reactive({ value: 2 })  // \u274C Original tracking lost

// 4. Extracting primitive from ref
const countRef = ref(10)
const primitiveValue = countRef.value  // \u274C Just a number, not reactive

provide('state', state)
</script>

<template>
  <div>
    <h1>Reactivity Loss Patterns</h1>
    <p>Count: {{ count }}</p>
    <p>User: {{ user.name }}</p>
    <ChildComponent />
  </div>
</template>`,

    "ChildComponent.vue": `<script setup lang="ts">
import { inject, computed, toRefs, toRef } from 'vue'

const state = inject('state') as { count: number; user: { name: string } }

// === ANTI-PATTERNS ===

// 1. Destructuring inject result (this will trigger a warning)
const { count } = state  // \u274C Loses reactivity

// 2. This one is intentionally suppressed with @vize forget
// @vize forget: intentionally reading one-time value
const userName = state.user.name  // This warning is suppressed

// === CORRECT PATTERNS ===

// Use toRef for single property
const countRef = toRef(state, 'count')

// Use toRefs for multiple properties
const { user } = toRefs(state as any)

// Use computed for derived values
const displayName = computed(() => state.user.name.toUpperCase())
</script>

<template>
  <div>
    <h2>Child Component</h2>
    <p>Broken count: {{ count }}</p>
    <p>Reactive count: {{ countRef }}</p>
    <p>Display name: {{ displayName }}</p>
  </div>
</template>`,

    "stores/user.ts": `import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useUserStore = defineStore('user', () => {
  const username = ref('john_doe')
  const email = ref('john@example.com')

  const displayName = computed(() => username.value.toUpperCase())

  function updateUser(name: string, mail: string) {
    username.value = name
    email.value = mail
  }

  return { username, email, displayName, updateUser }
})
`,

    "StoreExample.vue": `<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useUserStore } from './stores/user'

const userStore = useUserStore()

// \u274C WRONG: Destructuring Pinia store loses reactivity for state/getters
const { username, email } = userStore

// \u2713 CORRECT: Use storeToRefs for reactive state/getters
// const { username, email } = storeToRefs(userStore)

// \u2713 Actions can be destructured directly (they're just functions)
// const { updateUser } = userStore
</script>

<template>
  <div>
    <p>Username: {{ username }}</p>
    <p>Email: {{ email }}</p>
  </div>
</template>`,

    "SpreadPattern.vue": `<script setup lang="ts">
import { reactive, ref, toRaw } from 'vue'

interface User {
  id: number
  name: string
  settings: { theme: string }
}

const user = reactive<User>({
  id: 1,
  name: 'Bob',
  settings: { theme: 'dark' }
})

// === SPREAD ANTI-PATTERNS ===

// \u274C Spreading reactive object
const userCopy = { ...user }

// \u274C Spreading in function call
function logUser(u: User) {
  console.log(u)
}
logUser({ ...user })

// \u274C Array spread on reactive array
const items = reactive([1, 2, 3])
const itemsCopy = [...items]

// === CORRECT PATTERNS ===

// \u2713 Use toRaw if you need plain object
const rawUser = toRaw(user)

// \u2713 Clone with structuredClone for deep copy
const deepCopy = structuredClone(toRaw(user))

// \u2713 Pass reactive object directly
logUser(user)
</script>

<template>
  <div>
    <p>Original: {{ user.name }}</p>
    <p>Copy (not reactive): {{ userCopy.name }}</p>
  </div>
</template>`,
  },
};
