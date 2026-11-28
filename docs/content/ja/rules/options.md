---
title: ルールのオプション
---

<!-- Reviewed translation; source: rules/options.md -->

# ルールのオプション

<!-- Type-aware reference: [型ルールのオプション](./options-type.md) -->

ルールの判定方法をプロジェクトに合わせたいときに、このページを使ってください。
Vite+ では `vite.config.ts` の `withVue` に渡す `linter.ruleOptions` に指定し、`vp run lint` で検査します。
単独で使う互換設定では、同じ内容を `linter.ruleOptions` に指定します。

`linter.ruleOptions` には、オプションを受け取るルールの設定をプロジェクトごとに指定します。
定義されていないフィールドを指定するとエラーになります。同じルールに対して、後から一致する設定がある場合は、
そのルールのオプション全体を置き換えます。重大度は `linter.rules` で別に設定し、
`ruleOptions` では有効なルールの判定方法を指定します。以下で明記する場合を除き、オプションだけではルールは有効になりません。

ルールと重大度の指定方法は、[ネイティブ Lint の実行ガイド](../guide/lint-execution.md)で確認できます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: {
      "script/no-restricted-globals": "error",
      "vue/html-self-closing": "warn",
      "musea/prefer-design-tokens": "warn",
    },
    ruleOptions: {
      "script/no-restricted-globals": {
        globals: [
          {
            name: "process",
            message: "Read env via a typed helper.",
          },
          {
            name: "alert",
          },
        ],
      },
      "vue/html-self-closing": {
        html: {
          void: "always",
          normal: "never",
          component: "always",
        },
        svg: "always",
        math: "always",
      },
      "musea/prefer-design-tokens": {
        tokens: [
          {
            path: "color.primary",
            value: "#3b82f6",
            tier: "semantic",
          },
        ],
      },
    },
  },
});
```

| ルール | オプションの形式 | 既定値と動作 |
| --- | --- | --- |
| `html/no-empty-palpable-content` | `{ contentDirectives?: string[] }` | 既定は空のリストです。目に見える内容を挿入するディレクティブの名前を、接頭辞を除いて完全一致で指定します。ルールは別に有効にしてください。 |
| `script/no-restricted-globals` | `{ globals?: Array<{ name: string; message?: string }> }` | オプションを省略すると、組み込みの禁止リストにある `process`、`localStorage`、`sessionStorage` を検査します。空でない `globals` を指定すると、組み込みのリストを置き換えます。 |
| `script/no-restricted-members` | `{ members?: Array<{ object: string; property: string; message?: string }> }` | `members` を設定し、ルールを有効にした場合だけ報告します。`message` を省略すると、共通の説明を表示します。 |
| `script/define-macros-order` | `{ order?: string[]; defineExposeLast?: boolean }` | 既定は `["defineProps", "defineEmits"]` と `false` です。指定したマクロを import と型宣言の後に並べます。`defineExposeLast: true` では最後の直接の `defineExpose` を `<script setup>` の最終文にし、`order` から除外します。この指摘は手動で修正します。 |
| `vue/valid-v-on` | `{ modifiers?: string[] }` | 既定は空の一覧です。大文字・小文字を区別する独自の名前を組み込みの修飾子に追加します。ルールを有効にしたり、ハンドラーの省略を許可したりする設定ではありません。範囲別の設定は継承した一覧を置き換え、`[]` で解除します。[修飾子の詳しい例](./reference/vue-valid-v-on.md)を参照してください。 |
| `vue/component-name-in-template-casing` | `{ casing?: "PascalCase" \| "kebab-case", registeredComponentsOnly?: boolean, globals?: string[], ignores?: string[] }` | 既定は `PascalCase` です。登録済みのコンポーネントだけを対象にし、追加のグローバル名と除外する名前は既定で空です。`ignores` に指定したタグは、登録の有無にかかわらず検査しません。書かれたタグ名そのもの、または Rust の正規表現構文を使う `/pattern/flags` を指定します。 |
| `script/custom-event-name-casing` | `{ casing?: "lowercase" \| "camelCase" \| "kebab-case" }` | 既定は `camelCase` です。 |
| `script/consistent-type-imports` | `{ prefer?: "type-imports" \| "no-type-imports"; fixStyle?: "separate-type-imports" \| "inline-type-imports"; disallowTypeAnnotations?: boolean; emitDecoratorMetadata?: boolean }` | 既定は `type-imports`、`separate-type-imports`、`true`、`true` です。最初の 3 つは typescript-eslint のオプションです。`emitDecoratorMetadata` には、プロジェクトがデコレーターのメタデータを出力するかどうかを指定します（lint は `tsconfig.json` を読みません）。`true` の場合、デコレーター付きのシグネチャが直列化する型を値としての使用に数えます。 |
| `vue/component-definition-name-casing` | `"PascalCase" \| "kebab-case"` | 既定は eslint-plugin-vue と同じ `"PascalCase"` です。コンポーネント定義の静的な `name` と `app.component()` の第 1 引数を検査します。ファイル名は `vue/component-file-name-casing` が検査します。オプションだけではルールは有効になりません。 |
| `vue/prop-name-casing` | `{ casing?: "camelCase" \| "snake_case" }` | 既定は `camelCase` です。宣言した props 名を検査します。ルールは別に有効にし、公開 API の名前は手動で変更してください。 |
| `vue/slot-name-casing` | `"camelCase" \| "kebab-case" \| "singleword"` | 既定は `camelCase` です。描画される `<slot>` outlet の静的な名前を検査します。提供側の `v-slot` と `#name` 構文はこのルールの対象外です。 |
| `script/define-props-destructuring` | `{ destructure?: "only-when-assigned" \| "always" \| "never" }` | 既定は `always` で、代入した呼び出しと型付きの単独呼び出しを検査します。`only-when-assigned` は単独呼び出しを除外し、`never` は props オブジェクトでの参照を要求します。Vue 3.5 の分割代入はリアクティビティを保ちます。 |
| `vue/no-mutating-props` | `{ shallowOnly?: boolean }` | 既定は `false` です。`true` では、プロパティ自体の再代入を禁止したまま、入れ子のプロパティの変更を許可します。 |
| `vue/multi-word-component-names` | `{ ignores?: string[] }` | 組み込みの `App` と Vue コンポーネントの除外に加えるリストで、既定は空です。重複のない名前を完全一致で指定し、PascalCase の項目は kebab-case 表記も許可します。 |
| `vue/no-use-v-if-with-v-for` | `{ allowUsingIterationVar?: boolean }` | 既定は `false` です。`true` では、同じ要素の `v-if` が `v-for` の反復変数を参照する場合だけ許可し、無関係な条件は引き続き報告します。 |
| `vue/enforce-style-attribute` | `{ allow?: Array<"plain" \| "scoped" \| "module"> }` | 既定は `["scoped"]` です。許可する style ブロックのモードを指定します。 |
| `vue/no-restricted-syntax` | `Array<string \| { selector: string, message?: string }>` または `{ selectors?: Array<string \| { selector: string, message?: string }> }` | 既定は `[]` です。eslint-plugin-vue 形式で、vue-eslint-parser のテンプレート AST と各式の TS-ESTree ノードに対する esquery セレクターを、任意のメッセージとともに指定します。[対応するセレクターのサブセット](#vue-no-restricted-syntax-selectors)を参照してください。範囲外のセレクターは設定エラーになります。 |
| `vue/no-restricted-html-elements` | `string \| Array<string \| { element: string \| string[]; message?: string }>` | 設定するまで何も検査しません。指定した名前のネイティブ要素を報告します。エントリの `message` は既定の `Unexpected use of forbidden element <name>.` を置き換えます。同じ要素を複数のエントリが指定している場合は、最初のエントリのメッセージを使います。[禁止する要素](#vue-no-restricted-html-elements)を参照してください。 |
| `vue/no-restricted-v-on` | `string \| Array<string \| { argument: string; modifiers?: string[]; element?: string; message?: string }>` | 設定するまで何も検査しません。名前を指定すると、そのイベントのリスナーをすべて禁止します。オブジェクトのエントリは、`modifiers` をすべて持つリスナーだけを対象にし、`element` を指定した場合はその要素に限定します。`message` で独自のメッセージを指定できます。名前はリテラルとして扱います。[禁止するリスナー](#vue-no-restricted-v-on)を参照してください。 |
| `vue/quote-props` | `"as-needed"` | 保持されたテンプレート式にある不要な引用符付きオブジェクトキーを禁止します。 |
| `vue/sort-keys` | `{ order?: "asc" \| "desc"; caseSensitive?: boolean; natural?: boolean }` | 既定は昇順・大文字小文字を区別する辞書順です。対応するコンポーネントオプション内のオブジェクトキーを検査します。 |
| `vue/warn-custom-directive` | `{ ignore?: string[] }` | 既定は空のリストです。`ignore` の正確なディレクティブ名を登録済みとして扱います。 |
| `vue/no-bare-strings-in-template` | `{ allowlist?: string[] }` | 省略時は上流の既定 allowlist を使い、明示したリストはそれを置き換えます。リテラルまたは Rust の正規表現構文を使う `/pattern/flags` を指定でき、無効なパターンはリテラルのまま扱います。 |
| `a11y/anchor-has-content` | `{ components?: string[]; accessibleChildren?: string[]; accessibleDirectives?: string[] }` | どのリストも既定は空です。`components` はリンクとして検査し、`accessibleChildren` は内容として扱います。`accessibleDirectives`（`v-` を除いた名前）のディレクティブが付いた要素は内容を持つとみなします。[内容のオプション](#a11y-content-options)を参照してください。 |
| `a11y/heading-has-content` | `{ components?: string[]; accessibleChildren?: string[]; accessibleDirectives?: string[] }` | `a11y/anchor-has-content` と同じオプションを `<h1>` から `<h6>` に適用します。 |
| `a11y/label-has-for` | `{ controlComponents?: string[] }` | 既定は空のリストです。指定したコンポーネントをラベルのコントロールとして扱います。 |
| `vue/require-component-registration` | `{ globals?: string[]; ignorePatterns?: string[] }` | アプリケーションのプラグインやプレビューのセットアップで登録するコンポーネント名を指定します。PascalCase と kebab-case の表記を許可します。`ignorePatterns` には eslint-plugin-vue の `vue/no-undef-components` と同じく正規表現を指定します。ルールは別に有効にしてください。 |
| `vue/no-undefined-refs` | `{ globals?: string[] }` | 既定は空の一覧です。`unplugin-auto-import` の自動インポートなど、ビルド時にすべてのテンプレートで使えるようになる名前を完全一致で指定します。Nuxt プロジェクトでは、Nuxt が生成した自動インポートの名前を `vize lint` が加えるため、指定は不要です。オプションだけではルールは有効になりません。範囲別の設定は継承した一覧を置き換え、`[]` で解除します。[設定例](./reference/vue-no-undefined-refs.md)を参照してください。 |
| `vue/sfc-element-order` | `{ order?: Array<string \| string[]> }` | 既定は `[["script", "template"], "style"]` です。入れ子の配列は、その位置に配列内のどのブロックが現れてもよいことを表します。 |
| `vue/html-self-closing` | `{ html?: { void?: Style; normal?: Style; component?: Style }; svg?: Style; math?: Style }`。`Style` は `"always"`、`"never"`、`"any"` のいずれか | 既定は eslint-plugin-vue と同じ `html.void: "never"`、`html.normal: "always"`、`html.component: "always"`、`svg: "always"`、`math: "always"` です。 |
| `vue/html-button-has-type` | `{ button?: boolean; submit?: boolean; reset?: boolean }` | 3 つとも既定値は `true` です。`false` にした静的な button type は、値自体が有効でも禁止として報告します。 |
| `vue/html-quotes` | `"double" \| "single"` または `{ style?: "double" \| "single"; avoidEscape?: boolean }` | 既定はダブルクォート、`avoidEscape: false` です。`true` では、指定した引用符を内部に含むクォート済みの値を変更しません。 |
| `vue/mustache-interpolation-spacing` | `"always" \| "never"` | 既定は `"always"` です。`{{` と `}}` の直内側の空白を必須または禁止し、左右を個別に安全修正します。 |
| `vue/v-on-event-hyphenation` | `"always" \| "never" \| { style?: "always" \| "never"; ignore?: string[] }` | 既定は `always` と空の除外リストです。大文字・小文字を区別する部分文字列を含むイベント名を除外します。 |
| `vue/v-on-handler-style` | `Style \| { style?: Style; ignoreIncludesComment?: boolean }`（`Style` は `"method"`、`"inline"`、`"inline-function"`、`["method", "inline"]`、`["method", "inline-function"]`） | 既定は `"method"` で、メソッド参照だけを許可します。ほかの値は eslint-plugin-vue のもので、その既定は `["method", "inline-function"]` です。[ハンドラの書き方](./options-events.md#vue-v-on-handler-style)を参照してください。 |
| `vue/v-bind-style` | `"shorthand" \| "longform"` または `[style, { sameNameShorthand?: "always" \| "never" \| "ignore" }]` | 既定は `"shorthand"` です。バインディングディレクティブを `:class` と `v-bind:class` のどちらで書くかを指定します。longform は `.title` を `v-bind:title.prop` に修正し、明示的な修飾子を保持します。`v-bind="attrs"` のようなオブジェクト形式は変更しません。配列は eslint-plugin-vue の形式です。`sameNameShorthand: "always"` は `:title="title"` を `:title` と書くことを求め、`"never"` は値を書くことを求めます。既定の `"ignore"` はどちらも検査しません。[バインディングの表記](./options-vue.md#vue-v-bind-style)を参照してください。 |
| `vue/v-on-style` | `"shorthand" \| "longform"` | 既定は `"shorthand"` です。イベントディレクティブを `@click` と `v-on:click` のどちらで書くかを指定します。`v-on="listeners"` のようなオブジェクト形式は変更しません。 |
| `vue/v-slot-style` | `"shorthand" \| "longform"` または `{ atComponent?: SlotStyle; default?: SlotStyle; named?: "shorthand" \| "longform" }`（`SlotStyle` は `"v-slot"` も可） | 既定は `atComponent: "v-slot"`、`default: "shorthand"`、`named: "shorthand"` です。名前を消してしまうため、`named` では裸の `v-slot` を受け付けません。 |
| `vue/attribute-hyphenation` | `"always" \| "never"` または `{ style?, ignore? }` | テンプレートのコンポーネントに渡す属性名の形式を指定します。[属性名のオプション](./options-attributes.md)を参照してください。 |
| `vue/valid-v-slot` | `{ allowModifiers?: boolean }` | 既定は `false`。`true` では、名前付きスロットの修飾子を許可します。引数なしの修飾子や、そのスロット自身のプロパティに依存する動的な名前は報告します。 |
| `vue/no-duplicate-attributes` | `{ allowCoexistClass?: boolean; allowCoexistStyle?: boolean }` | どちらも既定は `true` で、eslint-plugin-vue と同じです。`false` にすると `class` と並んだ `:class`、`style` と並んだ `:style` を報告します。[テンプレートのオプション](./options-template.md#vue-no-duplicate-attributes)を参照してください。 |
| `vue/no-multiple-template-root` | `{ disallowComments?: boolean }` | 既定は `false`。`true` ではテンプレートのルートに直接置かれたコメントも報告します。 |
| `vue/no-lone-template` | `{ ignoreAccessible?: boolean }` | 既定は `false`。`true` では `id` か `ref` を持つ入れ子の `<template>` を許可します。 |
| `vue/no-static-inline-styles` | `{ allowBinding?: boolean }` | 既定は `false`。`true` では静的な `style` 属性だけを報告し、`:style` 内の静的な値は報告しません。 |
| `vue/no-unused-components` | `{ ignoreWhenBindingPresent?: boolean }` | 既定は `true` で、動的な `is` のバインディングがあるファイルでは報告しません。`false` では登録を検査します。 |
| `vue/no-v-html` | `{ ignorePattern?: string }` | 既定値はありません。式が Rust の正規表現に一致する `v-html` は報告しません。[テンプレートのオプション](./options-template.md#vue-no-v-html)を参照してください。 |
| `musea/prefer-design-tokens` | `{ tokens?: Array<{ path: string; value: string; tier?: string }> }` | トークンのデータを指定し、ルールを有効にするか、空でないトークンのリストで暗黙に選択した場合だけ報告します。`tier` の既定は `primitive` です。 |
| `type/strict-boolean-expressions` | `{ allowString?: boolean; allowNumber?: boolean; allowNullableObject?: boolean; allowNullableBoolean?: boolean; allowNullableString?: boolean; allowNullableNumber?: boolean; allowNullableEnum?: boolean; allowAny?: boolean }` | `allowString`、`allowNumber`、`allowNullableObject` の既定は `true`、その他は `false` です。ルールを明示的に有効にする必要があります。 |
| `type/no-confusing-void-expression` | `{ ignoreArrowShorthand?: boolean; ignoreVoidOperator?: boolean; ignoreVoidReturningFunctions?: boolean }` | typescript-eslint と同じく、3 つとも既定は `false` です。`ignoreArrowShorthand` で `() => voidCall()` を許可します。 |
| `type/no-unnecessary-condition` | `{ allowConstantLoopConditions?: boolean \| "always" \| "never" \| "only-allowed-literals"; checkTypePredicates?: boolean; allowRuleToRunWithoutStrictNullChecksIKnowWhatIAmDoing?: boolean }` | typescript-eslint と同じく、既定は `"never"` と `false` です。`"only-allowed-literals"` はループ条件の `true`、`false`、`1`、`0` を許可します。`true` は `"always"` と同じです。 |
| `type/restrict-plus-operands` | `{ allowAny?: boolean; allowBoolean?: boolean; allowNullish?: boolean; allowNumberAndString?: boolean; allowRegExp?: boolean; skipCompoundAssignments?: boolean }` | typescript-eslint と同じく、`allow*` の既定は `true`、`skipCompoundAssignments` は `false` です。typescript-eslint の `strict-type-checked` は `allow*` をすべて `false` にします。 |
| `type/restrict-template-expressions` | `{ allowAny?: boolean; allowArray?: boolean; allowBoolean?: boolean; allowNullish?: boolean; allowNumber?: boolean; allowRegExp?: boolean; allowNever?: boolean; allow?: Array<string \| { from: "file" \| "lib" \| "package"; name: string \| string[]; path?: string; package?: string }> }` | typescript-eslint と同じく、`allowAny`、`allowBoolean`、`allowNullish`、`allowNumber`、`allowRegExp` の既定は `true`、`allowArray` と `allowNever` は `false`、`allow` は標準ライブラリの `Error`、`URL`、`URLSearchParams` です。 |
| `type/return-await` | `"in-try-catch" \| "always" \| "error-handling-correctness-only" \| "never"` | typescript-eslint と同じく、既定は `"in-try-catch"` です。`strict-type-checked` が使う `"error-handling-correctness-only"` は、エラー処理の文脈だけを検査します。 |
| `type/await-thenable` | `{ allowMixedPromiseArrays?: boolean }` | typescript-eslint と同じく、既定は `false` です。 |
| `type/no-base-to-string` | `{ checkUnknown?: boolean; ignoredTypeNames?: string[] }` | typescript-eslint と同じく、既定は `false` と `["Error", "RegExp", "URL", "URLSearchParams"]` です。 |
| `type/no-deprecated` | `{ allow?: TypeSpecifier[] }` | typescript-eslint と同じく、既定は `[]` です。各要素は型または値の指定子（`string \| { from: "file" \| "lib" \| "package"; name: string \| string[]; path?: string; package?: string }`）です。 |
| `type/no-duplicate-type-constituents` | `{ ignoreIntersections?: boolean; ignoreUnions?: boolean }` | typescript-eslint と同じく、どちらも既定は `false` です。 |
| `type/no-floating-promises` | `{ allowForKnownSafeCalls?: TypeSpecifier[]; allowForKnownSafePromises?: TypeSpecifier[]; checkThenables?: boolean; ignoreIIFE?: boolean; ignoreVoid?: boolean }` | typescript-eslint と同じく、既定は `[]`、`[]`、`false`、`false`、`true` です。 |
| `type/no-meaningless-void-operator` | `{ checkNever?: boolean }` | typescript-eslint と同じく、既定は `false` です。 |
| `type/no-misused-promises` | `{ checksConditionals?: boolean \| { flagUnions?: "all" \| "strict" \| "none" }; checksSpreads?: boolean; checksVoidReturn?: boolean \| { arguments?, attributes?, inheritedMethods?, properties?, returns?, variables?: boolean } }` | typescript-eslint と同じく、すべての検査の既定は `true` です。`checksVoidReturn` のオブジェクト形式で個別の位置を無効にできます。 |
| `type/no-misused-spread` | `{ allow?: TypeSpecifier[] }` | typescript-eslint と同じく、既定は `[]` です。 |
| `type/no-unnecessary-boolean-literal-compare` | `{ allowComparingNullableBooleansToFalse?: boolean; allowComparingNullableBooleansToTrue?: boolean }` | typescript-eslint と同じく、どちらも既定は `true` です。 |
| `type/no-unnecessary-type-assertion` | `{ checkLiteralConstAssertions?: boolean; typesToIgnore?: string[] }` | typescript-eslint と同じく、既定は `false` と `[]` です。 |
| `type/no-unsafe-member-access` | `{ allowOptionalChaining?: boolean }` | typescript-eslint と同じく、既定は `false` です。 |
| `type/only-throw-error` | `{ allow?: TypeSpecifier[]; allowRethrowing?: boolean; allowThrowingAny?: boolean; allowThrowingUnknown?: boolean }` | typescript-eslint と同じく、既定は `[]`、3 つのスイッチは `true` です。 |
| `type/prefer-promise-reject-errors` | `{ allow?: TypeSpecifier[]; allowEmptyReject?: boolean; allowThrowingAny?: boolean; allowThrowingUnknown?: boolean }` | typescript-eslint と同じく、既定は `[]`、3 つのスイッチは `false` です。 |
| `type/unbound-method` | `{ ignoreStatic?: boolean; checkGlobalObjectMethods?: boolean }` | どちらも既定は `false` です。`ignoreStatic` は typescript-eslint と同じです。グローバルオブジェクトから読み出したライブラリのメソッド (`window.setTimeout`、`self.requestAnimationFrame`) は、レシーバーなしで呼ばれてもプラットフォームの操作がグローバルオブジェクトを使うため報告しません。`checkGlobalObjectMethods: true` で typescript-eslint と同じく報告します。 |

## 詳しい例

- [型ルールのオプション](./options-type.md)：真偽値の条件と null を許す範囲。
- [スクリプトルールのオプション](./options-script.md)：使用を禁止するグローバルとメンバー、カスタムイベント名の形式、props の分割代入。
- [Vue ルールのオプション](./options-vue.md)：コンポーネント名の形式、props を変更できる範囲、SFC ブロックの順序、自己終了タグ、イベントディレクティブ・イベント名・属性名の形式、スロットの修飾子。
- [Musea ルールのオプション](./options-musea.md)：デザイントークンの一覧。

## `html/no-empty-palpable-content`

カスタムディレクティブは、フォーカスの移動、スタイルの適用、内容の挿入などに使われます。
プロジェクトで、目に見える内容を挿入すると確認できるディレクティブだけを指定してください。
`v-` を除いたテンプレートの名前に完全一致します。引数や修飾子は、その名前を変えません。
空のリストを指定すると既定の動作に戻り、ファイルごとの設定はオプション全体を置き換えます。
この設定はサニタイズの安全性を保証せず、ルールを有効にするものでもありません。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: {
      "html/no-empty-palpable-content": "error",
    },
    ruleOptions: {
      "html/no-empty-palpable-content": {
        contentDirectives: ["safe-html"],
      },
    },
  },
});
```

悪い（内容の挿入元を指定していない場合）:

```vue
<p v-sanitize="message" />
```

良い（内容を挿入するディレクティブを指定した場合）:

```vue
<p v-safe-html="message" />
```

<span id="a11y-content-options"></span>

## `a11y/anchor-has-content`

オプションは eslint-plugin-vuejs-accessibility と同じです。`components` にはルーターのリンクなど、
リンクとして検査するコンポーネントを指定します。`accessibleChildren` には常にアクセシブルな内容を
描画するコンポーネントを指定します。`accessibleDirectives` には、付いた要素に内容を挿入する
カスタムディレクティブを `v-` を除いた名前で指定します。要素自身か子孫に付いていれば、値の有無を
問わず内容があるとみなします。値のある `v-text` と `v-html` は常に内容です。PascalCase、camelCase、
kebab-case の表記は同じ名前です。`aria-hidden="true"` で隠した子は、指定していても内容になりません。

```ts
const ruleOptions = {
  "a11y/anchor-has-content": {
    components: ["NuxtLink"],
    accessibleChildren: ["RetryImage"],
    accessibleDirectives: ["markup-inline"],
  },
};
```

報告される：`<NuxtLink to="/docs" />` と `<a href="/retry"><UnknownImage /></a>`
許可される：`<a href="/docs" v-markup-inline="label" />` と `<a href="/retry"><RetryImage /></a>`

## `a11y/heading-has-content`

同じ3つのオプションを `<h1>` から `<h6>` と `<component is="h2">` に適用します。カスタム
ディレクティブで内容を挿入する見出しは空ではありませんが、`accessibleDirectives` がなければ
ルールはそれを判断できません。

```ts
const ruleOptions = {
  "a11y/heading-has-content": {
    components: ["SectionTitle"],
    accessibleDirectives: ["markup-inline"],
  },
};
```

報告される：`<h4 />` と `<SectionTitle />`
許可される：`<h2 v-markup-inline="text" />` と `<h3 v-text="text" />`

<span id="scoped-entries"></span>

## ファイルごとの設定

`entries` がファイルに一致すると、その `linter.ruleOptions` が共通設定より優先されます。
同じルールのオプションは全体を置き換えます。`globals`、`members`、`order`、`tokens`
などの配列は、複数の設定から結合されません。

## `vue/html-button-has-type`

3 つの真偽値で、プロジェクトが許可する静的な button type を制限できます。type がない場合、空の値、
無効な値は常に報告し、値のある動的な `:type` は静的に判定できないため許可します。upstream のルールは
自動修正に対応していないため、この設定も診断のみを行います。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/html-button-has-type": "warn" },
    ruleOptions: {
      "vue/html-button-has-type": {
        button: true,
        submit: false,
        reset: false,
      },
    },
  },
});
```

この設定で報告される例：

```vue
<button type="submit">Save</button>
```

設定に沿った例：

```vue
<button type="button">Cancel</button>
```

## `vue/multi-word-component-names`

プロジェクトの公開用語として意図的に使う一語のコンポーネント名を指定します。このリストは
組み込みの `App` と Vue コンポーネントの除外に追加され、ルール自体は有効にしません。
名前は完全一致で、PascalCase の項目は kebab-case 表記も許可します。upstream のルールは
自動修正に対応していないため、除外していない名前は変更するか、明示的に設定します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/multi-word-component-names": "error" },
    ruleOptions: {
      "vue/multi-word-component-names": { ignores: ["Item"] },
    },
  },
});
```

この設定で報告される例：`Thing.vue`

```vue
<template><p>Thing</p></template>
```

この設定で許可される例：`Item.vue`

```vue
<template><p>Item</p></template>
```

## `vue/component-definition-name-casing`

オプションは eslint-plugin-vue と同じ `"PascalCase"`（既定）または `"kebab-case"` です。
`export default {}` や `defineComponent({})` などのコンポーネント定義の静的な `name`、
`defineOptions({ name })`、`app.component(name, component)` の第 1 引数が対象です。
ファイル名はオプションを持たない `vue/component-file-name-casing` が検査します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/component-definition-name-casing": "warn" },
    ruleOptions: { "vue/component-definition-name-casing": "kebab-case" },
  },
});
```

この設定で報告される例：

```vue
<script setup lang="ts">
defineOptions({ name: "MyComponent" });
</script>
```

設定に沿った例：

```vue
<script setup lang="ts">
defineOptions({ name: "my-component" });
</script>
```

計算で求める名前は検査しません。コンポーネント名を変えると `<KeepAlive include>` や
再帰的な自己参照から見つからなくなるため、ルールは名前を書き換えず、変換後の表記を help に示します。
オプションだけではルールを有効にしません。

## `vue/no-use-v-if-with-v-for`

同じ要素の `v-if` でループ項目を意図的に絞り込むプロジェクトでは、
`allowUsingIterationVar` を `true` にします。`v-for` の反復変数を参照しない条件は
引き続き報告され、このオプションだけではルールを有効にしません。upstream のルールは
自動修正に対応していないため、この設定も診断のみを行います。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-use-v-if-with-v-for": "warn" },
    ruleOptions: {
      "vue/no-use-v-if-with-v-for": {
        allowUsingIterationVar: true,
      },
    },
  },
});
```

この設定で報告される例：

```vue
<li v-for="item in items" v-if="showAll">{{ item.name }}</li>
```

この設定で許可される例：

```vue
<li v-for="item in items" v-if="item.visible">{{ item.name }}</li>
```

## `vue/require-component-registration`

アプリケーションのプラグインや Musea `previewSetup` が登録するコンポーネント名を指定します。
PascalCase と kebab-case の表記を許可し、正規表現は解釈しません。オプションだけでは
ルールは有効になりません。ファイルごとの設定はリスト全体を置き換え、空のリストで引き継いだ値を消去します。

まとめてグローバル登録されるコンポーネントには、eslint-plugin-vue の `vue/no-undef-components` と同じく
`ignorePatterns` に正規表現を指定します。書かれたタグ名、その kebab-case 形、PascalCase 形のいずれかに
一致したタグを報告しません。`ignorePatterns: ["^C[A-Z]"]` は `<CButton>` と `<c-toast>` を除外し、`<Card>` は
報告します。パターンは Rust の `regex` 構文で、コンパイルできないパターンはどのタグにも一致しません。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: {
      "vue/require-component-registration": "warn",
    },
    ruleOptions: {
      "vue/require-component-registration": {
        globals: ["MyButton", "MyIcon"],
      },
    },
  },
});
```

悪い（未登録のコンポーネント）:

```vue
<MissingWidget />
```

良い（グローバル名を指定したコンポーネント）:

```vue
<MyButton />
```

## `vue/no-restricted-html-elements`

各エントリには、要素名、または eslint-plugin-vue の `{ element, message }` オブジェクトを指定します。
`element` には 1 つの名前か名前の配列を指定できます。`message` のないエントリは既定のメッセージで報告します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-restricted-html-elements": "error" },
    ruleOptions: {
      "vue/no-restricted-html-elements": [
        "marquee",
        { element: ["a", "button"], message: "Use BaseLink or BaseButton." },
      ],
    },
  },
});
```

悪い（この設定で報告される例）: `<marquee>` は `Unexpected use of forbidden element marquee.`、
`<a>` は `Use BaseLink or BaseButton.` と報告されます。

```vue
<template><marquee>News</marquee><a href="/">Home</a></template>
```

良い（この設定に沿った例）:

```vue
<template><p>News</p><BaseLink to="/">Home</BaseLink></template>
```

## `vue/no-restricted-v-on`

各エントリには、イベント名、または eslint-plugin-vue の `{ argument, modifiers, element, message }`
オブジェクトを指定します。`modifiers` を指定すると、そのすべてを持つリスナーだけを対象にします。
`element` を指定すると、そのタグ名の要素にあるリスナーだけを対象にします。名前はリテラルとして扱い、
`argument` に `null` は指定できません。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-restricted-v-on": "error" },
    ruleOptions: {
      "vue/no-restricted-v-on": [
        { argument: "click", modifiers: ["stop"], message: "Handle the click in the child." },
        { argument: "focus", element: "BaseInput" },
      ],
    },
  },
});
```

悪い（この設定で報告される例）: `@click.stop` はエントリのメッセージで、`<BaseInput>` の `@focus` は
``Using `@focus` is not allowed on this <BaseInput>.`` と報告されます。

```vue
<template><div @click.stop="close"><BaseInput @focus="open" /></div></template>
```

良い（この設定に沿った例）:

```vue
<template><div @click="close"><input @focus="open" /></div></template>
```

## `vue/no-restricted-syntax` のセレクター {#vue-no-restricted-syntax-selectors}

Patina は [esquery](https://github.com/estools/esquery) の次のサブセットに対応します。

- ノード型（大文字・小文字を区別しません）と `*`
- 属性：`[attr]`、`[a.b.c=value]`、`!=`、`<`、`<=`、`>`、`>=`。値には引用符付き文字列、数値、引用符なし文字列、`/regex/flags` を指定できます（`i`、`m`、`s`、`u` フラグ、Rust の正規表現構文）
- フィールド：`.left`、`.a.b`
- 結合子：子孫（空白）、`>`、`~`、`+`、カンマ区切りのリスト
- `:matches()`、`:is()`、`:not()`、`:has()`、`:first-child`、`:last-child`、`:nth-child(n)`、`:nth-last-child(n)`、`:statement`、`:expression`、`:declaration`、`:function`、`:pattern`

先頭に結合子のない `:has()` の引数は、対象ノードの任意の子孫にマッチします。`>`、`~`、`+` で始まる引数は CSS と同様に対象ノードからの相対セレクターです。`VElement:has(> VStartTag > VAttribute[key.name='flex-wrap'])` は自身の開始タグに `flex-wrap` を持つ要素にマッチし、`:has(> A > B)` は子 `B` を持つ子 `A` を対象ノードに要求し、`:has(+ A)` と `:has(~ A)` は対象ノードの後ろの兄弟を調べます。esquery 1.7 は同じ引数を受け付けますが、最後の複合セレクターだけを対象ノードに結び付け（`:has(> A > B)` は自身が `A` で子 `B` を持つ対象ノードにマッチします）、`:has(+ …)` と `:has(~ …)` には何もマッチしません。

`:nth-child(an+b)` 形式、`[attr=type(...)]`、`:exit` 接尾辞には対応しません。テンプレートは vue-eslint-parser と同様に、`VStartTag`、`VAttribute`、`VDirectiveKey`、`VIdentifier`、`VLiteral`、`VExpressionContainer`、`VText` を子に持つ `VElement[name='template']` として扱います。`VText` は要素や補間の間に書かれたテキストを保持し、要素名は小文字に正規化されます（`rawName` は元の表記を保持します）。式は括弧を除いた TS-ESTree で、vue-eslint-parser が使用する `VOnExpression`、`VForExpression`、`VSlotScopeExpression` の内側にあります。
