---
title: Viteプラグイン
---

<!-- Reviewed translation; source: guide/vite-plugin.md; scope: native configuration guidance -->

# Vite プラグイン

> **⚠️ 進行中の作業:** Vize は積極的に開発中であり、まだ運用環境で使用する準備ができていません。重要なプロジェクトに採用する前に、徹底的にテストしてください。

> **バンドラーのステータス:** `@vizejs/vite-plugin` は現在最も安定したバンドラー統合です。
> rollup / webpack / esbuild の場合は `@vizejs/unplugin` を使用し、Rspack の場合は `@vizejs/rspack-plugin` を使用します。
> これらの非 Vite パスはまだ不安定なので、実験的なものとして扱う必要があります。

`@vizejs/vite-plugin` は、Vite プロジェクトにネイティブ速度の Vue SFC コンパイルを提供します。これは、`@vitejs/plugin-vue` の**ドロップイン置換**として設計されており、既存の Vue コンポーネントは変更することなく動作します。

## Vite+ から始める

コンパイル、Vue 型チェック、lint、フォーマットをまとめる場合は、
[Vite+ helper](./vite-plus.md)を使って `vite.config.ts` に設定します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

[移行ガイド](./migration.md)に import と plugin の具体的な差分を示しています。
通常の Vite の導入経路は、コンパイラだけを置き換えます。

<span id="drop-in-scope"></span>

## 置き換えの対応範囲

Vue 3 SFC の `<script setup>`・Options API、scoped style、CSS Modules、SSR、HMR が対象です。
Vue 2・2.7 の互換モードは別の導入経路です。webpack・rollup・esbuild・Rspack の統合は実験段階です。
`@vitejs/plugin-vue` の全オプションとの互換性は完成していません。
移行するオプションは[コンパイラ設定](#compiler-options)と
[#3227](https://github.com/ubugeeei-prod/vize/issues/3227)を確認してください。

## インストール

[Vite+ インストール ガイド](https://viteplus.dev/guide/install) から `vp` を一度インストールし、パッケージを追加します。

```bash
vp install -D @vizejs/vite-plugin
```

プロジェクトが `"vize"` から共有構成ヘルパーをインポートする場合にのみ、`vize` を直接の依存関係として追加します。
または、`vize:lint` や `vize:check` などのパッケージ スクリプトを公開します。

## 基本的な使い方

```javascript
// vite.config.js
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

それでおしまい。 `@vitejs/plugin-vue` を `@vizejs/vite-plugin` に置き換えると、プロジェクトが Rust を通じてコン​​パイルされます。

## TypeScript Vue インポート

プラグイン パッケージを `compilerOptions.types` に追加して、`.vue` インポートを直接解決できるようにします。
ローカル `env.d.ts` シムを書き込まない TypeScript:

```json
{
  "compilerOptions": {
    "types": ["vite/client", "@vizejs/vite-plugin"]
  }
}
```

これには、`vize` をプロジェクトの直接依存関係として追加する必要はありません。

Vite Plus プロジェクトの場合は、Vite Plus クライアント タイプを維持し、プラグイン パッケージを追加します。

```json
{
  "compilerOptions": {
    "types": ["vite-plus/client", "@vizejs/vite-plugin"]
  }
}
```

## ネイティブ設定を共有する

Vite+ では統合用の `compiler`、通常の Vite ではプラグインのオプションを使います。CLI・エディターと共有するネイティブ設定は、同じ Vite 設定のトップレベルの `vize` にまとめます。対応する記法、検索規則、リリース状況は[設定ガイド](./configuration.md)を参照してください。専用設定の形式も必要な場合に利用できます。

<span id="compiler-options"></span>

## コンパイラ オプション

`vize()` に直接渡したオプションが優先され、次にインラインの `config`、検出したプロジェクト設定、既定値の順になります。同じディレクトリでは `vize.config.*` が Vite 設定より優先されます。`config: false` で自動検索を無効にできます。

```ts
vize({
  vueVersion: 3,
  sourceMap: true,
  ssr: false,
  vapor: false,
  customRenderer: false,
  templateSyntax: "standard",
  scanPatterns: ["src/**/*.vue"],
  ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
});
```

| オプション             | どこに設定するか                                            | 説明                                                                                                                                                               |
| ---------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `vueVersion`           | `vize({ vueVersion })`                                      | `0.11`、`1`、`2`、または `"legacy"` を非侵襲的なレガシー Vue 互換モードで実行し、SFC コンパイルをホスト コンパイラーに任せるように設定します。                     |
| `sourceMap`            | `compiler.sourceMap` または `vize({ sourceMap })`           | ソースマップを生成します。デフォルトでは開発がオン、本番がオフになっています。                                                                                     |
| `ssr`                  | `compiler.ssr` または `vize({ ssr })`                       | Vite の SSR ビルド フラグが不十分な場合に SSR を強制的にコンパイルします。                                                                                         |
| `vapor`                | `compiler.vapor` または `vize({ vapor })`                   | Vapor バックエンドを通じてテンプレートをコンパイルします。                                                                                                         |
| `jsxMode`              | `compiler.jsxMode` または `vize({ jsxMode })`               | `.jsx`/`.tsx` コンポーネントのデフォルトの出力バックエンド (`"vdom"` / `"vapor"`)。コンポーネントごとの `"use vue:*"` ディレクティブはこれをオーバーライドします。 |
| `customRenderer`       | `compiler.customRenderer` または `vize({ customRenderer })` | 小文字の非 HTML タグをカスタム レンダラー要素として扱います。 `<TresMesh>` のような PascalCase タグには一致しません。                                                 |
| `customElements`       | `compiler.customElements` または `vize({ customElements })` | カスタム要素としてコンパイルするタグパターン。 TresJS の PascalCase レンダラータグには `["Tres*"]` を使います。                                                     |
| `templateSyntax`       | `compiler.templateSyntax` または `vize({ templateSyntax })` | `"standard"`、`"strict"`、または `"quirks"` テンプレート構文処理を選択します。                                                                                     |
| `whitespace`           | `compiler.whitespace`、`vize({ whitespace })`、または `template.compilerOptions.whitespace` | Vue の `"condense"` (既定) と `"preserve"` を選択します。Vize 固有の `"vue2-line-breaks"` は、テキストや補間の後の改行が Vue 3 でスペースに畳まれる箇所だけ、改行を残す移行モードです。Vue 2 の空白処理全体を再現するものではありません。 |
| `experimentals`        | top-level `experimentals` または `vize({ experimentals })`  | 完全 opt-in の Vue RFC と backend experiments。省略、`false`、`null` は無効です。                                                                                 |
| `include`              | `vite.include` または `vize({ include })`                   | プラグインがコンパイルする必要があるファイル。                                                                                                                     |
| `exclude`              | `vite.exclude` または `vize({ exclude })`                   | プラグインが無視する必要があるファイル。                                                                                                                           |
| `scanPatterns`         | `vite.scanPatterns` または `vize({ scanPatterns })`         | 起動時のプリコンパイルに使用される Glob パターン。                                                                                                                 |
| `ignorePatterns`       | `vite.ignorePatterns` または `vize({ ignorePatterns })`     | 起動時のプリコンパイル中に Glob パターンがスキップされました。                                                                                                     |
| `configMode`           | `vize({ configMode })`                                      | 共有設定の読み込みには、`"root"`、`"auto"`、または `false` を使用します。                                                                                          |
| `configFile`           | `vize({ configFile })`                                      | 特定の構成ファイルをロードします。                                                                                                                                 |
| `config`               | `vize({ config })`                                          | Vite Plus ランタイム設定のインライン共有構成。                                                                                                                     |
| `handleNodeModulesVue` | `vize({ handleNodeModulesVue })`                            | `node_modules` からインポートされた `.vue` ファイルをオンデマンドでコンパイルします。                                                                              |
| `debug`                | `vize({ debug })`                                           | プラグインのデバッグ ログを出力します。                                                                                                                            |

`experimentals` の flag 一覧、opt-in の値、alias、shared-config precedence は
[Experimentals](./experimentals.md) を参照してください。

一般的なレシピ:

```ts
// Vapor-oriented build
vize({ vapor: true });

// TresJS PascalCase レンダラータグ
vize({
  customRenderer: true,
  customElements: ["Tres*", "primitive"],
});

// Existing templates that rely on parser edge cases, such as
// v-for alias edge parens or `<div />` as a self-closing leaf
vize({ templateSyntax: "quirks" });

// Monorepo package with explicit scan roots
vize({
  root: import.meta.dirname,
  scanPatterns: ["src/**/*.vue", "examples/**/*.vue"],
});

// Legacy Vue / Nuxt 2 Bridge project with an existing host compiler plugin
vize({ vueVersion: 2 });
```

`vueVersion: 0.11`、`1`、`2`、および `"legacy"` は、ホスト コンパイラー互換モードです。Vize はそうではありません
これらのモードで `.vue` ファイルをコンパイルし、Vue 3 `vite:vue` API シムを公開せず、
Vue 3 バンドラー機能フラグを挿入します。既存の Vue コンパイラ プラグイン、`vue-loader`、または Nuxt 2 をそのまま使用します。
独自のコンパイラは正常に構成されています。

## 仕組み

プラグインは `.vue` ファイル リクエストをインターセプトし、Node.js NAPI バインディングを通じて Vize の Rust ネイティブ パイプラインを使用してコンパイルします。

1.**プリコンパイル**— `buildStart` で、プラグインはすべての `.vue` ファイルを検出し、`compileBatch` を使用してそれらをバッチでコンパイルします。これにより、Rust 側で Rayon ベースの並列コンパイルがトリガーされ、すべての CPU コアですべてのファイルが同時に処理されます。

2.**オンデマンド コンパイル**— 開発中に、キャッシュにない `.vue` ファイルが要求された場合 (動的インポートなど)、`compileFile` を介してオンザフライでコンパイルされます。

3.**HMR**— `.vue` ファイルが変更されると、そのファイルのみが再コンパイルされます。プラグインは、変更がスタイルのみであるかどうかを検出し、可能な場合はスタイルのみの HMR 更新を適用して、コンポーネント全体の再レンダリングを回避します。

4.**CSS 抽出**— 実稼働ビルドでは、Vue コンポーネントからスコープ指定されたすべての CSS が抽出され、`assets/vize-components.css` にマージされ、コンポーネントごとのスタイル挿入のオーバーヘッドが排除されます。

### コンパイル パイプライン

```
.vue file
  → Armature (Parser)          — Tokenizes and parses the SFC structure
  → Croquis (Semantic Analysis) — Analyzes template expressions and bindings
  → Atelier (Compilation)       — Generates optimized JavaScript output
  → Vitrine (NAPI Binding)      — Delivers the result to Node.js
  → Vite module graph            — Served as a virtual module
```

同じセマンティック分析レイヤーが、lint チェックと型チェックによって再利用されます。参照
[静的分析](./static-analysis.md) パイプラインの診断側用。

## 比較

| 特集                     | @vitejs/plugin-vue | @vizejs/vite-plugin                     |
| ------------------------ | ------------------ | --------------------------------------- |
| 言語                     | JavaScript         | Rust (NAPI)                             |
| SFC コンピレーション     | はい               | はい                                    |
| テンプレートのコンパイル | はい               | はい                                    |
| スクリプトのセットアップ | はい               | はい                                    |
| CSS スコープ             | はい               | はい                                    |
| SSRサポート              | はい               | はい                                    |
| HMR                      | はい               | はい (スタイルのみの最適化)             |
| バッチプリコンパイル     | いいえ             | はい (レーヨン経由の平行)               |
| CSS抽出                  | コンポーネントごと | 結合された単一ファイル                  |
| 蒸気モード               | 実験的             | ファーストクラス (`vize_atelier_vapor`) |

## 高度な機能

### バッチプリコンパイル

最初のリクエストで各 `.vue` ファイルをコンパイルする `@vitejs/plugin-vue` とは異なり、Vize はビルド開始時にマルチスレッド バッチ コンパイルを使用して、検出されたすべての `.vue` ファイルをプリコンパイルします。これはつまり：

- **開発サーバーの起動**- 最初のページを読み込む前にすべてのコンポーネントの準備が完了しています
- **実稼働ビルド**— 最初から最大の並列処理

### 静的アセットの書き換え

プラグインは、テンプレート内の静的アセット URL を自動的に書き換えます。例えば：

```vue
<template>
  <img src="./logo.png" />
</template>
```

`src` 属性は import ステートメントにホイストされ、Vite がアセット パイプライン (ハッシュ、最適化など) を通じてアセットを処理できるようになります。

### 置換の定義

Vite は通常、仮想モジュール (`\0` がプレフィックス付き) の `import.meta.*` 置換をスキップします。 Vize のプラグインは、定義置換を手動で適用して、コンパイルされた Vue コンポーネントで `import.meta.env.*` 値が正しく動作するようにします。

### 環境ごとの分離

Nuxt との互換性を確保するために、プラグインは Vite 環境 (クライアント対サーバー/SSR) ごとに `define` 値を分離します。これにより、クライアント側の環境値が SSR 出力に漏洩するのを防ぎます。

## Nuxt の互換性

このプラグインは、`@vitejs/plugin-vue` の API (Nuxt など) を調査するツールの互換性シムを公開します。これは、Vize が特別な設定を行わなくても、Nuxt の組み込み Vue 統合で動作することを意味します。

```ts
// nuxt.config.ts — using the dedicated Nuxt module
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

詳細については、[Nuxt 統合](../integrations/nuxt.md) を参照してください。

## 注意事項

- プラグインには、Node.js NAPI バインディング用の `@vizejs/native` が必要です (依存関係として自動的にインストールされます)
- Vapor モードのコンパイルは `vize_atelier_vapor` (Vue 3.6+) 経由で利用可能です
- VDOM コンパイルでは `vize_atelier_dom` を使用します
- プラグインは、コンパイルされたすべての CSS をモジュールとしてインポートするための `virtual:vize-styles` をサポートしています
- `.jsx`/`.tsx` Vue コンポーネントは、同じプラグインを通じて自動的にコンパイルされます。[JSX & TSX](./jsx.md) ガイドを参照してください。
- 実験的なロールアップ / webpack / esbuild / Rspack のサポートについては、[実験的なバンドラー統合](./unplugin.md) を参照してください。

<span id="単独-cli-lsp-と共有する設定-任意"></span>
