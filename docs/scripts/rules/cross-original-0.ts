// Complete project contexts retained from the original reference.
export const original0 = {
  "unmatched-inject": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./App.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
    },
    bad: {
      "keys/theme.ts":
        'import type { InjectionKey, Ref } from "vue";\n\nexport interface Theme {\n  color: string;\n}\n\nexport const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");',
      "App.vue":
        '<script setup lang="ts">\nimport ThemeLabel from "./ThemeLabel.vue";\n</script>\n\n<template>\n  <ThemeLabel />\n</template>',
      "ThemeLabel.vue":
        '<script setup lang="ts">\nimport { inject } from "vue";\nimport { ThemeKey } from "./keys/theme";\n\nconst theme = inject(ThemeKey);\n</script>',
    },
    good: {
      "keys/theme.ts":
        'import type { InjectionKey, Ref } from "vue";\n\nexport interface Theme {\n  color: string;\n}\n\nexport const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");',
      "App.vue":
        '<script setup lang="ts">\nimport { provide, ref } from "vue";\nimport ThemeLabel from "./ThemeLabel.vue";\nimport { ThemeKey, type Theme } from "./keys/theme";\n\nconst theme = ref<Theme>({ color: "blue" });\nprovide(ThemeKey, theme);\n</script>\n\n<template>\n  <ThemeLabel />\n</template>',
      "ThemeLabel.vue":
        '<script setup lang="ts">\nimport { inject } from "vue";\nimport { ThemeKey } from "./keys/theme";\n\nconst theme = inject(ThemeKey);\n</script>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
  "unused-provide": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./App.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
      "keys/theme.ts":
        'import type { InjectionKey, Ref } from "vue";\nexport interface Theme { color: string; }\nexport const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");',
    },
    bad: {
      "App.vue":
        '<script setup lang="ts">\nimport { provide, ref } from "vue";\nimport Dashboard from "./Dashboard.vue";\nimport { ThemeKey, type Theme } from "./keys/theme";\n\nconst theme = ref<Theme>({ color: "blue" });\nprovide(ThemeKey, theme);\n</script>\n\n<template>\n  <Dashboard />\n</template>',
      "Dashboard.vue": "<template>\n  <h1>Dashboard</h1>\n</template>",
    },
    good: {
      "App.vue":
        '<script setup lang="ts">\nimport { provide, ref } from "vue";\nimport Dashboard from "./Dashboard.vue";\nimport { ThemeKey, type Theme } from "./keys/theme";\n\nconst theme = ref<Theme>({ color: "blue" });\nprovide(ThemeKey, theme);\n</script>\n\n<template>\n  <Dashboard />\n</template>',
      "Dashboard.vue":
        '<script setup lang="ts">\nimport ThemeLabel from "./ThemeLabel.vue";\n</script>\n\n<template>\n  <ThemeLabel />\n</template>',
      "ThemeLabel.vue":
        '<script setup lang="ts">\nimport { inject } from "vue";\nimport { ThemeKey } from "./keys/theme";\n\nconst theme = inject(ThemeKey);\n</script>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
  "provide-without-symbol": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./ThemeProvider.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
    },
    bad: {
      "ThemeProvider.vue":
        '<script setup lang="ts">\nimport { provide, ref } from "vue";\nimport ThemeLabel from "./ThemeLabel.vue";\n\nconst theme = ref({ color: "blue" });\nprovide("theme", theme);\n</script>\n\n<template>\n  <ThemeLabel />\n</template>',
      "ThemeLabel.vue":
        '<script setup lang="ts">\nimport { inject } from "vue";\n\nconst theme = inject("theme");\n</script>',
    },
    good: {
      "ThemeProvider.vue":
        '<script setup lang="ts">\nimport { provide, ref } from "vue";\nimport ThemeLabel from "./ThemeLabel.vue";\nimport { ThemeKey, type Theme } from "./keys/theme";\n\nconst theme = ref<Theme>({ color: "blue" });\nprovide(ThemeKey, theme);\n</script>\n\n<template>\n  <ThemeLabel />\n</template>',
      "ThemeLabel.vue":
        '<script setup lang="ts">\nimport { inject } from "vue";\nimport { ThemeKey } from "./keys/theme";\n\nconst theme = inject(ThemeKey);\n</script>',
      "keys/theme.ts":
        'import type { InjectionKey, Ref } from "vue";\n\nexport interface Theme {\n  color: string;\n}\n\nexport const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
  "inject-without-symbol": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./ThemeProvider.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
      "keys/theme.ts":
        'import type { InjectionKey, Ref } from "vue";\nexport interface Theme { color: string; }\nexport const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");',
    },
    bad: {
      "ThemeProvider.vue":
        '<script setup lang="ts">\nimport { provide, ref } from "vue";\nimport ThemeLabel from "./ThemeLabel.vue";\n\nconst theme = ref({ color: "blue" });\nprovide("theme", theme);\n</script>\n\n<template>\n  <ThemeLabel />\n</template>',
      "ThemeLabel.vue":
        '<script setup lang="ts">\nimport { inject } from "vue";\n\nconst theme = inject("theme");\n</script>',
    },
    good: {
      "ThemeProvider.vue":
        '<script setup lang="ts">\nimport { provide, ref } from "vue";\nimport ThemeLabel from "./ThemeLabel.vue";\nimport { ThemeKey } from "./keys/theme";\n\nconst theme = ref({ color: "blue" });\nprovide(ThemeKey, theme);\n</script>\n\n<template>\n  <ThemeLabel />\n</template>',
      "ThemeLabel.vue":
        '<script setup lang="ts">\nimport { inject } from "vue";\nimport { ThemeKey } from "./keys/theme";\n\nconst theme = inject(ThemeKey);\n</script>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
  "non-reactive-provide": {
    shared: {
      "main.ts":
        'import { createApp } from "vue";\nimport Root from "./ThemeProvider.vue";\ncreateApp(Root).mount("#app");',
      "index.html": '<div id="app"></div>\n<script type="module" src="/main.ts"></script>',
    },
    bad: {
      "keys/theme.ts": 'export const ThemeKey = Symbol("theme");',
      "ThemeProvider.vue":
        '<script setup lang="ts">\nimport { provide } from "vue";\nimport ThemeLabel from "./ThemeLabel.vue";\nimport { ThemeKey } from "./keys/theme";\n\nconst theme = { color: "blue" };\nprovide(ThemeKey, theme);\n</script>\n\n<template>\n  <ThemeLabel />\n</template>',
      "ThemeLabel.vue":
        '<script setup lang="ts">\nimport { inject } from "vue";\nimport { ThemeKey } from "./keys/theme";\n\nconst theme = inject(ThemeKey);\n</script>',
    },
    good: {
      "keys/theme.ts": 'export const ThemeKey = Symbol("theme");',
      "ThemeProvider.vue":
        '<script setup lang="ts">\nimport { provide, ref } from "vue";\nimport ThemeLabel from "./ThemeLabel.vue";\nimport { ThemeKey } from "./keys/theme";\n\nconst theme = ref({ color: "blue" });\nprovide(ThemeKey, theme);\n</script>\n\n<template>\n  <ThemeLabel />\n</template>',
      "ThemeLabel.vue":
        '<script setup lang="ts">\nimport { inject } from "vue";\nimport { ThemeKey } from "./keys/theme";\n\nconst theme = inject(ThemeKey);\n</script>',
    },
    evidence: "Previous complete cross-file reference, with missing shared files restored",
  },
};
