---
title: コンパイラ設定リファレンス
---

<!-- Reviewed translation; source: guide/compiler-configuration-reference.md -->

# コンパイラ設定リファレンス

共通の設定ファイルの場所は[設定ガイド](./configuration.md)を参照してください。
単独ファイルの検索、スコープ、解析の設定は[単独リファレンス](./configuration-reference.md)に記載しています。

<span id="コンパイラ-オプション"></span>

## コンパイラオプション

以下は `defineConfig` の `compiler` に指定するオプションです。統合によっては、すべての項目を利用するわけではありません。

| オプション          | 値                                   | 主な用途                                                                         |
| ------------------- | ------------------------------------ | -------------------------------------------------------------------------------- |
| `sourceMap`         | `boolean`                            | Vite プラグインでソースマップを有効にする                                        |
| `ssr`               | `boolean`                            | Vite の SSR ビルドフラグに依存せず、SSR 用にコンパイルする                       |
| `vapor`             | `boolean`                            | Vapor モードのコンパイルを有効にする                                             |
| `jsxMode`           | `"vdom"` または `"vapor"`            | `.jsx`/`.tsx` コンポーネントの既定の出力バックエンドを選ぶ                       |
| `customRenderer`    | `boolean`                            | 小文字の非 HTML タグをカスタムレンダラーの要素として扱う                         |
| `customElements`    | `string[]`                           | カスタム要素としてコンパイルするタグのパターンを指定する（TresJS では `Tres*`）  |
| `templateSyntax`    | `"standard"`、`"strict"`、`"quirks"` | テンプレート構文に対して、警告、エラー、Vue の互換動作のいずれを使うか選ぶ       |
| `scriptExt`         | `"ts"` または `"js"`                 | npm の build コマンドで TypeScript 出力を保持するか、JavaScript に変換するか選ぶ |
| `mode`              | `"module"` または `"function"`       | 低レベルのコンパイラ出力モードを選ぶ                                             |
| `prefixIdentifiers` | `boolean`                            | テンプレート内の識別子に `_ctx` を付ける                                         |
| `hoistStatic`       | `boolean`                            | 静的ノードのホイスティングを制御する                                             |
| `cacheHandlers`     | `boolean`                            | イベントハンドラーのキャッシュを制御する                                         |
| `isTs`              | `boolean`                            | スクリプトブロックを TypeScript として解析する                                   |
| `runtimeModuleName` | `string`                             | ランタイムのインポート元モジュールを上書きする                                   |
| `runtimeGlobalName` | `string`                             | 関数形式または IIFE 形式の出力で使うランタイムのグローバル名を上書きする         |

Vite プロジェクトでは、プラグインに直接指定したオプションが共通設定より優先されます。

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      vapor: true,
      sourceMap: true,
      customRenderer: true,
      templateSyntax: "standard",
    }),
  ],
});
```

実験的な Vue RFC とバックエンドのフラグは、最上位の `experimentals` に指定します。
[実験機能](./experimentals.md)を参照してください。キーを省略した場合と、`false` または `null` を指定した場合は無効です。

## テンプレートの構文

`compiler.templateSyntax` の既定値は `"standard"` です。

- `"standard"` は修復できる不正な構文を受け入れ、警告を出したうえで有効な出力に書き換えます。
- `"strict"` は不正な構文をコンパイルエラーとして報告します。
- `"quirks"` は追加の警告を出さず、Vue が許容するテンプレート構文の互換動作を維持します。

対象となる既知のケースは次のとおりです。

- `v-for` のエイリアスの先頭または末尾に、対応する括弧のない `(` または `)` がある場合。
  Vue は `value`、`key`、`index` に分割する前に、その括弧を取り除きます。
  standard と strict は不正なエイリアスとして報告し、quirks は Vue と同じ動作をします。
- `<div />` や `<span />` のように、終了タグが必要な HTML 要素を自己終了構文で書いた場合。
  standard は警告を出して空の要素に書き換え、strict はエラーにします。
  quirks は自己終了する葉ノードとして保持します。

```text
<template>
  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="(item in items">{{ item }}</div>

  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="item) in items">{{ item }}</div>

  <!-- Standard warns and rewrites this as `<div></div>`. Strict errors. Quirk keeps it as a leaf. -->
  <div />
</template>
```

Vue 本体の実装:

- [`forAliasRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/utils.ts#L571)
- [`parseForExpression` の `stripParensRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/parser.ts#L493-L530)

不正な自己終了タグに対する HTML の strict モードの動作は、[トラブルシューティング](./troubleshooting.md)を参照してください。

## JSX および TSX 出力モード

> コンポーネントの記述 API、スコープ付きスタイル、型チェック、エディター対応、制限事項は、
> [JSX および TSX ガイド](./jsx.md)を参照してください。この節では、出力モードの設定項目を説明します。

Vize は `.jsx`/`.tsx` の Vue コンポーネントを Virtual DOM または [Vapor](https://blog.vuejs.org/posts/vue-vapor) にコンパイルします。
`compiler.jsxMode` は、コンポーネント自身が出力モードを指定していない場合の**共通の既定値**を選びます。
既定値は `"vdom"` です。

```ts
// vize.config.ts
import { defineConfig } from "@vizejs/vite-plugin";

export default defineConfig({
  compiler: {
    // Default every .jsx/.tsx component to Vapor output.
    jsxMode: "vapor",
  },
});
```

`jsxMode` と `compiler.vapor` は独立した設定です。`vapor` は `.vue` SFC の Vapor 出力を切り替え、
`jsxMode` は JSX/TSX の既定のバックエンドを選びます。
SFC は VDOM のまま、JSX の既定値を Vapor にすることも、その逆も可能です。
Vite プラグインに `jsxMode` を直接指定すると、共通設定を上書きできます。

### コンポーネントごとのディレクティブ

`"use strict"` と同じように、コンポーネントの関数本体の先頭にディレクティブを書くと、既定値を上書きできます。

```tsx
// Compiled to Vapor regardless of the configured default.
const Fast = () => {
  "use vue:vapor";
  return <div class="fast" />;
};

// Compiled to Virtual DOM regardless of the configured default.
const Classic = () => {
  "use vue:vdom";
  return <div class="classic" />;
};
```

各コンポーネントの出力先は個別に決まるため、**同じモジュール内で両方のバックエンドを使えます**。

```tsx
// vize.config: { compiler: { jsxMode: "vapor" } }

// No directive -> takes the configured default (Vapor here).
export const Dashboard = () => <main>{/* ... */}</main>;

// Opts back into Virtual DOM just for this component.
export const LegacyWidget = () => {
  "use vue:vdom";
  return <aside>{/* ... */}</aside>;
};
```

### 優先順位

コンポーネントの出力モードは、次の順に決まります。

1. コンポーネントごとの `"use vue:vapor"` / `"use vue:vdom"` ディレクティブ。
2. 共通設定の `compiler.jsxMode`、またはプラグインの `jsxMode` オプション。
3. 組み込みの既定値である `"vdom"`。

### 診断

`"use vue:"` で始まるディレクティブで、`"use vue:vdomx"` のように未対応のモードを指定すると、
コンパイルエラーとして報告されます。
同じコンポーネントに `"use vue:vapor"` と `"use vue:vdom"` のような競合するディレクティブがある場合もエラーです。
`"use strict"` などの無関係なディレクティブはそのまま保持されます。

## Vue の方言

`dialect` は単独の HTML ドキュメント（`.html`/`.htm`）で使う Vue の方言を選びます。

```json
{
  "dialect": "petite-vue"
}
```

- `"vue"` は単独の HTML ドキュメントを、CDN から Vue を読み込む通常の Vue ドキュメントとして扱います。
- `"petite-vue"` は [petite-vue](https://github.com/vuejs/petite-vue) の方言を選び、
  `v-scope` / `v-effect` の補完と petite-vue に対応した IDE 機能を有効にします。

キーを省略した場合は、ドキュメントの構造から自動判定します。
petite-vue パッケージを指す `<script src>`、`petite-vue` のインライン ES インポート、
または `PetiteVue.createApp` の呼び出しが判定の対象です。
コメントや本文に petite-vue と書くだけでは方言は切り替わりません。SFC は常に標準の Vue の方言を使います。
