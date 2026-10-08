// Authored examples retained from the previous category reference.
export const manual10 = {
  "vue/valid-v-if": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <div v-if></div>\n  <div v-if=""></div>\n  <div v-if="ready" v-else></div>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <div v-if="ready"></div>\n  <div v-if="count > 0"></div>\n</template>',
    },
    evidence: "docs/content/rules/vue-directive-validity.md",
  },
  "vue/valid-v-memo": {
    bad: {
      language: "vue",
      source: "<template>\n  <div v-memo></div>\n</template>",
    },
    good: {
      language: "vue",
      source: '<template>\n  <div v-memo="[valueA, valueB]">{{ label }}</div>\n</template>',
    },
    evidence: "docs/content/rules/vue-directive-validity.md",
  },
  "vue/valid-v-model": {
    bad: {
      language: "vue",
      source: '<template>\n  <div v-model="value"></div>\n  <input v-model />\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <input v-model="value" />\n  <select v-model="selected"></select>\n  <textarea v-model="text"></textarea>\n  <MyInput v-model="value" />\n</template>',
    },
    evidence: "docs/content/rules/vue-directive-validity.md",
  },
  "vue/valid-v-on": {
    bad: {
      language: "vue",
      source: "<template>\n  <div v-on></div>\n  <div @></div>\n  <div @click></div>\n</template>",
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <div @click="handleClick"></div>\n  <div v-on="{ click: handleClick }"></div>\n</template>',
    },
    evidence: "docs/content/rules/vue-directive-validity.md",
  },
  "vue/valid-v-show": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <div v-show></div>\n  <template v-show="ready"><div></div></template>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <div v-show="ready"></div>\n  <div v-show="count > 0"></div>\n</template>',
    },
    evidence: "docs/content/rules/vue-directive-validity.md",
  },
  "vue/valid-v-slot": {
    bad: {
      language: "vue",
      source:
        "<template>\n  <div v-slot:header></div>\n  <MyComponent v-slot v-slot:header />\n  <template v-slot:header v-slot:footer />\n</template>",
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>\n  <MyComponent>\n    <template #header>Header</template>\n  </MyComponent>\n</template>',
    },
    evidence: "docs/content/rules/vue-directive-validity.md",
  },
  "vue/warn-custom-block": {
    bad: {
      language: "vue",
      source:
        '<i18n>\n{ "en": { "hello": "Hello" } }\n</i18n>\n\n<template>\n  <p>{{ hello }}</p>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <p>{{ hello }}</p>\n</template>\n\n<script setup lang="ts">\nconst hello = "Hello";\n</script>',
    },
    evidence: "docs/content/rules/vue-sfc.md",
  },
  "vue/warn-custom-directive": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <input v-focus />\n  <input v-mask="\'###-####\'" />\n  <div v-click-outside="handleClose"></div>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <div v-if="ready"></div>\n  <input v-model="value" />\n  <button type="button" @click="onClick">Save</button>\n</template>',
    },
    evidence: "docs/content/rules/vue-directives.md",
  },
};
