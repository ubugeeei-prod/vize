// Authored examples retained from the previous category reference.
export const manual8 = {
  "vue/no-use-v-if-with-v-for": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <li v-for="item in items" v-if="item.visible" :key="item.id">\n    {{ item.name }}\n  </li>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\nconst visibleItems = computed(() => items.filter((item) => item.visible));\n</script>\n\n<template>\n  <li v-for="item in visibleItems" :key="item.id">\n    {{ item.name }}\n  </li>\n</template>',
    },
    evidence: "docs/content/rules/vue-template-structure.md",
  },
  "vue/no-v-html": {
    bad: {
      language: "vue",
      source: '<template>\n  <article v-html="content" />\n</template>',
    },
    good: {
      language: "vue",
      source: "<template>\n  <article>{{ content }}</article>\n</template>",
    },
    evidence: "docs/content/rules/vue-template-safety.md",
  },
  "vue/no-v-text-v-html-on-component": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <MyComponent v-html="content" />\n  <MyComponent v-text="content" />\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <div v-html="content"></div>\n  <component is="div" v-html="content" />\n  <MyComponent>{{ content }}</MyComponent>\n</template>',
    },
    evidence: "docs/content/rules/vue-template-safety.md",
  },
  "vue/permitted-contents": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <p><div>block in a paragraph</div></p>\n  <table><tr><td>row without tbody</td></tr></table>\n  <a href="#"><button type="button">nested control</button></a>\n  <ul><div>not a list item</div></ul>\n</template>',
    },
    good: {
      language: "vue",
      source:
        "<template>\n  <p><span>inline in a paragraph</span></p>\n  <table><tbody><tr><td>cell</td></tr></tbody></table>\n  <ul><li>list item</li><MyItem /></ul>\n</template>",
    },
    evidence: "docs/content/rules/vue-template-safety.md",
  },
  "vue/prefer-props-shorthand": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <MyComponent :foo="foo" />\n  <MyComponent :user-name="userName" />\n  <span :style="style" />\n  <div :aria-label="ariaLabel" />\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <MyComponent :foo />\n  <MyComponent :user-name />\n  <span :style />\n  <div :aria-label />\n  <MyComponent :foo="bar" />\n</template>',
    },
    evidence: "docs/content/rules/vue-formatting.md",
  },
  "vue/require-component-is": {
    bad: {
      language: "vue",
      source: "<template>\n  <component />\n</template>",
    },
    good: {
      language: "vue",
      source: '<template>\n  <component :is="currentComponent" />\n</template>',
    },
    evidence: "docs/content/rules/vue-components.md",
  },
  "vue/require-component-registration": {
    bad: {
      language: "vue",
      source:
        '<script setup lang="ts">\n// MyButton is never imported.\n</script>\n\n<template>\n  <MyButton>Save</MyButton>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\nimport MyButton from "./MyButton.vue";\n</script>\n\n<template>\n  <MyButton>Save</MyButton>\n</template>',
    },
    evidence: "docs/content/rules/vue-components.md",
  },
  "vue/require-scoped-style": {
    bad: {
      language: "vue",
      source: "<style>\n.button {\n  color: red;\n}\n</style>",
    },
    good: {
      language: "vue",
      source: "<style scoped>\n.button {\n  color: red;\n}\n</style>",
    },
    evidence: "docs/content/rules/vue-sfc.md",
  },
  "vue/require-v-for-key": {
    bad: {
      language: "vue",
      source: '<template>\n  <li v-for="item in items">{{ item.name }}</li>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <li v-for="item in items" :key="item.id">{{ item.name }}</li>\n</template>',
    },
    evidence: "docs/content/rules/vue-template-structure.md",
  },
  "vue/scoped-event-names": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <AudioPlayer\n    @playAudio="play"\n    @pauseAudio="pause"\n    @reloadAudio="reload"\n  />\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <AudioPlayer\n    @audio:play="play"\n    @audio:pause="pause"\n    @audio:reload="reload"\n  />\n</template>',
    },
    evidence: "docs/content/rules/vue-directives.md",
  },
};
