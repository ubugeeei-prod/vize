// CrossFile preset: Provide/Inject Tree
// Complex dependency injection patterns

import type { Preset } from "./crossfile";

export const PROVIDE_INJECT_PRESET: Omit<Preset, "icon"> = {
  id: "provide-inject",
  name: "Provide/Inject Tree",
  description: "Complex dependency injection patterns",
  files: {
    "App.vue": `<script setup lang="ts">
import { provide, ref, reactive, readonly } from 'vue'
import type { InjectionKey } from 'vue'
import ThemeProvider from './ThemeProvider.vue'

// === TYPED INJECTION KEYS ===
export const UserKey: InjectionKey<{ name: string; role: string }> = Symbol('user')
export const ConfigKey: InjectionKey<{ apiUrl: string }> = Symbol('config')

// \u2713 Provide typed values
const user = reactive({ name: 'Admin', role: 'admin' })
provide(UserKey, readonly(user))

// \u2713 Provide config
provide(ConfigKey, { apiUrl: 'https://api.example.com' })

// \u274C Untyped provide - consumers may use wrong type
provide('legacyData', { foo: 'bar' })

// \u274C Provide without consumer
provide('unusedKey', 'this is never injected')
</script>

<template>
  <div>
    <h1>Provide/Inject Patterns</h1>
    <ThemeProvider>
      <slot />
    </ThemeProvider>
  </div>
</template>`,

    "ThemeProvider.vue": `<script setup lang="ts">
import { provide, ref, computed, inject } from 'vue'
import type { InjectionKey, Ref, ComputedRef } from 'vue'
import SettingsPanel from './SettingsPanel.vue'

// === THEME INJECTION KEY ===
export interface ThemeContext {
  theme: Ref<'light' | 'dark'>
  toggleTheme: () => void
  isDark: ComputedRef<boolean>
}
export const ThemeKey: InjectionKey<ThemeContext> = Symbol('theme')

const theme = ref<'light' | 'dark'>('dark')
const toggleTheme = () => {
  theme.value = theme.value === 'light' ? 'dark' : 'light'
}
const isDark = computed(() => theme.value === 'dark')

provide(ThemeKey, {
  theme,
  toggleTheme,
  isDark,
})

// Also provide CSS variables approach
provide('cssVars', computed(() => ({
  '--bg-color': isDark.value ? '#1a1a1a' : '#ffffff',
  '--text-color': isDark.value ? '#ffffff' : '#1a1a1a',
})))
</script>

<template>
  <div :class="['theme-provider', theme]">
    <SettingsPanel />
    <slot />
  </div>
</template>`,

    "SettingsPanel.vue": `<script setup lang="ts">
import { inject } from 'vue'
import { ThemeKey, type ThemeContext } from './ThemeProvider.vue'
import { UserKey, ConfigKey } from './App.vue'

// \u2713 Typed inject with Symbol key
const theme = inject(ThemeKey)
if (!theme) {
  throw new Error('ThemeProvider not found')
}

// \u2713 Inject user with type safety
const user = inject(UserKey)

// \u274C Inject with default - may hide missing provider
const config = inject(ConfigKey, { apiUrl: 'http://localhost:3000' })

// \u274C Untyped inject - no type safety
const legacyData = inject('legacyData') as { foo: string }

// \u274C Inject key that doesn't exist (without default)
// const missing = inject('nonExistentKey')  // Would be undefined!

// \u274C Destructuring inject loses reactivity!
const { foo } = inject('legacyData') as { foo: string }
</script>

<template>
  <div class="settings-panel">
    <h2>Settings</h2>
    <p>Theme: {{ theme.theme.value }}</p>
    <p>User: {{ user?.name ?? 'Unknown' }}</p>
    <p>API: {{ config.apiUrl }}</p>
    <button @click="theme.toggleTheme">Toggle Theme</button>
  </div>
</template>`,

    "DeepChild.vue": `<script setup lang="ts">
import { inject, computed } from 'vue'
import { ThemeKey } from './ThemeProvider.vue'
import { UserKey } from './App.vue'

// \u2713 Inject works at any depth
const theme = inject(ThemeKey)
const user = inject(UserKey)

// \u2713 Create computed from injected values
const greeting = computed(() => {
  if (!user) return 'Hello!'
  return \`Hello, \${user.name}! You are \${user.role}\`
})

const themeClass = computed(() => theme?.isDark.value ? 'dark-mode' : 'light-mode')
</script>

<template>
  <div :class="['deep-child', themeClass]">
    <h3>Deep Child Component</h3>
    <p>{{ greeting }}</p>
    <p v-if="theme">Current theme: {{ theme.theme.value }}</p>
  </div>
</template>`,
  },
};
