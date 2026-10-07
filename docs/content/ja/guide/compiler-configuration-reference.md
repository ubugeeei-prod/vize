---
title: コンパイラ設定リファレンス
---

# コンパイラ設定リファレンス

統合ごとの設定場所は[設定ガイド](./configuration.md)を参照してください。
単独設定の検索・スコープ・解析項目は[単独リファレンス](./configuration-reference.md)に記載しています。

## コンパイラ オプション

これらのオプションは `compiler` の下にあります。これらはスキーマでサポートされており、`defineConfig` を通じて共有されます。そうではない
すべての統合はまだすべてのフィールドを消費します。

| オプション          | 値                                          | 共通用途                                                                |
| ------------------- | ------------------------------------------- | ----------------------------------------------------------------------- |
| `sourceMap`         | `boolean`                                   | Vite プラグインでソース マップを有効にする                              |
| `ssr`               | `boolean`                                   | Vite の SSR ビルド フラグに依存しない場合の SSR 用のコンパイル          |
| `vapor`             | `boolean`                                   | Vapor モードのコンパイルを有効にする                                    |
| `jsxMode`           | `"vdom"` または `"vapor"`                   | `.jsx`/`.tsx` コンポーネントのデフォルトの出力バックエンド              |
| `customRenderer`    | `boolean`                                   | 小文字の非 HTML タグをカスタム レンダラー要素として扱う                 |
| `customElements`    | `string[]`                                  | カスタム要素としてコンパイルするタグパターン（TresJS は `Tres*`）       |
| `templateSyntax`    | `"standard"`、`"strict"`、または `"quirks"` | テンプレート構文の警告、エラー、または Vue-quirk 処理を選択します。     |
| `scriptExt`         | `"ts"` または `"js"`                        | npm build コマンドで TS 出力を保存するか、JS にダウンコンパイルします。 |
| `mode`              | `"module"` または `"function"`              | 下位レベルのコンパイラ出力モード                                        |
| `prefixIdentifiers` | `boolean`                                   | テンプレート識別子の先頭に `_ctx` を付けます。                          |
| `hoistStatic`       | `boolean`                                   | 静的ノードのホイスティングを制御する                                    |
| `cacheHandlers`     | `boolean`                                   | イベント ハンドラーのキャッシュを制御する                               |
| `isTs`              | `boolean`                                   | スクリプト ブロックを TypeScript として解析する                         |
| `runtimeModuleName` | `string`                                    | ランタイムインポートモジュールをオーバーライドする                      |
| `runtimeGlobalName` | `string`                                    | 関数/IIFE スタイルの出力のランタイム グローバルをオーバーライドする     |

Vite プロジェクトの場合、直接プラグイン オプションが共有設定をオーバーライドします。

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

Experimental Vue RFC と backend flag は top-level `experimentals` に置きます。flag 一覧、opt-in の値、
alias、優先順位は [Experimentals](./experimentals.md) を参照してください。省略、`false`、`null` は無効です。

## テンプレートの構文

`compiler.templateSyntax` のデフォルトは `"standard"` です。

- `"standard"` は、回復可能な無効な構文を受け入れ、警告を発し、有効な出力に書き換えます。
- `"strict"` は、無効な構文をコンパイル エラーとして報告します。
- `"quirks"` は、追加の警告なしでテンプレート構文の互換性の問題を保持します。

既知のケースは次のとおりです。

- `v-for` のエイリアスに一致しない端括弧が含まれています。 Vue は先頭の `(` または末尾の `)` を削除します
  `value`、`key`、および `index` を分割する前のエイリアスから。標準モードと厳密モードのレポート
  これらのエイリアスは不正な形式ですが、quirk モードは Vue を反映します。
- `<div />` や `<span />` など、自己終了構文で記述された非 void HTML 要素。
  標準モードでは警告が発せられ、空の要素として書き換えられますが、厳密モードではエラーが発生し、互換モードでは保持されます。
  それらは自己閉鎖葉として機能します。

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

Vue のアップストリーム実装:

- [`forAliasRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/utils.ts#L571)
- [`stripParensRE` 中の `parseForExpression`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/parser.ts#L493-L530)

無効な場合の HTML 厳密モードの動作については、[トラブルシューティング](./troubleshooting.md) を参照してください。
自己終了タグ。

## JSX および TSX 出力モード

> 完全なオーサリング API、スコープ付きスタイル、型チェック、エディターのサポート、制限事項については、
> [JSX および TSX ガイド](./jsx.md)。このセクションでは、出力モードの構成キーのみを説明します。

Vize は、`.jsx`/`.tsx` Vue コンポーネントを仮想 DOM またはいずれかにコンパイルします。
[蒸気](https://blog.vuejs.org/posts/vue-vapor)出力。 `compiler.jsxMode` は**グローバルを選択します
明示的にオプトインしないコンポーネントの場合はデフォルト**。デフォルトは `"vdom"` です。

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

`jsxMode` は `compiler.vapor` から独立しています: `vapor` は `.vue` SFC の Vapor を切り替えますが、`jsxMode`
JSX/TSX のデフォルトのバックエンドを制御します。プロジェクトは、JSX をデフォルトで使用しながら、SFC を VDOM 上に維持できます。
蒸気、またはその逆。 Vite プラグインは、`jsxMode` をプラグイン オプションとして直接受け入れます。
共有設定をオーバーライドします。

### コンポーネントごとのディレクティブ

個々のコンポーネントは、`"use strict"` をミラーリングするディレクティブ プロローグでデフォルトをオーバーライドします。

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

各コンポーネントは独立してルーティングされるため、**単一のモジュールで両方のバックエンドを混在させることができます**。

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

コンポーネントの出力モードは次の順序で解決されます。

1. コンポーネントごとの `"use vue:vapor"` / `"use vue:vdom"` ディレクティブ。
2. 設定からの `compiler.jsxMode` のデフォルト (またはプラグインの `jsxMode` オプション)。
3. 組み込みフォールバック、`"vdom"`。

### 診断

`"use vue:"` で始まるが、既知のモードを指定していないディレクティブ (次のようなタイプミス)
`"use vue:vdomx"`) は、サイレントに無視されるのではなくコンパイル エラーとして報告され、2 つの競合する
1 つのコンポーネント内のモード ディレクティブ (`"use vue:vapor"` の後に `"use vue:vdom"`) も同様です。
診断されました。 `"use strict"` などの無関係なプロローグはそのまま残されます。

## Vue の方言

`dialect` は、スタンドアロン HTML ドキュメントの Vue 方言プロファイルを選択します (`.html`/`.htm`)。

```json
{
  "dialect": "petite-vue"
}
```

- `"vue"` は、スタンドアロン HTML ドキュメントをプレーンな Vue-from-CDN ドキュメントとして扱います。
- `"petite-vue"` は、スタンドアロン HTML ドキュメントを
  [プチビュー](https://github.com/vuejs/petite-vue) 方言 (`v-scope`/`v-effect`)
  補完機能と petite-vue 対応 IDE 機能)。

キーが存在しない場合、方言はドキュメントごとに構造的に検出されます: `<script src>`
petite-vue パッケージ、`petite-vue` のインライン ES インポート、または `PetiteVue.createApp` に解決します。
電話する。コメントや散文での petite-vue の言及は方言を切り替えることはなく、単一ファイルで行われます。
コンポーネントは常に標準の Vue 言語を使用します。
