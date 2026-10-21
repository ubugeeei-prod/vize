---
title: フォーマット
description: Vue コンポーネントを Vize でフォーマットします。Vite+ のプロジェクトでは、その他のファイルを Oxfmt が担当します。
---

<!-- Reviewed translation; source: guide/formatter.md; scope: complete document -->

# フォーマット

Vize は、Vue コンポーネントの `<script>`、`<template>`、`<style>` をまとめてフォーマットします。Vite+ のプロジェクトでは、Vize が `.vue` ファイルを、Oxfmt がそれ以外のファイルを担当するので、コマンドひとつでプロジェクト全体をフォーマットできます。

## プロジェクトをフォーマットする

[Vite+ 統合](./vite-plus.md)を使っている場合は、次のタスクを実行します。

```bash
vp run fmt:check
vp run fmt
```

`vp run fmt:check` は、変更が必要なファイルを報告し、1 つでもあれば失敗します。CI ではこちらを使います。`vp run fmt` は結果をファイルに書き込みます。プロジェクトの一部だけをフォーマットするには、`--` の後ろにパスを指定します。

Vite+ を使わない場合は、[CLI](./cli.md#フォーマット) を直接実行します。対象は Vue、JavaScript、TypeScript、JSX、TSX、JSON、JSONC のファイルです。YAML と Markdown はフォーマットしません。

```bash
vize fmt --check src
vize fmt --write src
```

`--check` は対象のファイルを一覧し、フォーマットが必要なファイルがあると終了コード 1 で終了します。

```text
Found 1 file(s)
Would reformat: src/Card.vue

Checked 1 file(s)
  1 file(s) would be reformatted
```

## 何が変わるか

次のコンポーネントを例にします。

```vue
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ title: string; count?: number }>();
const label = computed(() => `${props.title} (${props.count ?? 0})`);
</script>

<template>
  <article class="card" v-bind:aria-label="label">
    <h2>{{ label }}</h2>
    <slot />
  </article>
</template>

<style scoped>
.card {
  padding: 1rem;
  border: 1px solid #ddd;
}
</style>
```

`vize fmt --write` を実行すると、次のようになります。

```vue
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ title: string; count?: number }>();
const label = computed(() => `${props.title} (${props.count ?? 0})`);
</script>

<template>
  <article class="card" :aria-label="label">
    <h2>{{ label }}</h2>
    <slot />
  </article>
</template>

<style scoped>
.card {
  padding: 1rem;
  border: 1px solid #ddd;
}
</style>
```

スクリプトは、Oxfmt と同じ Oxc のフォーマッタで整形されます。テンプレートはインデントし直され、属性の間の空白がそろい、`v-bind:`、`v-on:`、`v-slot:` は省略記法に置き換わります。`<pre>`、`<textarea>`、`v-pre` の中の空白は、書いたとおりに残ります。

## スタイルを設定する

Vite+ のプロジェクトでは、`vite.config.ts` の `vue.formatter` にオプションを書きます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: {
    printWidth: 100,
    singleQuote: true,
  },
});
```

`fmt.ignorePatterns` は両方のフォーマッタに適用されます。`.vue` ファイルも Oxfmt に任せるには、`vue.formatter` を省略し、`fmt.vue: false` を指定します。明示した `vue.formatter` はタスクの制御設定より優先されます。単独の CLI やエディターが同じ設定をどこから読むかは、[設定](./configuration.md)を参照してください。

同じオプションは CLI のフラグでも指定できます。

| フラグ                             | 説明                                                            |
| ---------------------------------- | --------------------------------------------------------------- |
| `--print-width`                    | 1 行の最大幅を指定します。                                      |
| `--tab-width`                      | インデント 1 段あたりのスペース数を指定します。                 |
| `--use-tabs`                       | スペースの代わりにタブでインデントします。                      |
| `--single-quote`                   | ダブルクォートの代わりにシングルクォートを使います。            |
| `--trailing-comma`                 | 末尾のカンマを `all`、`es5`、`none` から選びます。               |
| `--arrow-parens`                   | 1 引数のアロー関数の括弧を `always`、`avoid` から選びます。      |
| `--bracket-spacing`                | オブジェクトリテラルの波括弧内の空白を切り替えます。            |
| `--bracket-same-line`              | 複数行タグの `>` を最後の属性行に置きます。                      |
| `--no-semi`                        | 文末のセミコロンを省略します。                                  |
| `--sort-attributes`                | テンプレートとブロックタグの属性を並べ替えます。                |
| `--single-attribute-per-line`      | 属性を 1 行に 1 つずつ書きます。                                |
| `--max-attributes-per-line`        | 指定した数を超えると、属性を折り返します。                      |
| `--normalize-directive-shorthands` | `v-bind:`、`v-on:`、`v-slot:` を `:`、`@`、`#` に置き換えます。 |

個々の設定は、次のガイドで詳しく説明しています。

- [Vue スクリプトの import を並べ替える](./formatter-import-sorting.md)
- [インデント幅を設定する](./formatter-indent-width.md)
- [フォーマッタの改行コードを設定する](./formatter-line-endings.md)
- [プロパティの引用符を設定する](./formatter-property-quotes.md)
- [JSX 属性の引用符を設定する](./formatter-jsx-quotes.md)

## オプション

次のオプションはすべて `vue.formatter` で指定できます。指定しなかったオプションは `preset` の値を使い、`preset` もなければここに示す既定値を使います。

| オプション                     | 既定値           | 説明                                                                                                                                                |
| ------------------------------ | ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `preset`                       | なし             | 名前の付いた既定値の組を適用します。`"vize"`、`"prettier"`、`"compact"`、`"documentation"` のいずれかです。一緒に指定したオプションが優先されます。 |
| `printWidth`                   | `100`            | 1 行の最大幅です。                                                                                                                                  |
| `tabWidth`                     | `2`              | インデント 1 段あたりのスペース数です。                                                                                                             |
| `useTabs`                      | `false`          | タブでインデントします。                                                                                                                            |
| `semi`                         | `true`           | 文末にセミコロンを付けます。                                                                                                                        |
| `singleQuote`                  | `false`          | スクリプトでシングルクォートを使います。                                                                                                            |
| `jsxSingleQuote`               | `false`          | JSX の属性でシングルクォートを使います。                                                                                                            |
| `trailingComma`                | `"all"`          | 末尾のカンマを付ける場所です。`"all"`、`"es5"`、`"none"` のいずれかです。                                                                           |
| `bracketSpacing`               | `true`           | オブジェクトリテラルの波括弧の内側にスペースを入れます。                                                                                            |
| `bracketSameLine`              | `false`          | 複数行のタグの `>` を最後の行の末尾に置きます。                                                                                                     |
| `arrowParens`                  | `"always"`       | 引数が 1 つのアロー関数を括弧で囲むかどうかです。`"always"` または `"avoid"` です。                                                                 |
| `endOfLine`                    | `"lf"`           | 改行コードです。`"lf"`、`"crlf"`、`"cr"`、`"auto"` のいずれかです。[改行コード](./formatter-line-endings.md)を参照してください。                    |
| `quoteProps`                   | `"as-needed"`    | オブジェクトのキーを引用符で囲む条件です。[プロパティの引用符](./formatter-property-quotes.md)を参照してください。                                  |
| `singleAttributePerLine`       | `false`          | 属性を 1 行に 1 つずつ書きます。                                                                                                                    |
| `maxAttributesPerLine`         | なし             | タグの属性がこの数を超えると折り返します。                                                                                                          |
| `vueIndentScriptAndStyle`      | `false`          | `<script>` と `<style>` の中身をインデントします。                                                                                                  |
| `sortAttributes`               | `true`           | テンプレートとブロックタグの属性を並べ替えます。                                                                                                    |
| `attributeSortOrder`           | `"alphabetical"` | 同じグループ内の属性の順序です。`"alphabetical"` または `"as-written"` です。                                                                       |
| `mergeBindAndNonBindAttrs`     | `false`          | 静的な属性を先にせず、`class` と `:class` をまとめて並べ替えます。                                                                                  |
| `attributeGroups`              | なし             | Vue スタイルガイドの順序の代わりに使う属性グループです。たとえば `[["id"], ["v-*"], ["*"]]` です。                                                  |
| `normalizeDirectiveShorthands` | `true`           | `v-bind:`、`v-on:`、`v-slot:` を `:`、`@`、`#` に置き換えます。                                                                                     |
| `sortBlocks`                   | `true`           | ブロックを script、script setup、template、style、カスタムブロックの順に並べます。                                                                  |
| `normalizeCssQuotes`           | `false`          | CSS、SCSS、Less の文字列の引用符を `singleQuote` の設定に合わせます。                                                                               |
| `normalizeInlineStyles`        | `false`          | 1 行の静的な `style` 属性を `color: red; top: 0` の形で書きます。                                                                                   |
| `normalizeVForParens`          | `false`          | `v-for` の別名が複数あるときは `(item, i) in items` のように括弧で囲みます。                                                                        |
| `selfCloseVoidElements`        | `false`          | 空要素を `<img />` のように閉じます。                                                                                                               |
| `reindentPreprocessorStyles`   | `false`          | SCSS と Less のブロックを入れ子の深さでインデントし直し、空行を整えます。                                                                           |
| `sortImports`                  | 無効             | スクリプトの import を並べ替えます。[import の並べ替え](./formatter-import-sorting.md)を参照してください。                                          |

Node の native `formatSfc` API でも、Prettier 移行用の 5 つの真偽値
`normalizeCssQuotes`、`normalizeInlineStyles`、`normalizeVForParens`、
`selfCloseVoidElements`、`reindentPreprocessorStyles` を同じ名前で指定できます。
明示した値は `false` を含めて `preset` より優先されます。

スクリプトの引用符ポリシーも設定、CLI の直接上書き、Node で共有されます。
`--jsx-single-quote[=false]` は `jsxSingleQuote`、
`--quote-props <as-needed|consistent|preserve>` は `quoteProps` に対応します。

末尾のカンマのポリシーも同じように共有されます。
`--trailing-comma <all|es5|none>` は、型付きの Node・設定オプション
`trailingComma` に対応します。CLI と Node の明示値は、選択したプリセットより優先されます。

アロー関数の引数の括弧も同じように公開されます。
`--arrow-parens <always|avoid>` は、型付きの Node・設定オプション
`arrowParens` に対応し、明示値はプリセットより優先されます。

波括弧とタグの閉じ括弧の配置も同じように公開されます。
`--bracket-spacing[=false]` と `--bracket-same-line[=false]` は、型付きの
Node・設定オプション `bracketSpacing` と `bracketSameLine` に対応します。
明示した `true` または `false` は、選択したプリセットより優先されます。

## コンパクトなアプリケーションコード

アプリケーションのテンプレートを密に保ちつつ、ブロックや属性を書いた順序を変えたくない場合は `compact` プリセットを使います。

```ts
export default defineConfig({}).withVue({
  formatter: {
    preset: "compact",
  },
});
```

このプリセットは `printWidth: 120` と `bracketSameLine: true` を設定し、属性と SFC ブロックの順序を保ちます。`v-bind:id` を `:id` にする省略記法の正規化は有効です。既存オプションに名前を付けた組み合わせだけなので、一緒に書いたオプションが優先されます。CLI では `vize fmt --preset compact` を使います。

## ドキュメントの例

文章中の例を狭く読みやすく保つには、`documentation` プリセットを使います。

```ts
export default defineConfig({}).withVue({
  formatter: {
    preset: "documentation",
  },
});
```

このプリセットは `printWidth: 80` と `singleAttributePerLine: true` を設定し、属性と SFC ブロックを書いた順に保ち、省略記法は正規化します。CSS、インライン style、`v-for` の別名、空要素、プリプロセッサのレイアウトに対する Prettier 移行用の正規化は有効にしません。CLI では `vize fmt --preset documentation` を使います。

## Prettier から移行する

Vize の出力は Prettier や Oxfmt とほぼ同じですが、いくつかの既定値は意図して変えています。Vue ファイルを Prettier と同じように整形するには、`prettier` プリセットを指定します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: {
    preset: "prettier",
  },
});
```

このプリセットは属性とブロックを書いた順に残し（`sortAttributes: false`、`sortBlocks: false`）、さらに次の動作をまとめて指定します。

- 行幅を Prettier と同じ 80 にします。Vize の既定値は Oxfmt と同じ 100 です。
- `v-bind:`、`v-on:`、`v-slot:` は書いたとおりに残します。
- CSS、SCSS、Less の文字列をダブルクォートにします。`--single-quote` を付けるとシングルクォートにします。
- 1 行の `style` 属性を、末尾の `;` を付けずに `color: red; top: 0` の形で書きます。
- `v-for` の別名が複数あるときは、`v-for="(item, i) in items"` のように括弧で囲みます。
- 空要素を `<img src="a.png" />` のように閉じます。
- SCSS と Less のブロックを入れ子の深さでインデントし直し、空行を整えます。

コマンドラインでは、`vize fmt --preset prettier` で同じプリセットを指定できます。オプションは、設定ファイルのプリセット、設定ファイルのその他のオプション、`--preset`、その他のフラグの順に適用されます。

Node API でも同じ `vize`、`prettier`、`compact`、`documentation` の4つを指定できます。

```ts
import { formatSfc } from "@vizejs/native";

const result = formatSfc(source, {
  preset: "prettier",
  singleQuote: true,
});
```

明示したオプションは `false` を含めてプリセットより優先されます。`preset` を省略すると従来の Vize の既定値を使い、未対応の名前を指定するとエラーになります。

`sortAttributes: false` のときは、`<script>`、`<template>`、`<style>`、カスタムブロックのタグの属性も書いた順のまま残ります。既定値では `<script lang="ts" setup>` が `<script setup lang="ts">` になります。

次の違いはプリセットを使っても残ります。

- 要素内のテキストは改行位置を変えません。Prettier は長いテキストを行幅まで詰め直します。
- `v-for` 以外のテンプレートの式は書いた形のまま残ります。たとえば `{ open: boolean, close: () => void }` のようなスロットの型の `,` 区切りはそのままです。Prettier は `;` に置き換えます。
- SCSS と Less のブロックの値は書いたとおりに残します。プリセットが変えるのは引用符、インデント、空行だけです。Prettier は `.2s` を `0.2s` にするなど、値も整形します。インデント構文の Sass と Stylus のブロックは書いたとおりに残し、Pug のテンプレートは外側のインデントだけを調整します。

## 保存時にフォーマットする

[エディターの設定](./vite-plus-editor.md)を行うと、Vue ファイルのフォーマッタとして Vize が選ばれます。その他のファイルには、これまでのフォーマッタがそのまま使われます。
