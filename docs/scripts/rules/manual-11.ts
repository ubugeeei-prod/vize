import type { JsonValue, RuleExample } from "./types.ts";

const template = (bad: string, good: string, ruleOptions?: JsonValue): RuleExample => ({
  bad: { language: "vue", source: `<template>\n${bad}\n</template>` },
  good: { language: "vue", source: `<template>\n${good}\n</template>` },
  evidence: "crates/vize_patina/src/rules/vue/compatibility_template.rs",
  ruleOptions,
});

const sfc = (bad: string, good: string, ruleOptions?: JsonValue): RuleExample => ({
  bad: { language: "vue", source: bad },
  good: { language: "vue", source: good },
  evidence: "crates/vize_patina/src/rules/vue/compatibility_sfc.rs",
  ruleOptions,
});

export const manual11: Record<string, RuleExample> = {
  "vue/camelcase": template(
    "  <p>{{ { user_name: value } }}</p>",
    "  <p>{{ { userName: value } }}</p>",
  ),
  "vue/dot-notation": template('  <p>{{ user["name"] }}</p>', "  <p>{{ user.name }}</p>"),
  "vue/eqeqeq": template("  <p>{{ value == null }}</p>", "  <p>{{ value === null }}</p>"),
  "vue/no-console": template(
    '  <button @click="console.log(value)">Save</button>',
    '  <button @click="logValue">Save</button>',
  ),
  "vue/no-constant-condition": template(
    "  <p>{{ true ? primary : fallback }}</p>",
    "  <p>{{ ready ? primary : fallback }}</p>",
  ),
  "vue/no-empty-pattern": template(
    "  <p>{{ items.map(({}) => 1) }}</p>",
    "  <p>{{ items.map(({ id }) => id) }}</p>",
  ),
  "vue/no-implicit-coercion": template("  <p>{{ !!value }}</p>", "  <p>{{ Boolean(value) }}</p>"),
  "vue/no-irregular-whitespace": template(
    "  <p>{{ value + other }}</p>",
    "  <p>{{ value + other }}</p>",
  ),
  "vue/no-loss-of-precision": template(
    "  <p>{{ 9007199254740993 }}</p>",
    "  <p>{{ preciseValue }}</p>",
  ),
  "vue/no-negated-condition": template(
    "  <p>{{ !ready ? fallback : primary }}</p>",
    "  <p>{{ ready ? primary : fallback }}</p>",
  ),
  "vue/no-sparse-arrays": template("  <p>{{ [1,, 3] }}</p>", "  <p>{{ [1, 2, 3] }}</p>"),
  "vue/no-useless-concat": template(
    '  <p>{{ "hello" + " world" }}</p>',
    '  <p>{{ "hello world" }}</p>',
  ),
  "vue/object-shorthand": template(
    '  <Widget :options="{ value: value }" />',
    '  <Widget :options="{ value }" />',
  ),
  "vue/prefer-template": template(
    '  <p>{{ "Hello " + name }}</p>',
    "  <p>{{ `Hello ${name}` }}</p>",
  ),
  "vue/quote-props": template(
    "  <Widget :options='{ \"name\": value }' />",
    '  <Widget :options="{ name: value }" />',
    "as-needed",
  ),
  "vue/no-restricted-syntax": {
    ...template("  <p>{{ format(value) }}</p>", "  <p>{{ formattedValue }}</p>", {
      selectors: ["CallExpression"],
    }),
    eslintDifference: {
      en: "The option value may be the ESLint array of selector strings and `{ selector, message }` objects, or `{ selectors: [...] }`. Selectors are a subset of esquery, listed under [rule options](../options.md#vue-no-restricted-syntax-selectors). A selector outside that subset fails configuration. `:nth-child(an+b)` formulas, relative `:has(> …)` selectors, `[attr=type(...)]`, and the `:exit` suffix are outside the subset.",
    },
  },
  "vue/no-literals-in-template": template(
    '  <Widget :options="{ eager: true }" />',
    '  <Widget :options="options" />',
  ),
  "vue/no-restricted-v-bind": template(
    '  <input :v-focus="enabled" />',
    '  <input v-focus="enabled" />',
  ),
  "vue/prefer-v-model": template(
    '  <TextField :model-value="value" @update:model-value="value = $event" />',
    '  <TextField v-model="value" />',
  ),
  "vue/v-if-else-key": template(
    '  <section><Panel v-if="ready" /><Panel v-else /></section>',
    '  <section><Panel v-if="ready" key="ready" /><Panel v-else key="fallback" /></section>',
  ),
  "vue/enforce-style-attribute": sfc(
    "<template><div /></template>\n<style module>.card { color: red }</style>",
    "<template><div /></template>\n<style scoped>.card { color: red }</style>",
    { allow: ["scoped"] },
  ),
  "vue/next-tick-style": sfc(
    '<script setup>\nimport { nextTick } from "vue"\nnextTick(() => focusInput())\n</script>\n<template />',
    '<script setup>\nimport { nextTick } from "vue"\nawait nextTick()\nfocusInput()\n</script>\n<template />',
  ),
  "vue/no-content-outside-block": sfc(
    "unexpected\n<template><div /></template>",
    "<template><div>expected</div></template>",
  ),
  "vue/no-define-model-default": sfc(
    '<script setup>\nconst model = defineModel({ default: "draft" })\n</script>\n<template />',
    '<script setup lang="ts">\nconst model = defineModel<string>()\n</script>\n<template />',
  ),
  "vue/no-props-shadow": sfc(
    '<script setup>\ndefineProps({ value: String })\nconst value = "local"\n</script>\n<template />',
    '<script setup>\ndefineProps({ value: String })\nconst localValue = "local"\n</script>\n<template />',
  ),
  "vue/no-shadow-native-events": sfc(
    "<script setup>\nconst emit = defineEmits(['click'])\n</script>\n<template />",
    "<script setup>\nconst emit = defineEmits(['activate'])\n</script>\n<template />",
  ),
  "vue/prefer-single-event-payload": sfc(
    '<script setup lang="ts">\nconst emit = defineEmits<{ move: [x: number, y: number] }>()\nemit("move", 1, 2)\n</script>\n<template />',
    '<script setup lang="ts">\nconst emit = defineEmits<{ move: [point: { x: number; y: number }] }>()\nemit("move", { x: 1, y: 2 })\n</script>\n<template />',
  ),
  "vue/require-macro-variable-name": sfc(
    "<script setup>\nconst p = defineProps({ value: String })\n</script>\n<template />",
    "<script setup>\nconst props = defineProps({ value: String })\n</script>\n<template />",
  ),
  "vue/sort-keys": sfc(
    "<script>\nexport default { methods: { zebra() {}, alpha() {} } }\n</script>\n<template />",
    "<script>\nexport default { methods: { alpha() {}, zebra() {} } }\n</script>\n<template />",
    { order: "asc", caseSensitive: true, natural: false },
  ),
  "vue/block-lang": sfc(
    '<script setup>\nconst label = "a";\n</script>\n<template>\n  <p>{{ label }}</p>\n</template>',
    '<script setup lang="ts">\nconst label = "a";\n</script>\n<template>\n  <p>{{ label }}</p>\n</template>',
    { script: { lang: "ts", allowNoLang: false } },
  ),
  "vue/require-default-export": sfc(
    '<script lang="ts">\nconst label = "a";\n</script>\n<template>\n  <p>label</p>\n</template>',
    '<script lang="ts">\nconst label = "a";\nexport default { name: "Label" };\n</script>\n<template>\n  <p>label</p>\n</template>',
  ),
  "max-lines": {
    bad: {
      language: "vue",
      source:
        '<script setup lang="ts">\nconst label = "a";\n</script>\n\n<template>\n  <ul>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n  </ul>\n</template>\n',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\nconst label = "a";\n</script>\n<template>\n  <p>{{ label }}</p>\n</template>\n',
    },
    evidence: "crates/vize_patina/src/rules/vue/max_lines.rs",
    ruleOptions: { max: 10 },
    eslintDifferenceHeading: { en: "Differences from ESLint" },
    eslintDifference: {
      en: "The count covers the whole file Vize lints, including `<template>` and `<style>`, so an SFC can be capped in one place. A trailing newline does not add a line. `skipComments` skips a line whose remaining text is only a `//` comment, a `/* */` comment, or an HTML comment, and it does not treat text inside a string as a comment. A comment that shares a line with code stays counted. ESLint takes those comment lines from the parser's comment list.",
    },
  },
  "vue/max-lines-per-block": {
    bad: {
      language: "vue",
      source:
        '<script setup lang="ts">\nconst label = "a";\n</script>\n\n<template>\n  <ul>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n    <li>{{ label }}</li>\n  </ul>\n</template>\n',
    },
    good: {
      language: "vue",
      source:
        '<script setup lang="ts">\nconst label = "a";\n</script>\n<template>\n  <p>{{ label }}</p>\n</template>\n',
    },
    evidence: "crates/vize_patina/src/rules/vue/max_lines_per_block.rs",
    ruleOptions: { template: 10 },
  },
  "vue/require-prop-comment": sfc(
    '<script setup lang="ts">\nconst { size } = defineProps<{ size: string }>();\n</script>\n<template>\n  <p>{{ size }}</p>\n</template>',
    '<script setup lang="ts">\nconst { size } = defineProps<{\n  /** The rendered size. */\n  size: string;\n}>();\n</script>\n<template>\n  <p>{{ size }}</p>\n</template>',
  ),
};
