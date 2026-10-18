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

Vite+ のプロジェクトでは、`vite.config.ts` の `fmt.vize` にオプションを書きます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  fmt: {
    vize: {
      printWidth: 100,
      singleQuote: true,
    },
  },
});
```

`fmt.ignorePatterns` は両方のフォーマッタに適用されます。`fmt.vize: false` を指定すると、`.vue` ファイルも Oxfmt に任せます。単独の CLI やエディターが同じ設定をどこから読むかは、[設定](./configuration.md)を参照してください。

同じオプションは CLI のフラグでも指定できます。

| フラグ                             | 説明                                                            |
| ---------------------------------- | --------------------------------------------------------------- |
| `--print-width`                    | 1 行の最大幅を指定します。                                      |
| `--tab-width`                      | インデント 1 段あたりのスペース数を指定します。                 |
| `--use-tabs`                       | スペースの代わりにタブでインデントします。                      |
| `--single-quote`                   | ダブルクォートの代わりにシングルクォートを使います。            |
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

## Prettier から移行する

Vize の出力は Prettier や Oxfmt とほぼ同じですが、いくつかの既定値は意図して変えています。Prettier と同じように属性やブロックを書いた順に残すには、並べ替えを無効にします。

```ts
fmt: {
  vize: {
    sortAttributes: false,
    sortBlocks: false,
  },
},
```

コマンドラインでは、`vize fmt --preset prettier` でこれらの設定に加えて次の動作をまとめて指定できます。

- 行幅を Prettier と同じ 80 にします。Vize の既定値は Oxfmt と同じ 100 です。
- `v-bind:`、`v-on:`、`v-slot:` は書いたとおりに残します。
- CSS、SCSS、Less の文字列をダブルクォートにします。`--single-quote` を付けるとシングルクォートにします。
- 1 行の `style` 属性を、末尾の `;` を付けずに `color: red; top: 0` の形で書きます。
- `v-for` の別名が複数あるときは、`v-for="(item, i) in items"` のように括弧で囲みます。
- 空要素を `<img src="a.png" />` のように閉じます。
- SCSS と Less のブロックを入れ子の深さでインデントし直し、空行を整えます。

同時に指定したフラグはプリセットより優先されます。

Node API でも同じ `vize` と `prettier` の2つを指定できます。

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
