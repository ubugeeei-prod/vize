---
title: ルールのオプション
---
<!-- Reviewed translation; source: rules/options.md -->

# ルールのオプション
<!-- Type-aware reference: [型ルールのオプション](./options-type.md) -->

ルールの判定方法をプロジェクトに合わせたいときに、このページを使ってください。
Vite+ では `vite.config.ts` の `vue.linter.ruleOptions` に指定し、`vp run lint` で検査します。
単独で使う互換設定では、同じ内容を `linter.ruleOptions` に指定します。

`linter.ruleOptions` には、オプションを受け取るルールの設定をプロジェクトごとに指定します。
定義されていないフィールドを指定するとエラーになります。同じルールに対して、後から一致する設定がある場合は、
そのルールのオプション全体を置き換えます。重大度は `linter.rules` で別に設定し、
`ruleOptions` では有効なルールの判定方法を指定します。以下で明記する場合を除き、オプションだけではルールは有効になりません。

ルールと重大度の指定方法は、[ネイティブ Lint の実行ガイド](../guide/lint-execution.md)で確認できます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "script/no-restricted-globals": "error",
        "vue/html-self-closing": "warn",
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "script/no-restricted-globals": {
          "globals": [
            {
              "name": "process",
              "message": "Read env via a typed helper."
            },
            {
              "name": "alert"
            }
          ]
        },
        "vue/html-self-closing": {
          "html": {
            "void": "always",
            "normal": "never",
            "component": "always"
          },
          "svg": "always",
          "math": "always"
        },
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
          ]
        }
      }
    }
  }
});
```

| ルール | オプションの形式 | 既定値と動作 |
| --- | --- | --- |
| `html/no-empty-palpable-content` | `{ contentDirectives?: string[] }` | 既定は空のリストです。目に見える内容を挿入するディレクティブの名前を、接頭辞を除いて完全一致で指定します。ルールは別に有効にしてください。 |
| `script/no-restricted-globals` | `{ globals?: Array<{ name: string; message?: string }> }` | オプションを省略すると、組み込みの禁止リストにある `process`、`localStorage`、`sessionStorage` を検査します。空でない `globals` を指定すると、組み込みのリストを置き換えます。 |
| `script/no-restricted-members` | `{ members?: Array<{ object: string; property: string; message?: string }> }` | `members` を設定し、ルールを有効にした場合だけ報告します。`message` を省略すると、共通の説明を表示します。 |
| `script/define-macros-order` | `{ order?: string[] }` | 既定は `["defineProps", "defineEmits"]` です。指定したマクロを、import と型宣言の後に、設定した順序で並べるよう検査します。 |
| `vue/valid-v-on` | `{ modifiers?: string[] }` | 既定は空の一覧です。大文字・小文字を区別する独自の名前を組み込みの修飾子に追加します。ルールを有効にしたり、ハンドラーの省略を許可したりする設定ではありません。範囲別の設定は継承した一覧を置き換え、`[]` で解除します。[修飾子の詳しい例](./reference/vue-valid-v-on.md)を参照してください。 |
| `vue/component-name-in-template-casing` | `{ casing?: "PascalCase" \| "kebab-case" }` | 既定は `PascalCase` です。登録済みのコンポーネントだけを対象にし、追加のグローバル名は既定で空です。 |
| `script/custom-event-name-casing` | `{ casing?: "camelCase" \| "kebab-case" }` | 既定は `camelCase` です。 |
| `script/define-props-destructuring` | `{ destructure?: "only-when-assigned" \| "always" \| "never" }` | 既定は `only-when-assigned` です。`always` は代入しない呼び出しも検査し、`never` は props オブジェクトでの参照を要求します。Vue 3.5 の分割代入はリアクティビティを保ちます。 |
| `vue/no-mutating-props` | `{ shallowOnly?: boolean }` | 既定は `false` です。`true` では、プロパティ自体の再代入を禁止したまま、入れ子のプロパティの変更を許可します。 |
| `vue/require-component-registration` | `{ globals?: string[] }` | アプリケーションのプラグインやプレビューのセットアップで登録するコンポーネント名を指定します。PascalCase と kebab-case の表記を許可します。ルールは別に有効にしてください。 |
| `vue/sfc-element-order` | `{ order?: Array<string \| string[]> }` | 既定は `[["script", "template"], "style"]` です。入れ子の配列は、その位置に配列内のどのブロックが現れてもよいことを表します。 |
| `vue/html-self-closing` | `{ html?: { void?: Style; normal?: Style; component?: Style }; svg?: Style; math?: Style }`。`Style` は `"always"`、`"never"`、`"any"` のいずれか | 既定は `html.void: "always"`、`html.normal: "any"`、`html.component: "always"`、`svg: "always"`、`math: "always"` です。 |
| `vue/v-on-event-hyphenation` | `"always" \| "never"` | コンポーネントの静的なイベントリスナー名を、ハイフン区切りにするかを指定します。 |
| `vue/v-on-style` | `"shorthand" \| "longform"` | 既定は `"shorthand"` です。イベントディレクティブを `@click` と `v-on:click` のどちらで書くかを指定します。`v-on="listeners"` のようなオブジェクト形式は変更しません。 |
| `vue/attribute-hyphenation` | `"always" \| "never"` | テンプレートのコンポーネントに渡すプロパティの属性名を、ハイフン区切りにするかを指定します。 |
| `vue/valid-v-slot` | `{ allowModifiers?: boolean }` | 既定は `false`。`true` では、名前付きスロットの修飾子を許可します。引数なしの修飾子や、そのスロット自身のプロパティに依存する動的な名前は報告します。 |
| `musea/prefer-design-tokens` | `{ tokens?: Array<{ path: string; value: string; tier?: string }> }` | トークンのデータを指定し、ルールを有効にするか、空でないトークンのリストで暗黙に選択した場合だけ報告します。`tier` の既定は `primitive` です。 |
| `type/strict-boolean-expressions` | `{ allowString?: boolean; allowNumber?: boolean; allowNullableObject?: boolean; allowNullableBoolean?: boolean; allowNullableString?: boolean; allowNullableNumber?: boolean; allowNullableEnum?: boolean; allowAny?: boolean }` | `allowString`、`allowNumber`、`allowNullableObject` の既定は `true`、その他は `false` です。ルールを明示的に有効にする必要があります。 |

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

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "html/no-empty-palpable-content": "error"
      },
      "ruleOptions": {
        "html/no-empty-palpable-content": {
          "contentDirectives": [
            "safe-html"
          ]
        }
      }
    }
  }
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

<span id="scoped-entries"></span>

## ファイルごとの設定

`entries` がファイルに一致すると、その `linter.ruleOptions` が共通設定より優先されます。
同じルールのオプションは全体を置き換えます。`globals`、`members`、`order`、`tokens`
などの配列は、複数の設定から結合されません。

## `vue/require-component-registration`

アプリケーションのプラグインや Musea `previewSetup` が登録するコンポーネント名を指定します。
PascalCase と kebab-case の表記を許可し、正規表現は解釈しません。オプションだけでは
ルールは有効になりません。ファイルごとの設定はリスト全体を置き換え、空のリストで引き継いだ値を消去します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "vue/require-component-registration": "warn"
      },
      "ruleOptions": {
        "vue/require-component-registration": {
          "globals": [
            "MyButton",
            "MyIcon"
          ]
        }
      }
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
