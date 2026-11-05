import type { CodeExample, RuleExample } from "./types.ts";

const vue = (source: string): CodeExample => ({ language: "vue", source });
const script = (source: string, setup = true) =>
  vue(`<script${setup ? " setup" : ""} lang="ts">\n${source}\n</script>`);

export const catalogueAdditionExamples: Record<string, RuleExample> = {
  "vue/comment-directive": {
    bad: vue("<template>\n  <!-- eslint-disable-next-line -->\n  <div />\n</template>"),
    good: vue("<template>\n  <div />\n</template>"),
    evidence: "crates/vize_patina/src/context/eslint_directive.rs",
    reportUnusedDisableDirectives: true,
    note: "Unused template suppression comments are reported only when `reportUnusedDisableDirectives: true` is enabled. The example removes an unused comment; comments that actually suppress a finding remain useful. Listed rule names count only when Vize executes those rules.",
    noteJa:
      "未使用のテンプレート抑制コメントを報告するには、`reportUnusedDisableDirectives: true` が必要です。この例は不要なコメントを削除します。実際に診断を抑制するコメントは使用できます。ルール名を列挙した場合、Vize がその実行で検査するルールだけを使用判定に含めます。",
  },
  "vue/html-end-tags": {
    bad: vue("<template><ul><li>First</ul></template>"),
    good: vue("<template><ul><li>First</li></ul></template>"),
    evidence: "crates/vize_patina/src/rules/vue/html_end_tags/tests.rs",
    note: "This rule requires explicit end tags where the HTML parser permits omission, such as `li`, `p`, and `td`. Missing mandatory end tags are owned by `parser/template`; void elements do not need an end tag.",
    noteJa:
      "HTML パーサーが省略を許す `li`・`p`・`td` などの終了タグを明示します。必須の終了タグの欠落は `parser/template` が診断します。空要素には終了タグは不要です。",
  },
  "vue/jsx-uses-vars": {
    bad: vue(
      '<script setup lang="jsx">\nimport MyButton from "./MyButton.vue";\nconst view = () => <div />;\n</script>\n<template><div /></template>',
    ),
    good: vue(
      '<script setup lang="jsx">\nimport MyButton from "./MyButton.vue";\nconst view = () => <MyButton />;\n</script>\n<template><div /></template>',
    ),
    evidence: "crates/vize_patina/src/rules/script/jsx_uses_vars.rs",
    companionRules: { "vue/no-unused-components": "error" },
    badDiagnostic: "vue/no-unused-components",
    note: "This helper does not emit a `vue/jsx-uses-vars` diagnostic. It credits JSX tag bindings when `vue/no-unused-components` decides whether a component is used. Enable both rules for this example. `<MyButton.Part />` credits the root `MyButton`; namespaced tags and `this` members do not supply an imported component binding.",
    noteJa:
      "この補助ルール自体は `vue/jsx-uses-vars` の診断を出しません。`vue/no-unused-components` が使用の有無を判定するとき、JSX のタグに使ったバインディングを使用済みとして扱います。この例では両方のルールを有効にします。`<MyButton.Part />` はルートの `MyButton` を使用済みとします。名前空間付きのタグや `this` のメンバーは、import したコンポーネントの使用として扱いません。",
  },
  "vue/match-component-file-name": {
    bad: script('defineOptions({ name: "OtherPanel" });'),
    good: script('defineOptions({ name: "MyPanel" });'),
    filename: "MyPanel.vue",
    evidence: "crates/vize_patina/src/rules/vue/match_component_file_name.rs",
    note: "The filename is part of this check. The default comparison converts a declared component name to PascalCase or kebab-case and compares it with the unchanged file stem. Files containing several component definitions are left alone.",
    noteJa:
      "この検査にはファイル名も使います。宣言したコンポーネント名を PascalCase または kebab-case に変換し、拡張子を除いた元のファイル名と比較します。コンポーネント定義が複数あるファイルは検査しません。",
  },
  "vue/no-custom-modifiers-on-v-model": {
    bad: vue('<template><MyField v-model.capitalize="value" /></template>'),
    good: vue('<template><MyField v-model="value" /></template>'),
    evidence: "crates/vize_patina/src/rules/vue/no_custom_modifiers_on_v_model.rs",
    note: "This opt-in rule enforces a Vue 2 compatibility contract on component `v-model`. Only `lazy`, `number`, and `trim` are allowed modifiers. Vue 3 components can implement custom modifiers, so Vue 3 presets do not enable this restriction. Native elements are outside this rule's check.",
    noteJa:
      "コンポーネントの `v-model` に Vue 2 の互換性制約を適用する、個別に有効化するルールです。修飾子は `lazy`・`number`・`trim` のみ許可します。Vue 3 のコンポーネントは独自の修飾子を実装できるため、Vue 3 のプリセットでは有効になりません。ネイティブ要素はこの検査の対象外です。",
  },
  "vue/no-ref-object-reactivity-loss": {
    bad: script('import { ref } from "vue";\nconst count = ref(0);\nconst snapshot = count.value;'),
    good: script(
      'import { computed, ref } from "vue";\nconst count = ref(0);\nconst snapshot = computed(() => count.value);',
    ),
    evidence: "crates/vize_patina/src/rules/script/no_ref_object_reactivity_loss/tests.rs",
    note: "The check follows recognized ref factories and same-scope value copies. Reads inside another function and writes to `.value` are left alone. It is a source-based opt-in check, without a native type-query requirement.",
    noteJa:
      "認識できる ref の生成元から、同じスコープで値をコピーする箇所を検査します。別の関数内での読み取りや `.value` への書き込みは対象外です。ソースを解析する、個別に有効化する検査で、ネイティブの型問い合わせは不要です。",
  },
  "vue/no-setup-props-reactivity-loss": {
    bad: script("const props = defineProps({ count: Number });\nconst snapshot = props.count;"),
    good: script(
      'import { computed } from "vue";\nconst props = defineProps({ count: Number });\nconst snapshot = computed(() => props.count);',
    ),
    evidence: "crates/vize_patina/src/rules/script/no_setup_props_reactivity_loss/tests.rs",
    note: "Checks root-scope prop copies in Options API `setup` and `<script setup>`. Vue 3.5 reactive props destructuring itself is retained; copying a destructured prop later at the script root is reported. Reads inside another function are allowed.",
    noteJa:
      "Options API の `setup` と `<script setup>` のルートスコープで props の値をコピーする箇所を検査します。Vue 3.5 のリアクティブな props 分割代入そのものは許可し、その後で script のルートへ値をコピーする箇所を報告します。別の関数内での読み取りは許可します。",
  },
  "vue/no-this-in-before-route-enter": {
    bad: script(
      "export default {\n  beforeRouteEnter(to, from, next) {\n    this.load();\n    next();\n  },\n  methods: { load() {} },\n};",
      false,
    ),
    good: script(
      "export default {\n  beforeRouteEnter(to, from, next) {\n    next((vm) => vm.load());\n  },\n  methods: { load() {} },\n};",
      false,
    ),
    evidence: "crates/vize_patina/src/rules/script/no_this_in_before_route_enter/tests.rs",
  },
  "vue/no-v-for-template-key": {
    bad: vue(
      '<template><ul><template v-for="item in items" :key="item.id"><li>{{ item.label }}</li></template></ul></template>',
    ),
    good: vue(
      '<template><ul><template v-for="item in items"><li :key="item.id">{{ item.label }}</li></template></ul></template>',
    ),
    evidence: "crates/vize_patina/src/rules/vue/no_v_for_template_key.rs",
    note: "This is a Vue 2 compatibility rule and stays opt-in. Vue 2 expects the key on a real child; Vue 3 expects it on `<template v-for>` instead. For the Vue 3 convention, use `vue/no-v-for-template-key-on-child`; do not enable both conventions together.",
    noteJa:
      "Vue 2 の互換性を検査する、個別に有効化するルールです。Vue 2 では実体のある子に key を置き、Vue 3 では逆に `<template v-for>` に置きます。Vue 3 の規約には `vue/no-v-for-template-key-on-child` を使い、両方を同時に有効にしないでください。",
  },
  "vue/prefer-prop-type-boolean-first": {
    bad: script("defineProps({ enabled: [String, Boolean] });"),
    good: script("defineProps({ enabled: [Boolean, String] });"),
    evidence: "crates/vize_patina/src/rules/script/props_emits/prefer_prop_type_boolean_first.rs",
    note: "Changing constructor order can change runtime boolean casting. This opt-in check covers runtime `defineProps` and Options API prop type arrays; it does not reorder TypeScript union types.",
    noteJa:
      "コンストラクターの順序を変えると、実行時の真偽属性の変換も変わる場合があります。この個別に有効化する検査は、実行時の `defineProps` と Options API の型コンストラクター配列が対象です。TypeScript の union 型を並べ替えるものではありません。",
  },
  "vue/require-expose": {
    bad: script("export default { methods: { focus() {} } };", false),
    good: script('export default { expose: ["focus"], methods: { focus() {} } };', false),
    evidence: "crates/vize_patina/src/rules/script/require_expose/tests.rs",
    note: "This check concerns Options API component objects. An explicit empty `expose` list is valid. Setup's `expose` call and returning a render function also satisfy the check; mixins are exempt. The script pass does not inspect a sibling `<script setup>` block, and `defineExpose` is not this rule's target.",
    noteJa:
      "Options API のコンポーネントオブジェクトが対象です。空の `expose` 一覧を明示してもかまいません。setup の `expose` 呼び出しや描画関数の返却でも条件を満たします。mixin は対象外です。この script の検査は併記された `<script setup>` ブロックを参照せず、`defineExpose` を対象にするルールでもありません。",
  },
  "vue/valid-model-definition": {
    bad: script('export default { model: { prop: "value", events: "input" } };', false),
    good: script('export default { model: { prop: "value", event: "input" } };', false),
    evidence: "crates/vize_patina/src/rules/script/valid_model_definition.rs",
    note: "This opt-in rule checks Vue 2's `model` option. Only the static keys `prop` and `event` are accepted; their values are not validated. Vue 3 uses the newer component `v-model` contract instead of this option.",
    noteJa:
      "Vue 2 の `model` オプションを検査する、個別に有効化するルールです。静的なキーは `prop` と `event` のみ許可し、それぞれの値は検証しません。Vue 3 ではこのオプションに代わる新しいコンポーネントの `v-model` 規約を使います。",
  },
};
