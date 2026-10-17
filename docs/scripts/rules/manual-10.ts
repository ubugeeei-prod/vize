import type { RuleExample } from "./types.ts";
// Authored examples retained from the previous category reference.
export const manual10: Record<string, RuleExample> = {
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
  "vue/no-deprecated-v-on-number-modifiers": {
    bad: {
      language: "vue",
      source:
        '<template>\n<input @keyup.13="submit" />\n<input v-on:keyup.27="cancel" />\n<input @keyup.13.stop="submit" />\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n<input @keyup.enter="submit" />\n<input @keyup.esc="cancel" />\n</template>',
    },
    note: '## Numeric selection and fixes\n\nThe first decimal-prefix numeric modifier determines the finding. `@keyup.13.27="submit"` reports only `13` and offers `@keyup.enter.27="submit"`; another fix pass can replace `27` with `escape`. `@keyup.13suffix="submit"` also becomes `@keyup.enter="submit"`. Values 0–9 are exempt, so `@keyup.0.13="submit"` is clean under this rule. Unknown values such as `999` report without a fix. The existing Vue 2 and petite-vue exemption remains in effect.',
    noteJa:
      '## 数値修飾子の選択と修正\n\n最初に十進整数として読み取れる修飾子だけを判定します。`@keyup.13.27="submit"` では\n`13` を報告し、`@keyup.enter.27="submit"` に修正します。次の修正で `27` を `escape` にできます。\n`@keyup.13suffix="submit"` も `@keyup.enter="submit"` に修正します。0〜9 は対象外のため、\n`@keyup.0.13="submit"` はこのルールでは報告しません。`999` など未知のコードは報告だけ行います。\nVue 2 と petite-vue では従来どおり報告しません。',
    evidence: "crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs",
  },
  "vue/valid-v-on": {
    bad: {
      language: "vue",
      source:
        '<template>\n  <div v-on></div>\n  <div @></div>\n  <div @click></div>\n  <div @click.sync="handleClick"></div>\n  <div @click.nativeish="handleClick"></div>\n  <div @click.once></div>\n</template>',
    },
    good: {
      language: "vue",
      source:
        '<template>\n  <div @click="handleClick"></div>\n  <form @submit.prevent></form>\n  <input @keyup.page-down="handlePageDown" />\n  <input @keyup.あ="handleKey" />\n  <div v-on="{ click: handleClick }"></div>\n</template>',
    },
    note: '## Modifier validation\n\nUnsupported modifiers are reported individually at the modifier name, including on long-form `v-on:click`, dynamic event arguments, and object listener syntax. The accepted names match `eslint-plugin-vue` 10.9.2: Vue\'s built-in event modifiers, the complete keyboard alias table, any single Unicode code point, and names with a finite decimal integer prefix (JavaScript `Number.parseInt(name, 10)` semantics). For example, `.13suffix` is accepted by this rule. Numeric-key deprecation and `.native` migration diagnostics belong to separate rules.\n\nOnly `.stop` and `.prevent` provide behavior without a handler. A modifier such as `.once`, `.enter`, or a configured custom name still needs a handler expression.\n\nProjects with custom event handling can add names under `linter.ruleOptions`:\n\n```json\n{\n  "linter": {\n    "rules": { "vue/valid-v-on": "error" },\n    "ruleOptions": {\n      "vue/valid-v-on": { "modifiers": ["custom"] }\n    }\n  }\n}\n```\n\nThis allows `@click.custom="handler"`. Names are exact and case-sensitive; options supplement the built-in names and do not enable a disabled rule. Scoped config entries can replace inherited names with a new list or reset them with `[]`.\n\nThe keyboard table is adapted from the MIT-licensed [eslint-plugin-vue keyboard aliases](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json). Its source and license are preserved alongside the implementation.',
    noteJa:
      '## 修飾子の検証\n\n未対応の修飾子は、修飾子の名前ごとに報告します。長い `v-on:click`、動的なイベント引数、オブジェクト形式も対象です。受け付ける名前は `eslint-plugin-vue` 10.9.2 に合わせ、Vue 組み込みのイベント修飾子、完全なキー別名一覧、単一の Unicode コードポイント、有限な十進整数の接頭辞を持つ名前です。整数は JavaScript の `Number.parseInt(name, 10)` と同じように読み取るため、`.13suffix` もこのルールでは有効です。数値キーの非推奨判定と `.native` の移行診断は別のルールが担当します。\n\nハンドラーなしで動作するのは `.stop` と `.prevent` だけです。`.once`、`.enter`、設定した独自の名前にもハンドラー式が必要です。\n\n独自のイベント処理で使う名前は `linter.ruleOptions` に追加できます。\n\n```json\n{\n  "linter": {\n    "rules": { "vue/valid-v-on": "error" },\n    "ruleOptions": {\n      "vue/valid-v-on": { "modifiers": ["custom"] }\n    }\n  }\n}\n```\n\nこれにより `@click.custom="handler"` を使えます。名前は大文字と小文字を区別して完全一致で比較し、組み込みの名前に追加します。ルール自体が無効な場合は有効にしません。範囲別の設定では継承した一覧を置き換え、`[]` でリセットできます。\n\nキーの一覧は、MIT ライセンスの [eslint-plugin-vue のキー別名](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json) を元にしています。出典とライセンスは実装とともに保持しています。',
    ruleOptions: { modifiers: ["custom"] },
    evidence: "crates/vize_patina/src/rules/vue/valid_v_on.rs",
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
