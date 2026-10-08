import type { RuleExample } from "./types.ts";
// Authored examples retained from the previous category reference.
export const manual6: Record<string, RuleExample> = {
  "vapor/prefer-static-class": {
    bad: {
      language: "vue",
      source:
        "<template>\n  <section :class=\"'panel panel-primary'\">Profile</section>\n</template>",
    },
    good: {
      language: "vue",
      source: '<template>\n  <section class="panel panel-primary">Profile</section>\n</template>',
    },
    evidence: "docs/content/rules/vapor.md",
  },
  "vue/attribute-order": {
    bad: {
      language: "vue",
      source: '<template>\n  <div @click="onClick" v-if="show" id="main"></div>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <div v-if="show" id="main" @click="onClick"></div>\n</template>',
    },
    evidence: "docs/content/rules/vue-formatting.md",
  },
  "vue/component-name-in-template-casing": {
    bad: {
      language: "vue",
      source:
        '<script setup>\nimport MyComponent from "./MyComponent.vue";\n</script>\n<template>\n  <my-component />\n  <myComponent />\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<script setup>\nimport MyComponent from "./MyComponent.vue";\n</script>\n<template>\n  <MyComponent />\n  <RouterView />\n  <slot />\n</template>',
    },
    evidence: "docs/content/rules/vue-components.md",
  },
  "vue/html-quotes": {
    bad: {
      language: "vue",
      source:
        "<template>\n  <div class='foo'></div>\n  <div class=foo></div>\n  <div v-if='ready'></div>\n</template>",
    },
    good: {
      language: "vue",
      source: '<template>\n  <div class="foo"></div>\n  <div v-if="ready"></div>\n</template>',
    },
    evidence: "docs/content/rules/vue-formatting.md",
  },
  "vue/html-self-closing": {
    bad: {
      language: "vue",
      source: "<template>\n  <MyComponent></MyComponent>\n  <img>\n  <br>\n</template>",
    },
    good: {
      language: "vue",
      source:
        "<template>\n  <MyComponent />\n  <div></div>\n  <div />\n  <img />\n  <br />\n  <div>content</div>\n</template>",
    },
    evidence: "docs/content/rules/vue-formatting.md",
  },
  "vue/mustache-interpolation-spacing": {
    bad: {
      language: "vue",
      source:
        "<template>\n  <div>{{text}}</div>\n  <div>{{ text}}</div>\n  <div>{{text }}</div>\n</template>",
    },
    good: {
      language: "vue",
      source:
        "<template>\n  <div>{{ text }}</div>\n  <div>{{ foo.bar }}</div>\n  <div>{{ foo + bar }}</div>\n</template>",
    },
    evidence: "docs/content/rules/vue-formatting.md",
  },
  "vue/no-boolean-attr-value": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <input disabled="disabled" />\n  <input checked="checked" />\n  <button disabled="true">Save</button>\n</template>',
    },
    good: {
      language: "vue",
      source:
        "<template>\n  <input disabled />\n  <input checked />\n  <button disabled>Save</button>\n</template>",
    },
    evidence: "docs/content/rules/vue-formatting.md",
  },
  "vue/no-child-content": {
    bad: {
      language: "vue",
      source: '<template>\n  <p v-text="message">Fallback text</p>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <p v-text="message" />\n</template>',
    },
    evidence: "docs/content/rules/vue-template-structure.md",
  },
  "vue/no-dupe-v-else-if": {
    bad: {
      language: "vue",
      source:
        "<template>\n  <p v-if=\"status === 'ready'\">Ready</p>\n  <p v-else-if=\"status === 'ready'\">Still ready</p>\n</template>",
    },
    good: {
      language: "vue",
      source:
        "<template>\n  <p v-if=\"status === 'ready'\">Ready</p>\n  <p v-else-if=\"status === 'loading'\">Loading</p>\n</template>",
    },
    evidence: "docs/content/rules/vue-template-structure.md",
  },
  "vue/no-duplicate-attributes": {
    bad: {
      language: "vue",
      source: '<template>\n  <button class="primary" class="large">Save</button>\n</template>',
    },
    good: {
      language: "vue",
      source: '<template>\n  <button class="primary large">Save</button>\n</template>',
    },
    evidence: "docs/content/rules/vue-template-safety.md",
  },
};
