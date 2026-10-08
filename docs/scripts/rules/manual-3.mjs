// Authored examples retained from the previous category reference.
export const manual3 = {
  "css/no-hardcoded-values": {
    bad: {
      language: "vue",
      source: "<style scoped>\n.button {\n  padding: 12px 16px;\n  color: #174ea6;\n}\n</style>",
    },
    good: {
      language: "vue",
      source:
        "<style scoped>\n.button {\n  padding: var(--space-3) var(--space-4);\n  color: var(--color-action-text);\n}\n</style>",
    },
    evidence: "docs/content/rules/musea-and-css.md",
  },
  "css/no-id-selectors": {
    bad: {
      language: "vue",
      source: "<style scoped>\n#submit {\n  font-weight: 600;\n}\n</style>",
    },
    good: {
      language: "vue",
      source: "<style scoped>\n.submit {\n  font-weight: 600;\n}\n</style>",
    },
    evidence: "docs/content/rules/musea-and-css.md",
  },
  "css/no-important": {
    bad: {
      language: "vue",
      source: "<style scoped>\n.button {\n  color: red !important;\n}\n</style>",
    },
    good: {
      language: "vue",
      source: "<style scoped>\n.button {\n  color: var(--button-color);\n}\n</style>",
    },
    evidence: "docs/content/rules/musea-and-css.md",
  },
  "css/no-v-bind-performance": {
    bad: {
      language: "vue",
      source: "<style scoped>\n.card {\n  transform: translateX(v-bind(offset));\n}\n</style>",
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />\n</template>',
    },
    evidence: "docs/content/rules/musea-and-css.md",
  },
  "css/prefer-logical-properties": {
    bad: {
      language: "vue",
      source: "<style scoped>\n.panel {\n  margin-left: 1rem;\n}\n</style>",
    },
    good: {
      language: "vue",
      source: "<style scoped>\n.panel {\n  margin-inline-start: 1rem;\n}\n</style>",
    },
    evidence: "docs/content/rules/musea-and-css.md",
  },
  "css/require-font-display": {
    bad: {
      language: "vue",
      source:
        '<style>\n@font-face {\n  font-family: "Inter";\n  src: url("/inter.woff2") format("woff2");\n}\n</style>',
    },
    good: {
      language: "vue",
      source:
        '<style>\n@font-face {\n  font-family: "Inter";\n  src: url("/inter.woff2") format("woff2");\n  font-display: swap;\n}\n</style>',
    },
    evidence: "docs/content/rules/musea-and-css.md",
  },
  "ecosystem/nuxt-prefer-nuxt-link": {
    bad: {
      language: "vue",
      source: '<template>\n  <a href="/settings">Settings</a>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <NuxtLink to="/settings">Settings</NuxtLink>\n</template>',
    },
    evidence: "docs/content/rules/ecosystem.md",
  },
  "ecosystem/pinia-prefer-store-to-refs": {
    bad: {
      language: "vue",
      source: '<script setup lang="ts">\nconst { name } = useUserStore();\n</script>',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\nconst store = useUserStore();\nconst { name } = storeToRefs(store);\n</script>',
    },
    evidence: "docs/content/rules/ecosystem.md",
  },
  "ecosystem/void-link-require-href": {
    bad: {
      language: "vue",
      source:
        '<script setup>\nimport { Link } from "@void/vue";\n</script>\n\n<template>\n  <Link>Settings</Link>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<script setup>\nimport { Link } from "@void/vue";\n</script>\n\n<template>\n  <Link href="/settings">Settings</Link>\n</template>',
    },
    evidence: "docs/content/rules/ecosystem.md",
  },
  "ecosystem/void-link-valid-method": {
    bad: {
      language: "vue",
      source:
        '<script setup>\nimport { Link } from "@void/vue";\n</script>\n\n<template>\n  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<script setup>\nimport { Link } from "@void/vue";\n</script>\n\n<template>\n  <Link href="/posts/1" method="DELETE">Delete</Link>\n</template>',
    },
    evidence: "docs/content/rules/ecosystem.md",
  },
};
