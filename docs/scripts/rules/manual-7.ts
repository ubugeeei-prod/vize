import type { RuleExample } from "./types.ts";
// Authored examples retained from the previous category reference.
export const manual7: Record<string, RuleExample> = {
  "vue/no-inline-style": {
    bad: {
      language: "vue",
      source: '<template>\n  <div style="color: red">Text</div>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <div class="text-red">Text</div>\n  <span :class="{ \'text-red\': isRed }">Text</span>\n  <div :style="{ width: `${ratio}%` }">Text</div>\n</template>',
    },
    evidence: "docs/content/rules/vue-formatting.md",
  },
  "vue/no-multi-spaces": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <div  class="panel"></div>\n  <div class="panel"  id="main"></div>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <div class="panel"></div>\n  <div class="panel" id="main"></div>\n</template>',
    },
    evidence: "docs/content/rules/vue-formatting.md",
  },
  "vue/no-mutating-props": {
    bad: {
      language: "vue",
      source:
        '<script setup lang="ts">\nconst props = defineProps<{ count: number }>();\n\nprops.count++;\n</script>',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\nconst props = defineProps<{ count: number }>();\nconst emit = defineEmits<{ "update:count": [value: number] }>();\n\nfunction increment() {\n  emit("update:count", props.count + 1);\n}\n</script>',
    },
    evidence: "docs/content/rules/vue-components.md",
  },
  "vue/no-non-component-keep-alive-child": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <KeepAlive>\n    <div v-if="ready">\n      <UserCard />\n    </div>\n  </KeepAlive>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <KeepAlive>\n    <UserCard v-if="ready" />\n  </KeepAlive>\n  <KeepAlive>\n    <div v-show="opened">\n      <UserCard />\n    </div>\n  </KeepAlive>\n</template>',
    },
    evidence: "docs/content/rules/vue-components.md",
  },
  "vue/no-reserved-component-names": {
    bad: {
      language: "vue",
      source: '<script>\nexport default {\n  name: "button",\n};\n</script>',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\ndefineOptions({ name: "AppButton" });\n</script>\n\n<template>\n  <Transition>\n    <AppButton />\n  </Transition>\n</template>',
    },
    evidence: "docs/content/rules/vue-components.md",
  },
  "vue/no-src-attribute": {
    bad: {
      language: "vue",
      source:
        '<template src="./template.html"></template>\n<script src="./script.ts"></script>\n<style src="./style.css"></style>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <p>Hello</p>\n</template>\n\n<script setup lang="ts">\nconst label = "Hello";\n</script>\n\n<style scoped>\np {\n  color: red;\n}\n</style>',
    },
    evidence: "docs/content/rules/vue-sfc.md",
  },
  "vue/no-textarea-mustache": {
    bad: {
      language: "vue",
      source: "<template>\n  <textarea>{{ message }}</textarea>\n</template>",
    },
    good: {
      language: "vue",
      source: '<template>\n  <textarea v-model="message"></textarea>\n</template>',
    },
    evidence: "docs/content/rules/vue-template-safety.md",
  },
  "vue/no-unused-components": {
    bad: {
      language: "vue",
      source:
        '<script setup lang="ts">\nimport UserAvatar from "./UserAvatar.vue";\n</script>\n\n<template>\n  <p>{{ user.name }}</p>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\nimport UserAvatar from "./UserAvatar.vue";\n</script>\n\n<template>\n  <UserAvatar :user="user" />\n</template>',
    },
    evidence: "docs/content/rules/vue-components.md",
  },
  "vue/no-unused-properties": {
    bad: {
      language: "vue",
      source:
        '<script setup lang="ts">\ndefineProps<{ title: string; description: string }>();\n</script>\n\n<template>\n  <h1>{{ title }}</h1>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\ndefineProps<{ title: string; description: string }>();\n</script>\n\n<template>\n  <h1>{{ title }}</h1>\n  <p>{{ description }}</p>\n</template>',
    },
    evidence: "docs/content/rules/vue-components.md",
  },
  "vue/no-unused-vars": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>\n  <template v-slot="{ foo }">\n    <span>Hello</span>\n  </template>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>\n  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>\n  <template v-slot="{ data }">\n    <span>{{ data }}</span>\n  </template>\n</template>',
    },
    evidence: "docs/content/rules/vue-template-structure.md",
  },
};
