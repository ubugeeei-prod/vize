import type { JsonValue, RuleExample } from "./types.ts";

const vue = (source: string) => ({ language: "vue", source });

function example(
  bad: string,
  good: string,
  evidence: string,
  ruleOptions: JsonValue,
  eslintDifference?: string,
): RuleExample {
  return {
    bad: vue(bad),
    good: vue(good),
    evidence,
    ruleOptions,
    ...(eslintDifference ? { eslintDifference: { en: eslintDifference } } : {}),
  };
}

const props = "crates/vize_patina/src/rules/vue/configured/props.rs";
const template = "crates/vize_patina/src/rules/vue/configured/template.rs";
const events = "crates/vize_patina/src/rules/vue/configured/template_events.rs";
const names = "crates/vize_patina/src/rules/vue/configured/names.rs";

export const manual12: Record<string, RuleExample> = {
  "vue/max-props": example(
    '<script setup lang="ts">\ndefineProps<{ a: number; b: number; c: number }>();\n</script>\n<template>\n  <p>props</p>\n</template>',
    '<script setup lang="ts">\ndefineProps<{ a: number }>();\n</script>\n<template>\n  <p>props</p>\n</template>',
    props,
    { maxProps: 2 },
  ),
  "vue/max-template-depth": example(
    "<template>\n  <div>\n    <section>\n      <div>\n        <span>deep</span>\n      </div>\n    </section>\n  </div>\n</template>",
    "<template>\n  <p>ok</p>\n</template>",
    template,
    { maxDepth: 3 },
  ),
  "vue/no-restricted-block": example(
    '<template>\n  <p>ok</p>\n</template>\n<i18n lang="json">{}</i18n>',
    "<template>\n  <p>ok</p>\n</template>",
    "crates/vize_patina/src/rules/vue/configured/blocks.rs",
    ["i18n"],
  ),
  "vue/no-restricted-call-after-await": example(
    '<script lang="ts">\nimport { defineComponent, onMounted } from "vue";\nexport default defineComponent({\n  async setup() {\n    await Promise.resolve();\n    onMounted(() => {});\n  },\n});\n</script>\n<template>\n  <p>ok</p>\n</template>',
    '<script lang="ts">\nimport { defineComponent, onMounted } from "vue";\nexport default defineComponent({\n  setup() {\n    onMounted(() => {});\n  },\n});\n</script>\n<template>\n  <p>ok</p>\n</template>',
    "crates/vize_patina/src/rules/vue/configured/await_call.rs",
    [{ module: "vue", path: "onMounted" }],
  ),
  "vue/no-restricted-class": example(
    '<template>\n  <div class="legacy">x</div>\n</template>',
    '<template>\n  <div class="current">x</div>\n</template>',
    template,
    ["legacy"],
  ),
  "vue/no-restricted-component-names": example(
    '<script setup lang="ts">\ndefineOptions({ name: "RestrictedShowcase" });\n</script>\n<template>\n  <p>ok</p>\n</template>',
    '<script setup lang="ts">\ndefineOptions({ name: "Widget" });\n</script>\n<template>\n  <p>ok</p>\n</template>',
    names,
    ["RestrictedShowcase"],
  ),
  "vue/no-restricted-component-options": example(
    '<script setup lang="ts">\ndefineOptions({ inheritAttrs: false });\n</script>\n<template>\n  <p>ok</p>\n</template>',
    '<script setup lang="ts">\ndefineOptions({ name: "Widget" });\n</script>\n<template>\n  <p>ok</p>\n</template>',
    names,
    ["inheritAttrs"],
  ),
  "vue/no-restricted-custom-event": example(
    '<template>\n  <button type="button" @click="$emit(\'legacyEvent\')">x</button>\n</template>',
    '<template>\n  <button type="button" @click="$emit(\'save\')">x</button>\n</template>',
    events,
    ["legacyEvent"],
    "Template `$emit` calls are reported. A call through a `setup()` parameter or `this.$emit` stays quiet, and so does the binding `defineEmits` returns.",
  ),
  "vue/no-restricted-html-elements": example(
    "<template>\n  <marquee>x</marquee>\n</template>",
    "<template>\n  <p>x</p>\n</template>",
    template,
    ["marquee"],
  ),
  "vue/no-restricted-props": example(
    '<script setup lang="ts">\ndefineProps<{ third?: number }>();\n</script>\n<template>\n  <p>ok</p>\n</template>',
    '<script setup lang="ts">\ndefineProps<{ first?: number }>();\n</script>\n<template>\n  <p>ok</p>\n</template>',
    props,
    ["third"],
  ),
  "vue/no-restricted-static-attribute": example(
    '<template>\n  <button type="button" data-legacy="x">x</button>\n</template>',
    '<template>\n  <button type="button">x</button>\n</template>',
    events,
    ["data-legacy"],
  ),
  "vue/no-restricted-v-on": example(
    '<template>\n  <button type="button" @legacy-click="save">x</button>\n</template>',
    '<template>\n  <button type="button" @click="save">x</button>\n</template>',
    events,
    ["legacy-click"],
  ),
  "vue/no-unsupported-features": example(
    '<script setup lang="ts">\nconst model = defineModel<string>();\n</script>\n<template>\n  <p>{{ model }}</p>\n</template>',
    '<script setup lang="ts">\nconst label = "ok";\n</script>\n<template>\n  <p>{{ label }}</p>\n</template>',
    names,
    { version: "^3.3.0" },
    "The configured minimum is compared with the script macros `defineModel` (3.4.0), `defineOptions` (3.3.0), and `defineSlots` (3.3.0). Template syntax such as `v-memo`, slot shorthand, and dynamic directive arguments is outside this rule. An empty version checks nothing, and `ignores` skips a macro id such as `define-model`.",
  ),
};
