---
title: Vite プラグイン
---

<!-- Reviewed translation; source: guide/vite-plugin.md -->

# Vite プラグイン

> [!WARNING]
> Vize は開発を続けており、本番環境への導入準備はまだ整っていません。重要なプロジェクトで採用する前に、十分に検証してください。

> **バンドラー統合の状況:** 現在、最も安定している統合は `@vizejs/vite-plugin` です。
> rollup / webpack / esbuild には `@vizejs/unplugin`、Rspack には `@vizejs/rspack-plugin` を使います。
> Vite 以外の統合はまだ不安定なため、実験的な機能として扱ってください。

`@vizejs/vite-plugin` は、Vite プロジェクトの Vue SFC をネイティブコードの速度でコンパイルします。
Vue 3 SFC では **`@vitejs/plugin-vue` を置き換える**設計になっており、既存の `<script setup>` と Options API のコンポーネントをそのまま使えます。

## Vite+ から始める

コンパイル、ネイティブの Vue 型チェック、lint、フォーマットをまとめて使う場合は、
[Vite+ 用の設定ヘルパー](./vite-plus.md) で `vite.config.ts` に設定します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

[移行ガイド](./migration.md)に、置き換える import とプラグインの呼び出しを具体的に示しています。
以下の通常の Vite 向け手順では、コンパイラだけを置き換えます。

<span id="drop-in-scope"></span>

## 置き換えの対応範囲

既存プラグインを置き換えられる範囲には、次の制限があります。

- **対応範囲:** Vite でコンパイルする Vue 3 SFC が対象です。`<script setup>` と Options API の両方に加え、scoped style、CSS Modules、SSR、HMR に対応します。
- **開発中:** Vue 2 と 2.7（`vue.version: "2"` / `"2.7"`、または `vize({ vueVersion: 2 })`）の互換モードは既定で無効です。このモードでは Vize は `.vue` ファイルをコンパイルせず、既存の Vue コンパイラ、`vue-loader`、Nuxt 2 が引き続き処理を担当します。[コンパイラ オプション](#compiler-options)を参照してください。
- **保証の対象外:** webpack、rollup、esbuild、Rspack 向けの `@vizejs/unplugin` と `@vizejs/rspack-plugin` は実験段階であり、既存プラグインをそのまま置き換えられる保証はありません。[実験的なバンドラー統合](./unplugin.md)を参照してください。
- **未完成:** `@vitejs/plugin-vue` の全オプションとの互換性はまだありません。現在は `include`、`exclude`、`isProduction` を反映します。その他の公開オプション、挙動が異なる理由、残る課題は [#3227](https://github.com/ubugeeei-prod/vize/issues/3227) で追跡しています。

## インストール

[Vite+ のインストールガイド](https://viteplus.dev/guide/install)に従って `vp` を一度インストールし、パッケージを追加します。

```bash
vp install -D @vizejs/vite-plugin
```

`"vize"` から共有設定の helper を import する場合や、`vize:lint`、`vize:check` などのパッケージスクリプトを用意する場合は、`vize` も直接の依存関係に追加します。

## 基本的な使い方

```javascript
// vite.config.js
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

`@vitejs/plugin-vue` を `@vizejs/vite-plugin` に置き換えると、Rust でコンパイルします。
Vite+ の型チェック、lint、フォーマット、ビルドをまとめる場合は、[Vite+ の `defineConfig()`](./vite-plus.md) を使ってください。

<span id="typescript-vue-インポート"></span>

## TypeScript から Vue ファイルをインポートする

`compilerOptions.types` にプラグインのパッケージを追加すると、ローカルの `env.d.ts` に型宣言を手書きせずに、TypeScript から `.vue` ファイルを直接 import できます。

```json
{
  "compilerOptions": {
    "types": ["vite/client", "@vizejs/vite-plugin"]
  }
}
```

この設定のために、`vize` をプロジェクトの直接の依存関係に追加する必要はありません。

Vite+ のプロジェクトでは、既存の Vite+ のクライアント型を残してプラグインのパッケージを追加します。

```json
{
  "compilerOptions": {
    "types": ["vite-plus/client", "@vizejs/vite-plugin"]
  }
}
```

<span id="単独-cli-lsp-と共有する設定-任意"></span>

## ネイティブ設定を共有する

Vite+ では `@vizejs/vite-plugin/vite-plus` の
`defineConfig(vpConfig).withVue(vueConfig)` で、既存の設定にネイティブ Vue 設定を追加します。
通常の Vite では、同じ設定ファイルのトップレベルの `vue:` に共有設定を置き、
`plugins: [vize()]` を使います。CLI とエディターも同じプロジェクト設定を読みます。

TypeScript の `compilerOptions`、標準の `extends`・`references` は `tsconfig.json` に、
Vue テンプレートの設定は `vueCompilerOptions` に置いてください。
専用の `vize.config.*` とトップレベルの `vize:` は廃止しました。
[設定ガイド](./configuration.md)に移行方法を、
[共有 Vue 設定](./vue-configuration.md)にスコープの扱いをまとめています。

<span id="compiler-options"></span>

## コンパイラ オプション

`vize()` に直接渡したオプションが優先され、次にインラインの `config`、検索で見つかったプロジェクト設定、既定値の順に適用します。
専用の `vize.config.*` は廃止しました。`vite.config.*` の `vue:` に移してください。
`config: false` で設定ファイルの自動検索を無効にできます。

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

| オプション | 設定場所 | 説明 |
| --- | --- | --- |
| `vueVersion` | `vize({ vueVersion })` | `0.11`、`1`、`2`、`"legacy"` を指定すると、既存環境に介入しない旧 Vue の互換モードで動作し、SFC のコンパイルを既存のコンパイラに任せます。 |
| `sourceMap` | `vue.compiler.sourceMap` または `vize({ sourceMap })` | `sources` が元の `.vue` ファイルを指すソースマップを生成します。`build.sourcemap` の指定がない場合、既定では開発時に有効、本番ビルド時に無効です。 |
| `ssr` | `vue.compiler.ssr` または `vize({ ssr })` | Vite の SSR ビルドフラグだけでは足りない場合に、SSR 向けのコンパイルを強制します。 |
| `vapor` | `vue.compiler.vapor` または `vize({ vapor })` | Vapor バックエンドでテンプレートをコンパイルします。 |
| `jsxMode` | `vue.compiler.jsxMode` または `vize({ jsxMode })` | `.jsx` / `.tsx` コンポーネントの既定の出力先を `"vdom"` / `"vapor"` から選びます。コンポーネントごとの `"use vue:*"` ディレクティブが優先します。 |
| `customRenderer` | `vue.compiler.customRenderer` または `vize({ customRenderer })` | 小文字の非 HTML タグをカスタムレンダラーの要素として扱います。`<TresMesh>` などの PascalCase タグには一致しません。 |
| `customElements` | `vue.compiler.customElements` または `vize({ customElements })` | Vue コンポーネントではなくカスタム要素としてコンパイルするタグのパターンです。TresJS の PascalCase レンダラータグには `["Tres*"]` を使います。 |
| `templateSyntax` | `vue.compiler.templateSyntax` または `vize({ templateSyntax })` | テンプレート構文の処理を `"standard"`、`"strict"`、`"quirks"` から選びます。 |
| `whitespace` | `vue.compiler.whitespace`、`vize({ whitespace })`、または `template.compilerOptions.whitespace` | Vue の `"condense"`（既定）と `"preserve"` を選びます。Vize の独自コンパイラ/Vite オプションでは `"vue2-line-breaks"` も指定でき、Vue 3 がテキストや補間の後の改行を空白一つにまとめる箇所で改行を残します。移行用の限定的なモードであり、Vue 2 の空白処理全体を再現するものではありません。Vue 2.7 の `whitespace: "preserve"`（Vue 2、vue-loader 15、Nuxt 2 が使う設定）と同じ出力にするには `"vue2-preserve"` を使います。テキストや補間の前後の空白は書いたとおりに残り、要素の間にある空白だけの部分は空白一つになります。 |
| `template.compilerOptions.comments` | `vize({ template: { compilerOptions: { comments } } })` | テンプレートに書いたコメントは、既定では開発時に残し、本番ビルド時に省略します。`true` / `false` で変更できます。ネイティブ API を直接呼ぶ場合は `templateComments` を使い、既定値は `false` です。 |
| `experimentals` | トップレベルの `experimentals` または `vize({ experimentals })` | Vue RFC とバックエンドの実験的機能を明示的に有効にします。キーの省略、`false`、`null` は無効です。 |
| `include` | `vue.vite.include` または `vize({ include })` | プラグインがコンパイルするファイルを指定します。 |
| `exclude` | `vue.vite.exclude` または `vize({ exclude })` | プラグインが処理しないファイルを指定します。 |
| `scanPatterns` | `vue.vite.scanPatterns` または `vize({ scanPatterns })` | 起動時のプリコンパイルで検索する Glob パターンです。 |
| `ignorePatterns` | `vue.vite.ignorePatterns` または `vize({ ignorePatterns })` | 起動時のプリコンパイルで除外する Glob パターンです。 |
| `configMode` | `vize({ configMode })` | 共有設定の読み込み方法に `"root"`、`"auto"`、`false` を指定します。 |
| `configFile` | `vize({ configFile })` | 読み込む設定ファイルを明示します。 |
| `config` | `vize({ config })` | Vite+ の実行時設定に使うインラインの共有設定です。 |
| `handleNodeModulesVue` | `vize({ handleNodeModulesVue })` | `node_modules` から import した `.vue` ファイルを必要になった時点でコンパイルします。 |
| `debug` | `vize({ debug })` | プラグインのデバッグログを出力します。 |

`experimentals` の全フラグ、有効にする値、別名、共有設定の優先順位は、[Experimentals](./experimentals.md)を参照してください。

よく使う設定例:

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

`vueVersion: 0.11`、`1`、`2`、`"legacy"` は、既存のコンパイラとの互換モードです。
これらのモードでは、Vize は `.vue` ファイルをコンパイルせず、Vue 3 の `vite:vue` API シムも公開せず、Vue 3 向けのバンドラー機能フラグも挿入しません。
既存の Vue コンパイラプラグイン、`vue-loader`、Nuxt 2 のコンパイラは、通常どおり設定を残してください。

## 仕組み

プラグインは `.vue` ファイルへのリクエストを受け取り、Node.js の NAPI バインディングを介して Vize の Rust パイプラインでコンパイルします。

1. **プリコンパイル:** `buildStart` で `.vue` ファイルを検索し、`compileBatch` でまとめてコンパイルします。Rust 側では Rayon による並列処理が動き、複数の CPU コアでファイルを処理します。
2. **必要に応じたコンパイル:** 開発中にキャッシュにない `.vue` ファイルが要求された場合（動的 import など）は、`compileFile` でその場でコンパイルします。
3. **HMR:** `.vue` ファイルの変更時は、そのファイルだけを再コンパイルします。変更が style のみに収まるかを判定し、可能な場合は style だけを更新して、コンポーネント全体の再描画を避けます。
4. **CSS の抽出:** 本番ビルドでは、Vue コンポーネントの scoped CSS を抽出して `assets/vize-components.css` にまとめ、コンポーネントごとにスタイルを挿入する負荷を減らします。

<span id="コンパイル-パイプライン"></span>

### コンパイルのパイプライン

```text
.vue file
  → Parser            — Tokenizes and parses the SFC structure
  → Semantic analysis — Resolves template expressions and bindings
  → Code generation   — Emits optimized JavaScript (VDOM, SSR, or Vapor)
  → Node.js binding   — Hands the result to Vite
  → Vite module graph — Served as a virtual module
```

同じ意味解析の仕組みを、lint と型チェックでも使います。
診断を出す処理については、[静的解析](./static-analysis.md)を参照してください。

## 比較

| 機能 | @vitejs/plugin-vue | @vizejs/vite-plugin |
| --- | --- | --- |
| 実装言語 | JavaScript | Rust（NAPI） |
| SFC のコンパイル | 対応 | 対応 |
| テンプレートのコンパイル | 対応 | 対応 |
| script setup | 対応 | 対応 |
| scoped CSS | 対応 | 対応 |
| SSR | 対応 | 対応 |
| HMR | 対応 | 対応（style だけの更新を最適化） |
| バッチプリコンパイル | 非対応 | 対応（Rayon で並列処理） |
| CSS の抽出 | コンポーネントごと | 一つのファイルにまとめる |
| Vapor | 実験段階 | 対応（実験段階） |
| プラグインのオプション | 公開オプション全体 | `include` / `exclude` / `isProduction`。残る対応は [#3227](https://github.com/ubugeeei-prod/vize/issues/3227) を参照 |

## 高度な機能

### バッチプリコンパイル

`@vitejs/plugin-vue` は、各 `.vue` ファイルを最初のリクエスト時にコンパイルします。
Vize は、ビルド開始時に見つかったすべての `.vue` ファイルを、複数スレッドでまとめてプリコンパイルします。

- **開発サーバーの起動:** 最初のページを読み込む前に、すべてのコンポーネントを用意します。
- **本番ビルド:** 開始時から並列処理を使います。

### 静的アセットの書き換え

プラグインは、テンプレート内の静的アセット URL を自動で書き換えます。

```vue
<template>
  <img src="./logo.png" />
</template>
```

`src` 属性を import 文に変換し、Vite がハッシュの付与や最適化など、通常のアセット処理を行えるようにします。

<span id="置換の定義"></span>

### define による置き換え

Vite は通常、`\0` で始まる仮想モジュールで `import.meta.*` の置き換えを行いません。
Vize のプラグインは `define` の置き換えを明示的に適用し、コンパイルした Vue コンポーネントでも `import.meta.env.*` の値を正しく使えるようにします。

### 環境ごとの分離

Nuxt と連携できるよう、プラグインは Vite の環境ごとに `define` の値を分けます。
クライアントとサーバー/SSR を分離し、クライアント用の値が SSR の出力に入り込むのを防ぎます。

<span id="nuxt-の互換性"></span>

### スタイルプリプロセッサと Pug

`<style lang="scss">`・`sass`・`less`・`stylus` は Vue と同じ順序で処理します。まずプリプロセッサを実行し、その出力に Vize がスコープを付けて `v-bind()` を解決します。`<style module>` のクラス名は、バンドラーの CSS Modules 処理が生成します。`@vitejs/plugin-vue` と同じく、プリプロセッサのパッケージ（`sass` または `sass-embedded`、`less`、`stylus`）をプロジェクトにインストールしてください。`<template lang="pug">` は Vize 自身がコンパイルするため、追加のパッケージは不要です。

| 対象 | scss / sass / less / stylus | pug |
| --- | --- | --- |
| Vite | Vite が前処理し、その結果に Vize がスコープを付ける | ネイティブ |
| `@vizejs/unplugin`（webpack・Rollup・Rolldown・esbuild） | webpack はプロジェクトのローダーを使用。Rollup・Rolldown・esbuild はインストール済みのプリプロセッサを実行 | ネイティブ |
| `@vizejs/rspack-plugin` | プロジェクトのローダー（css-loader またはネイティブ CSS） | ネイティブ |
| `vize build` | インストール済みのプリプロセッサを Node で実行 | ネイティブ |
| `vize fmt` | プリプロセッサのブロックは 1 バイトも変更しない | 整形する |
| `vize lint` | テンプレートのルールを適用。スタイルブロックは CSS として lint しない | テンプレートのルールを適用 |
| LSP | CSS の補完とホバー（インデント記法を含む）、`v-bind()` の定義ジャンプ | タグの補完と定義ジャンプ |

## Nuxt との連携

プラグインは、Nuxt などが `@vitejs/plugin-vue` の API を確認する際に使える互換性シムを公開しています。
この仕組みにより、Nuxt に組み込まれた Vue の処理とも連携できます。

```ts
// nuxt.config.ts — using the dedicated Nuxt module
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

詳しくは、[Nuxt との連携ガイド](../integrations/nuxt.md)を参照してください。

## 注意事項

- Node.js の NAPI バインディングには `@vizejs/native` が必要です。依存関係として自動でインストールされます。
- Vapor モードのコンパイルに対応しています（Vue 3.6 以降、実験段階）。
- コンパイルしたすべての CSS をモジュールとして import するために、`virtual:vize-styles` を使えます。
- `.jsx` / `.tsx` の Vue コンポーネントも同じプラグインで自動でコンパイルします。[JSX と TSX](./jsx.md)を参照してください。
- rollup / webpack / esbuild / Rspack への実験的な対応は、[実験的なバンドラー統合](./unplugin.md)を参照してください。
