const vue = (source) => ({ language: "vue", source });
const script = (source) => vue(`<script setup lang="ts">\n${source}\n</script>`);
const pair = (bad, good, evidence, extra = {}) => ({ bad, good, evidence, ...extra });

export const scopedOverrides = {
  "vue/no-negated-v-if-condition": pair(
    vue('<template>\n<div v-if="!ok">A</div>\n<div v-else>B</div>\n</template>'),
    vue(
      '<template>\n<div v-if="ok">B</div>\n<div v-else>A</div>\n\n<div v-if="!ok">A</div>\n\n<div v-if="a !== b">A</div>\n<div v-else>B</div>\n</template>',
    ),
    "crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs",
  ),
  "vue/component-definition-name-casing": pair(
    vue("<template><p>Content</p></template>"),
    vue("<template><p>Content</p></template>"),
    "crates/vize_patina/src/rules/vue/component_definition_name_casing.rs",
    {
      badFilename: "myComponent.vue",
      goodFilename: "MyComponent.vue",
      note: "The component filename is checked. PascalCase and kebab-case are accepted; mixed casing is reported.",
      noteJa:
        "対象はファイル名です。PascalCase と kebab-case は許可され、混在する形式は検出されます。",
    },
  ),
  "vue/multi-word-component-names": pair(
    vue("<template><p>Item</p></template>"),
    vue("<template><p>Item</p></template>"),
    "crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs",
    {
      badFilename: "Item.vue",
      goodFilename: "TodoItem.vue",
      note: "The filename is the finding. Rename the same component; changing a child tag does not fix it.",
      noteJa:
        "検出対象はファイル名です。同じコンポーネントのファイル名を変更します。子要素のタグ名は対象ではありません。",
    },
  ),
  "script/prefer-computed": pair(
    script(
      'import { ref, watch } from "vue";\nconst count = ref(0);\nconst doubled = ref(0);\nwatch(count, (value) => { doubled.value = value * 2; });',
    ),
    script(
      'import { ref, computed } from "vue";\nconst count = ref(0);\nconst doubled = computed(() => count.value * 2);',
    ),
    "crates/vize_patina/tests/derived_watchers_7901.rs",
    {
      note: "The watcher must only derive the destination. Editable copies and callbacks with other side effects are allowed.",
      noteJa:
        "派生値だけを代入する watcher が対象です。ユーザーが編集するコピーや別の副作用を持つ処理は対象外です。",
    },
  ),
  "type/strict-boolean-expressions": pair(
    script("const count: number | undefined = undefined;\nif (count) console.log(count);"),
    script(
      "const count: number | undefined = undefined;\nif (count !== undefined && count > 0) console.log(count);",
    ),
    "crates/vize_patina/src/linter/native_type_aware/strict_boolean/tests.rs",
    {
      typeAware: true,
      note: "Enable typeAware and this rule explicitly. The default disallows nullable numbers, while non-null numbers are allowed.",
      noteJa:
        "typeAware とルールを明示的に有効にします。既定では null を含み得る数値は許可されず、通常の数値は許可されます。",
    },
  ),
  "type/no-reactivity-loss": pair(
    script(
      'import { reactive } from "vue";\nconst state = reactive({ count: 0 });\nconst count = state.count;',
    ),
    script(
      'import { reactive, toRef } from "vue";\nconst state = reactive({ count: 0 });\nconst count = toRef(state, "count");',
    ),
    "crates/vize_patina/src/linter/native_type_aware/reactivity_loss.rs",
    { typeAware: true },
  ),
  "type/no-floating-promises": pair(
    script("async function save(): Promise<void> {}\nsave();"),
    script("async function save(): Promise<void> {}\nvoid save();"),
    "crates/vize_patina/src/linter/native_type_aware/tests.rs",
    { typeAware: true },
  ),
  "type/no-unsafe-template-binding": pair(
    vue(
      '<script setup lang="ts">\nconst value: any = "Hello";\n</script>\n<template><p>{{ value }}</p></template>',
    ),
    vue(
      '<script setup lang="ts">\nconst value: string = "Hello";\n</script>\n<template><p>{{ value }}</p></template>',
    ),
    "crates/vize_patina/src/linter/native_type_aware/unsafe_template_tests.rs",
    { typeAware: true },
  ),
  "type/require-typed-props": pair(
    script('defineProps(["title"]);'),
    script("defineProps<{ title: string }>();"),
    "crates/vize_patina/src/linter/native_type_aware/tests.rs",
    { typeAware: true },
  ),
  "type/require-typed-emits": pair(
    script('defineEmits(["save"]);'),
    script("defineEmits<{ save: [] }>();"),
    "crates/vize_patina/src/linter/native_type_aware/tests.rs",
    { typeAware: true },
  ),
  "script/no-restricted-members": pair(
    script('const token = window.localStorage.getItem("token");'),
    script('const token = authStorage.read("token");'),
    "crates/vize_patina/src/linter/restricted_rules.rs",
    {
      ruleOptions: { members: [{ object: "window", property: "localStorage" }] },
      note: "This example configures window.localStorage. The rule has no default deny list; enabling it alone does not report a member.",
      noteJa:
        "この例では window.localStorage を禁止しています。既定の禁止リストはなく、有効にするだけでは検出されません。",
    },
  ),
  "musea/prefer-design-tokens": pair(
    vue(
      '<art title="Button" component="Button">\n<variant name="Primary"><Button /></variant>\n</art>\n<style scoped>\n.button {\n  color: #3b82f6;\n}\n</style>',
    ),
    vue(
      '<art title="Button" component="Button">\n<variant name="Primary"><Button /></variant>\n</art>\n<style scoped>\n.button {\n  color: var(--color-primary);\n}\n</style>',
    ),
    "crates/vize_patina/src/linter/musea_config.rs",
    {
      filename: "Button.art.vue",
      ruleOptions: { tokens: [{ path: "color.primary", value: "#3b82f6", tier: "semantic" }] },
      note: "Requires an .art.vue file and the token inventory shown below. It does not infer a token from an arbitrary color.",
      noteJa:
        ".art.vue ファイルと下記の token 一覧が必要です。任意の色から token を推測するルールではありません。",
    },
  ),
  "vue/max-template-complexity": pair(
    vue(
      `<script setup lang="ts">\ndefineProps<{ rows: Row[] }>();\n</script>\n<template>\n  <section>\n    <h1>{{ user ? user.name : 'Guest' }}</h1>\n    <DataTable :rows="rows">\n      <template #cell="{ row, column }">\n        <span v-if="column.key === 'status'" :class="row.active ? 'on' : 'off'">{{ row.status ?? 'unknown' }}</span>\n        <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>\n        <template v-else>\n          <em v-for="tag in row.tags" :key="tag">\n            <b v-if="tag.pinned || tag.starred">{{ tag.hot ? '!' : '' }}</b>\n          </em>\n        </template>\n      </template>\n    </DataTable>\n    <p v-if="!rows.length && !loading">No data</p>\n  </section>\n</template>`,
    ),
    vue('<template>\n  <RowList v-if="ready" :rows="rows" />\n</template>'),
    "crates/vize_patina/src/rules/facts/max_template_complexity/tests.rs",
    {
      note: "Bad has cyclomatic complexity 13 and cognitive complexity 25 (limits: 11 and 16). Each component is measured separately; only inline HTML templates are supported.",
      noteJa:
        "悪い例は cyclomatic complexity 13、cognitive complexity 25 で、閾値 11 と 16 を超えます。コンポーネント単位で計測し、対象はインライン HTML テンプレートです。",
    },
  ),
};
