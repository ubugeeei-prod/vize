// Complete project contexts retained from the original reference.
export const original2 = {
  "destructuring-breaks-reactivity": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./UserPage.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
    },
    bad: {
      "UserPage.vue":
        '<script setup lang="ts">\nimport { reactive } from "vue";\nimport UserSummary from "./UserSummary.vue";\n\nconst user = reactive({ name: "Ada" });\n</script>\n\n<template>\n  <UserSummary :item="user" />\n</template>',
      "UserSummary.vue":
        '<script setup lang="ts">\nconst props = defineProps<{ item: { name: string } }>();\nconst { item } = props;\n</script>',
    },
    good: {
      "UserPage.vue":
        '<script setup lang="ts">\nimport { reactive } from "vue";\nimport UserSummary from "./UserSummary.vue";\n\nconst user = reactive({ name: "Ada" });\n</script>\n\n<template>\n  <UserSummary :item="user" />\n</template>',
      "UserSummary.vue":
        '<script setup lang="ts">\nimport { toRef } from "vue";\n\nconst props = defineProps<{ item: { name: string } }>();\nconst item = toRef(props, "item");\n</script>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
  "hydration-risk": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport App from "./App.vue";\ncreateApp(App).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
    },
    bad: {
      "App.vue":
        '<script setup lang="ts">\nimport { ref, watch } from "vue";\nconst count = ref(0);\nwatch(count.value, () => {});\n</script>\n<template><p>{{ count }}</p></template>',
    },
    good: {
      "App.vue":
        '<script setup lang="ts">\nimport { ref, watch } from "vue";\nconst count = ref(0);\nwatch(() => count.value, () => {});\n</script>\n<template><p>{{ count }}</p></template>',
    },
    evidence: "crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs: NonReactiveWatchSource",
  },
  "async-boundary": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./SearchPage.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
      "api.ts":
        "export interface Result { items: string[]; }\nexport async function load(query: string, options?: { signal?: AbortSignal }): Promise<Result> {\n  const response = await fetch(`/search?q=${encodeURIComponent(query)}`, options);\n  return response.json();\n}",
    },
    bad: {
      "SearchPage.vue":
        '<script setup lang="ts">\nimport { ref } from "vue";\nimport SearchResults from "./SearchResults.vue";\n\nconst query = ref("");\n</script>\n\n<template>\n  <SearchResults :query="query" />\n</template>',
      "SearchResults.vue":
        '<script setup lang="ts">\nimport { load, type Result } from "./api";\nimport { ref, watch } from "vue";\n\nconst props = defineProps<{ query: string }>();\nconst result = ref<Result | null>(null);\n\nwatch(\n  () => props.query,\n  async (value) => {\n    result.value = await load(value);\n  },\n);\n</script>',
    },
    good: {
      "SearchPage.vue":
        '<script setup lang="ts">\nimport { ref } from "vue";\nimport SearchResults from "./SearchResults.vue";\n\nconst query = ref("");\n</script>\n\n<template>\n  <SearchResults :query="query" />\n</template>',
      "SearchResults.vue":
        '<script setup lang="ts">\nimport { load, type Result } from "./api";\nimport { ref, watch } from "vue";\n\nconst props = defineProps<{ query: string }>();\nconst result = ref<Result | null>(null);\n\nwatch(\n  () => props.query,\n  async (value, _oldValue, onCleanup) => {\n    const controller = new AbortController();\n    let active = true;\n\n    onCleanup(() => {\n      active = false;\n      controller.abort();\n    });\n\n    const next = await load(value, { signal: controller.signal });\n    if (active) result.value = next;\n  },\n);\n</script>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
  "watcheffect-async": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./SearchPage.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
      "api.ts":
        "export interface Result { items: string[]; }\nexport async function load(query: string, options?: { signal?: AbortSignal }): Promise<Result> {\n  const response = await fetch(`/search?q=${encodeURIComponent(query)}`, options);\n  return response.json();\n}",
    },
    bad: {
      "SearchPage.vue":
        '<script setup lang="ts">\nimport { ref } from "vue";\nimport SearchResults from "./SearchResults.vue";\n\nconst query = ref("");\n</script>\n\n<template>\n  <SearchResults :query="query" />\n</template>',
      "SearchResults.vue":
        '<script setup lang="ts">\nimport { load, type Result } from "./api";\nimport { ref, watchEffect } from "vue";\n\nconst props = defineProps<{ query: string }>();\nconst result = ref<Result | null>(null);\n\nwatchEffect(async () => {\n  result.value = await load(props.query);\n});\n</script>',
    },
    good: {
      "SearchPage.vue":
        '<script setup lang="ts">\nimport { ref } from "vue";\nimport SearchResults from "./SearchResults.vue";\n\nconst query = ref("");\n</script>\n\n<template>\n  <SearchResults :query="query" />\n</template>',
      "SearchResults.vue":
        '<script setup lang="ts">\nimport { load, type Result } from "./api";\nimport { ref, watch } from "vue";\n\nconst props = defineProps<{ query: string }>();\nconst result = ref<Result | null>(null);\n\nwatch(\n  () => props.query,\n  async (value, _oldValue, onCleanup) => {\n    const controller = new AbortController();\n    let active = true;\n\n    onCleanup(() => {\n      active = false;\n      controller.abort();\n    });\n\n    const next = await load(value, { signal: controller.signal });\n    if (active) result.value = next;\n  },\n);\n</script>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
  "injected-async-mutation-race": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./StoreProvider.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
      "api.ts":
        "export async function loadCount(query: string, options?: { signal?: AbortSignal }): Promise<number> {\n  const response = await fetch(`/count?q=${encodeURIComponent(query)}`, options);\n  return Number(await response.text());\n}",
      "CountSummary.vue":
        '<script setup lang="ts">\nimport { inject } from "vue";\nimport { StoreKey } from "./keys/store";\nconst store = inject(StoreKey)!;\n</script>\n<template><p>{{ store.count }}</p></template>',
    },
    bad: {
      "keys/store.ts":
        'import type { InjectionKey } from "vue";\n\nexport interface Store {\n  count: number;\n}\n\nexport const StoreKey: InjectionKey<Store> = Symbol("store");',
      "StoreProvider.vue":
        '<script setup lang="ts">\nimport { provide, reactive } from "vue";\nimport CountLoader from "./CountLoader.vue";\nimport CountSummary from "./CountSummary.vue";\nimport { StoreKey, type Store } from "./keys/store";\n\nconst store = reactive<Store>({ count: 0 });\nprovide(StoreKey, store);\n</script>\n\n<template>\n  <CountLoader />\n  <CountSummary />\n</template>',
      "CountLoader.vue":
        '<script setup lang="ts">\nimport { loadCount } from "./api";\nimport { inject, ref, watch } from "vue";\nimport { StoreKey } from "./keys/store";\n\nconst store = inject(StoreKey)!;\nconst query = ref("");\n\nwatch(query, async (value) => {\n  store.count = await loadCount(value);\n});\n</script>',
    },
    good: {
      "keys/store.ts":
        'import type { InjectionKey } from "vue";\n\nexport interface Store {\n  count: number;\n}\n\nexport const StoreKey: InjectionKey<Store> = Symbol("store");',
      "StoreProvider.vue":
        '<script setup lang="ts">\nimport { provide, reactive } from "vue";\nimport CountLoader from "./CountLoader.vue";\nimport CountSummary from "./CountSummary.vue";\nimport { StoreKey, type Store } from "./keys/store";\n\nconst store = reactive<Store>({ count: 0 });\nprovide(StoreKey, store);\n\nfunction applyLoadedCount(count: number) {\n  store.count = count;\n}\n</script>\n\n<template>\n  <CountLoader @loaded="applyLoadedCount" />\n  <CountSummary />\n</template>',
      "CountLoader.vue":
        '<script setup lang="ts">\nimport { loadCount } from "./api";\nimport { ref, watch } from "vue";\n\nconst emit = defineEmits<{ loaded: [count: number] }>();\nconst query = ref("");\n\nwatch(query, async (value, _oldValue, onCleanup) => {\n  const controller = new AbortController();\n  let active = true;\n\n  onCleanup(() => {\n    active = false;\n    controller.abort();\n  });\n\n  const count = await loadCount(value, { signal: controller.signal });\n  if (active) emit("loaded", count);\n});\n</script>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
};
