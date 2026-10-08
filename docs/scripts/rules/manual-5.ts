// Authored examples retained from the previous category reference.
export const manual5 = {
  "musea/no-empty-variant": {
    bad: {
      language: "vue",
      source: '<art title="Button" component="./Button.vue">\n  <variant name="primary" />\n</art>',
    },
    good: {
      language: "vue",
      source:
        '<art title="Button" component="./Button.vue">\n  <variant name="primary">\n    <Button tone="primary">Save</Button>\n  </variant>\n</art>',
    },
    evidence: "docs/content/rules/musea-and-css.md",
  },
  "musea/require-component": {
    bad: {
      language: "vue",
      source: '<art title="Button">\n  <variant name="primary" />\n</art>',
    },
    good: {
      language: "vue",
      source:
        '<script setup>\ndefineArt("./Button.vue", { title: "Button" });\n</script>\n\n<art>\n  <variant name="primary" />\n</art>',
    },
    evidence: "docs/content/rules/musea-and-css.md",
  },
  "musea/require-title": {
    bad: {
      language: "vue",
      source: '<art component="./Button.vue">\n  <variant name="primary" />\n</art>',
    },
    good: {
      language: "vue",
      source:
        '<script setup>\ndefineArt("./Button.vue", { title: "Button" });\n</script>\n\n<art>\n  <variant name="primary" />\n</art>',
    },
    evidence: "docs/content/rules/musea-and-css.md",
  },
  "musea/unique-variant-names": {
    bad: {
      language: "vue",
      source:
        '<art title="Button" component="./Button.vue">\n  <variant name="primary" />\n  <variant name="primary" />\n</art>',
    },
    good: {
      language: "vue",
      source:
        '<art title="Button" component="./Button.vue">\n  <variant name="primary" />\n  <variant name="secondary" />\n</art>',
    },
    evidence: "docs/content/rules/musea-and-css.md",
  },
  "musea/valid-variant": {
    bad: {
      language: "vue",
      source: '<art title="Button" component="./Button.vue">\n  <variant />\n</art>',
    },
    good: {
      language: "vue",
      source: '<art title="Button" component="./Button.vue">\n  <variant name="primary" />\n</art>',
    },
    evidence: "docs/content/rules/musea-and-css.md",
  },
  "script/no-options-api": {
    bad: {
      language: "vue",
      source:
        '<script lang="ts">\nexport default {\n  data() {\n    return { count: 0 };\n  },\n};\n</script>',
    },
    good: {
      language: "vue",
      source: '<script setup lang="ts" vapor>\nconst count = ref(0);\n</script>',
    },
    evidence: "docs/content/rules/type-and-script.md",
  },
  "ssr/no-browser-globals-in-ssr": {
    bad: {
      language: "vue",
      source: '<script setup lang="ts">\nconst width = window.innerWidth;\n</script>',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\nconst width = ref(0);\n\nonMounted(() => {\n  width.value = window.innerWidth;\n});\n</script>',
    },
    evidence: "docs/content/rules/ssr.md",
  },
  "ssr/no-hydration-mismatch": {
    bad: {
      language: "vue",
      source: "<template>\n  <p>{{ Math.random() }}</p>\n</template>",
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\nconst seed = useState("seed", () => "stable");\n</script>\n\n<template>\n  <p>{{ seed }}</p>\n</template>',
    },
    evidence: "docs/content/rules/ssr.md",
  },
  "vapor/no-inline-template": {
    bad: {
      language: "vue",
      source:
        "<template>\n  <LegacyCard inline-template>\n    <p>Profile</p>\n  </LegacyCard>\n</template>",
    },
    good: {
      language: "vue",
      source:
        "<template>\n  <LegacyCard>\n    <template #default>\n      <p>Profile</p>\n    </template>\n  </LegacyCard>\n</template>",
    },
    evidence: "docs/content/rules/vapor.md",
  },
  "vapor/no-vue-lifecycle-events": {
    bad: {
      language: "vue",
      source: '<template>\n  <input @vue:mounted="focusInput" />\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts" vapor>\nconst input = useTemplateRef<HTMLInputElement>("input");\n\nonMounted(() => {\n  input.value?.focus();\n});\n</script>\n\n<template>\n  <input ref="input" />\n</template>',
    },
    evidence: "docs/content/rules/vapor.md",
  },
};
