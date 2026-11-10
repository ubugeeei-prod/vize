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
  "vue/no-restricted-syntax": template(
    "  <p>{{ format(value) }}</p>",
    "  <p>{{ formattedValue }}</p>",
    { selectors: ["CallExpression"] },
  ),
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
  "vue/require-macro-variable-name": sfc(
    "<script setup>\nconst p = defineProps({ value: String })\n</script>\n<template />",
    "<script setup>\nconst props = defineProps({ value: String })\n</script>\n<template />",
  ),
  "vue/sort-keys": sfc(
    "<script>\nexport default { methods: { zebra() {}, alpha() {} } }\n</script>\n<template />",
    "<script>\nexport default { methods: { alpha() {}, zebra() {} } }\n</script>\n<template />",
    { order: "asc", caseSensitive: true, natural: false },
  ),
};
