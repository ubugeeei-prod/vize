import type { RuleExample } from "./types.ts";
// Authored examples retained from the previous category reference.
export const manual9: Record<string, RuleExample> = {
  "vue/sfc-element-order": {
    bad: {
      language: "vue",
      source:
        '<style scoped>\n.panel {\n  color: red;\n}\n</style>\n<script setup lang="ts">\nconst label = "Save";\n</script>',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\nconst label = "Save";\n</script>\n\n<template>\n  <p>{{ label }}</p>\n</template>\n\n<style scoped>\np {\n  color: red;\n}\n</style>',
    },
    evidence: "docs/content/rules/vue-sfc.md",
  },
  "vue/single-style-block": {
    bad: {
      language: "vue",
      source:
        "<style scoped>\n.panel {\n  color: red;\n}\n</style>\n\n<style scoped>\n.title {\n  color: blue;\n}\n</style>",
    },
    good: {
      language: "vue",
      source: "<style scoped>\n.panel {\n  color: red;\n}\n.title {\n  color: blue;\n}\n</style>",
    },
    evidence: "docs/content/rules/vue-sfc.md",
  },
  "vue/use-unique-element-ids": {
    bad: {
      language: "vue",
      source: '<template>\n  <label for="email">Email</label>\n  <input id="email" />\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<script setup>\nimport { useId } from "vue";\n\nconst emailId = useId();\n</script>\n\n<template>\n  <label :for="emailId">Email</label>\n  <input :id="emailId" />\n</template>',
    },
    evidence: "docs/content/rules/accessibility-integrity.md",
  },
  "vue/use-v-on-exact": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">\n    Save\n  </button>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <button\n    type="button"\n    @click.exact="handleClick"\n    @click.ctrl="handleCtrlClick"\n  >\n    Save\n  </button>\n</template>',
    },
    evidence: "docs/content/rules/vue-directives.md",
  },
  "vue/v-bind-style": {
    bad: {
      language: "vue",
      source: '<template>\n  <div v-bind:class="panelClass"></div>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <div :class="panelClass"></div>\n</template>',
    },
    evidence: "docs/content/rules/vue-directives.md",
  },
  "vue/v-on-style": {
    bad: {
      language: "vue",
      source: '<template>\n  <div v-on:click="handleClick"></div>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <div @click="handleClick"></div>\n</template>',
    },
    evidence: "docs/content/rules/vue-directives.md",
  },
  "vue/v-slot-style": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <MyComponent #default="props">{{ props.item }}</MyComponent>\n  <MyComponent>\n    <template v-slot:header>Header</template>\n  </MyComponent>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <MyComponent v-slot="props">{{ props.item }}</MyComponent>\n  <MyComponent>\n    <template #header>Header</template>\n  </MyComponent>\n</template>',
    },
    evidence: "docs/content/rules/vue-directives.md",
  },
  "vue/valid-v-bind": {
    bad: {
      language: "vue",
      source: "<template>\n  <div v-bind></div>\n  <div :></div>\n</template>",
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <div :class="panelClass"></div>\n  <div v-bind="{ class: panelClass }"></div>\n  <div :loading></div>\n</template>',
    },
    evidence: "docs/content/rules/vue-directive-validity.md",
  },
  "vue/valid-v-else": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <div v-else="ready"></div>\n  <div v-else v-if="ready"></div>\n  <div v-else></div>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <div v-if="ready"></div>\n  <div v-else></div>\n</template>',
    },
    evidence: "docs/content/rules/vue-directive-validity.md",
  },
  "vue/valid-v-for": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <div v-for></div>\n  <div v-for=""></div>\n  <div v-for.stop="item in items"></div>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <div v-for="item in items" :key="item.id"></div>\n  <div v-for="(item, index) of items" :key="index"></div>\n</template>',
    },
    evidence: "docs/content/rules/vue-directive-validity.md",
  },
};
