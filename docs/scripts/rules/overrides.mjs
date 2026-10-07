import { verifiedOverrides } from "./verified-examples.mjs";
import { scopedOverrides } from "./scoped-examples.mjs";

const vue = (source) => ({ language: "vue", source });
const template = (source) => vue(`<template>\n${source}\n</template>`);
const script = (source, setup = true) =>
  vue(`<script${setup ? " setup" : ""} lang="ts">\n${source}\n</script>`);
const pair = (bad, good, evidence, extra = {}) => ({ bad, good, evidence, ...extra });

export const overrides = {
  ...scopedOverrides,
  ...verifiedOverrides,
  "css/prefer-nested-selectors": pair(
    vue("<style scoped>\n.card .title { color: red; }\n</style>"),
    vue("<style scoped>\n.card { .title { color: red; } }\n</style>"),
    "crates/vize_patina/src/rules/css/prefer_nested_selectors.rs",
  ),
  "vue/no-invalid-html-attribute": pair(
    template('<a href="/guide" rel="stylesheet">Guide</a>'),
    template('<a href="/guide" rel="help">Guide</a>'),
    "crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs",
  ),
  "vue/no-deprecated-inline-template": pair(
    template("<Card inline-template><p>Details</p></Card>"),
    template("<Card><p>Details</p></Card>"),
    "crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs",
  ),
  "vue/no-static-inline-styles": pair(
    template('<p style="color: red">Notice</p>'),
    vue(
      '<template><p class="notice">Notice</p></template>\n<style scoped>.notice { color: red; }</style>',
    ),
    "crates/vize_patina/src/rules/vue/no_static_inline_styles.rs",
  ),
  "vue/no-undefined-refs": pair(
    vue('<script setup>const message = "Hello";</script>\n<template>{{ missing }}</template>'),
    vue('<script setup>const message = "Hello";</script>\n<template>{{ message }}</template>'),
    "crates/vize_patina/src/rules/vue/no_undefined_refs.rs",
  ),
  "vue/no-use-v-else-with-v-for": pair(
    template(
      '<p v-if="ready">Ready</p>\n<p v-else v-for="item in items" :key="item.id">{{ item.name }}</p>',
    ),
    template(
      '<p v-if="ready">Ready</p>\n<template v-else><p v-for="item in items" :key="item.id">{{ item.name }}</p></template>',
    ),
    "crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs",
  ),
  "vue/no-multiple-template-root": pair(
    template("<p>First</p>\n<p>Second</p>"),
    template("<section><p>First</p><p>Second</p></section>"),
    "crates/vize_patina/src/rules/vue/no_multiple_template_root.rs",
    {
      note: "Enable only for a single-root contract. Vue 3 normally supports fragments.",
      noteJa: "単一ルートを要求する場合に有効にします。通常の Vue 3 は複数ルートを許可します。",
    },
  ),
  "script/return-in-computed-property": pair(
    script('import { computed } from "vue";\nconst total = computed(() => { 1 + 2; });'),
    script('import { computed } from "vue";\nconst total = computed(() => { return 1 + 2; });'),
    "crates/vize_patina/src/rules/script/return_in_computed_property.rs",
  ),
  "script/no-ref-as-operand": pair(
    script('import { ref } from "vue";\nconst count = ref(0);\nconst next = count + 1;'),
    script('import { ref } from "vue";\nconst count = ref(0);\nconst next = count.value + 1;'),
    "crates/vize_patina/src/rules/script/no_ref_as_operand.rs",
  ),
  "script/no-reserved-keys": pair(
    script('export default { data() { return { $el: "custom" }; } };', false),
    script('export default { data() { return { elementLabel: "custom" }; } };', false),
    "crates/vize_patina/src/rules/script/no_reserved_keys.rs",
  ),
  "script/no-use-computed-property-like-method": pair(
    script(
      "export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total()); } } };",
      false,
    ),
    script(
      "export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total); } } };",
      false,
    ),
    "crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs",
  ),
  "nuxt/no-page-meta-runtime-values": pair(
    script("definePageMeta({ title: useRoute() });"),
    script("definePageMeta({ validate: () => Boolean(useRoute().params.id) });"),
    "crates/vize_patina/src/rules/script/no_page_meta_runtime_values/tests.rs",
  ),
  "script/no-potential-component-option-typo": pair(
    script("export default { methdos: { save() {} } };", false),
    script("export default { methods: { save() {} } };", false),
    "crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs",
  ),
  "script/no-deprecated-destroyed-lifecycle": pair(
    script("export default { beforeDestroy() { clearTimeout(this.timer); } };", false),
    script("export default { beforeUnmount() { clearTimeout(this.timer); } };", false),
    "crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs",
  ),
  "script/no-unstable-nested-components": pair(
    script(
      'import { defineComponent } from "vue";\nexport default { setup() { const Child = defineComponent({ render() { return null; } }); return { Child }; } };',
      false,
    ),
    script(
      'import { defineComponent } from "vue";\nconst Child = defineComponent({ render() { return null; } });\nexport default { setup() { return { Child }; } };',
      false,
    ),
    "crates/vize_patina/src/rules/script/no_unstable_nested_components.rs",
  ),
  "script/no-export-in-script-setup": pair(
    script("export const count = 1;"),
    script("const count = 1;"),
    "crates/vize_patina/src/rules/script/no_export_in_script_setup.rs",
  ),
  "nuxt/nuxt-config-keys-order": pair(
    script("export default defineNuxtConfig({ ssr: true, modules: [] });", false),
    script("export default defineNuxtConfig({ modules: [], ssr: true });", false),
    "crates/vize_patina/src/rules/script/nuxt_config_keys_order/tests.rs",
    { filename: "nuxt.config.ts", standaloneScript: true },
  ),
  "nuxt/prefer-import-meta": pair(
    script('if (process.client) console.log("browser");'),
    script('if (import.meta.client) console.log("browser");'),
    "crates/vize_patina/src/rules/script/prefer_import_meta.rs",
  ),
  "nuxt/no-nuxt-config-test-key": pair(
    script("export default defineNuxtConfig({ test: true });", false),
    script("export default defineNuxtConfig({});", false),
    "crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs",
    { filename: "nuxt.config.ts", standaloneScript: true },
  ),
  "script/require-explicit-emits": pair(
    script('const emit = defineEmits([]);\nemit("save");'),
    script('const emit = defineEmits(["save"]);\nemit("save");'),
    "crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs",
  ),
  "script/no-required-prop-with-default": pair(
    script(
      'export default { props: { title: { type: String, required: true, default: "Untitled" } } };',
      false,
    ),
    script('export default { props: { title: { type: String, default: "Untitled" } } };', false),
    "crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs",
  ),
  "vue/no-unused-setup-bindings": pair(
    vue('<script setup>const message = "Hello";</script>\n<template><p>Welcome</p></template>'),
    vue(
      '<script setup>const message = "Hello";</script>\n<template><p>{{ message }}</p></template>',
    ),
    "crates/vize_patina/src/rules/facts/unused_setup_bindings.rs",
  ),
};
