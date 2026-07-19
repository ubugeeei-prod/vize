---
title: はじめる
---

<!-- Generated translation; source: getting-started.md -->

# はじめる

> **⚠️ 進行中の作業:**Vize は積極的に開発中であり、まだ運用環境で使用する準備ができていません。 API とパッケージの境界は予告なく変更される場合があります。

## ヴィゼとは何ですか?

Vize (_/viːz/_) は、Rust で書かれた Vue.js ツールチェーンです。ワークスペースには共有が含まれています
以下の構成要素:

| エリア               | メイン Rust クレート                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | ユーザー向けのエントリ ポイント                    |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------- |
| 編集                 | [`vize_atelier_core`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_core)、[`vize_atelier_dom`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_dom)、[`vize_atelier_vapor`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_vapor)、[`vize_atelier_ssr`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_ssr)、 [`vize_atelier_sfc`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_sfc) | `@vizejs/vite-plugin`、npm `vize:build` スクリプト |
| 糸くず               | [`vize_patina`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_patina)                                                                                                                                                                                                                                                                                                                                                                                                              | npm `vize:lint` スクリプト、`oxlint-plugin-vize`   |
| フォーマット         | [`vize_glyph`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_glyph)                                                                                                                                                                                                                                                                                                                                                                                                                | npm `vize:fmt` スクリプト                          |
| タイプチェック       | [`vize_canon`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_canon)                                                                                                                                                                                                                                                                                                                                                                                                                | npm `vize:check` スクリプト                        |
| エディターのサポート | [`vize_maestro`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_maestro)                                                                                                                                                                                                                                                                                                                                                                                                            | VS コード、ゼッド、ラスト `vize lsp`               |
| 美術館の美術道具     | [`vize_musea`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_musea)                                                                                                                                                                                                                                                                                                                                                                                                                | `@vizejs/vite-plugin-musea`                        |
| バインディング       | [`vize_vitrine`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_vitrine)                                                                                                                                                                                                                                                                                                                                                                                                            | `@vizejs/native`、`@vizejs/wasm`                   |

このガイドでは、JavaScript パッケージ管理とプロジェクト コマンドに [Vite+](https://viteplus.dev/) (`vp`) を推奨します。ワークスペースの基礎となるツールを使用しながら、パッケージ マネージャー間でインストールと実行のフローの一貫性を保ちます。

`vp` をまだ持っていない場合は、一度インストールして新しいシェルを開きます。

```bash
curl -fsSL https://vite.plus | bash
```

詳細については、[Vite+ ドキュメント](https://viteplus.dev/) および [依存関係のインストール ガイド](https://viteplus.dev/guide/install) を参照してください。

## Vize の機能

大まかに言うと、Vize はいくつかの再利用可能なレーンに分割されています。

| レーン                   | パッケージまたはスクリプト               | 得られるもの                                                                                      |
| ------------------------ | ---------------------------------------- | ------------------------------------------------------------------------------------------------- |
| コンパイル               | `@vizejs/vite-plugin`、`vize:build`      | Rust ネイティブ Vue SFC コンパイル、SSR 出力、Vapor モード、スコープ付き CSS 処理                 |
| 静的解析                 | `vize:lint`、`oxlint-plugin-vize`        | Vue テンプレート、スクリプト、CSS、a11y、SSR、Vapor、Musea、クロスファイル、およびタイプ認識診断  |
| タイプチェック           | `vize:check`                             | 仮想 TypeScript の生成、プロジェクト診断、Vue からソースへの診断マッピング                        |
| フォーマット             | `vize:fmt`                               | プロジェクトおよび CLI オプションを使用した Vue SFC フォーマット                                  |
| コンポーネントギャラリー | `@vizejs/vite-plugin-musea`、`musea-vrt` | アート ファイル、コンポーネント バリアント、プレビュー セットアップ、デザイン トークン、a11y、VRT |
| エディターのサポート     | VS コード、ゼッド、ラスト `vize lsp`     | オプトイン診断とエディター機能                                                                    |

lint および型チェック モデルについては、[静的解析](./guide/static-analysis.md) を参照してください。
具体的なルール出力の [ルール](./rules/index.md)、および
[構成](./guide/configuration.md) 共有構成およびコンパイラ オプション用。

`.vue` SFC ではなく JSX/TSX でコンポーネントをオーサリングしますか? [JSX & TSX](./guide/jsx.md) ガイドを参照してください。
`.jsx`/`.tsx` Vue コンポーネントは同じ Rust レーンを通じてコンパイルされます。

## エントリーポイントを選択してください

### 1. Vite プロジェクト

既存の Vite プロジェクトでネイティブ Vue コンパイルが必要な場合は、Vite プラグインを使用します。

```bash
vp install -D @vizejs/vite-plugin
```

共有構成ヘルパーをインポートする場合にのみ、`vize` を直接の依存関係としてインストールします。
`"vize"` または `vize:lint` や `vize:check` などの Vize パッケージ スクリプトを追加します。

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

同じ設定をパッケージ化する場合は、`vize.config.ts` にコンパイラ オプションを追加します。
スクリプトとプラグイン:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

### 2. Nuxt プロジェクト

Nuxt 独自の Vite パイプライン内で Vize を実行する場合は、Nuxt モジュールを使用します。

```bash
vp install @vizejs/nuxt
```

モジュールを `nuxt.config.ts` に追加します。

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

通常どおり Nuxt 開発サーバーを実行します。モジュールは Vue SFC 用に `@vizejs/vite-plugin` を登録します
Nuxt の自動インポート、コンポーネント、ミドルウェア、SSR 変換を保持しながらコンパイルします。

Musea のセットアップと Nuxt 固有の注意事項については、[Nuxt Integration](./integrations/nuxt.md) ガイドを参照してください。

### 3. npm パッケージ スクリプト + 共有設定

共有設定ユーティリティとネイティブ コマンドを次から利用できるようにしたい場合は、`vize` npm パッケージを使用します。
プロジェクトのスクリプト。

```bash
vp install -D vize
```

推奨されるパッケージ スクリプト:

```json
{
  "scripts": {
    "vize:build": "vize build src",
    "vize:fmt": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path src",
    "vize:check": "vize check src",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:fmt
vp run vize:lint
vp run vize:check
vp run vize:build
vp run vize:ready
```

npm パッケージの `vize check` コマンドは、パッケージ化された NAPI チェッカーを使用し、Vue コンポーネントを出力できます
`--declaration --declaration-dir dist/types` で宣言します。必要な場合は Rust CLI を使用してください
`check-server`、LSP、IDE 管理、または Vue、TS、TSX、および `.d.ts` 入力にわたるプロジェクト診断。

### 4. 完全な Rust CLI

ほとんどのアプリケーション ワークフローでは、上記の npm パッケージ スクリプトを使用する必要があります。次の場合には Rust バイナリを使用してください。
今すぐ完全なネイティブ CLI (LSP、IDE 管理、プロファイリング、または `check-server`) が必要です。 v1 アルファの場合、
サポートされているパブリック チャネルは、GitHub リリース バイナリと Nix エントリ ポイントです。 Rust CLIはそうではありません
crates.io を通じてまだ公開されていません。

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

```bash
vize build src/**/*.vue
vize fmt --check src
vize lint --profile src
vize check --profile src
vize ready src
vize lsp
```

## ネイティブ型チェック

`vize check` は `vize_canon` を利用しており、ネイティブ TypeScript 診断用に [`corsa-bind`](https://github.com/ubugeeei/corsa-bind) プロジェクト セッションを利用するようになりました。 Vize は Vue SFC 用の仮想 TypeScript を生成し、Corsa にプロジェクト対応の診断を要求し、その結果を元の `.vue`、`.ts`、`.tsx`、および `.d.ts` ファイルにマッピングし直します。

このパスはまだ成熟しているため、エディターのタイプ チェックは現時点ではオプトイン機能のままです。の
ランタイム スタックは `@typescript/native-preview` パッケージ、Corsa/corsa-bind は API レイヤー Vize
と対話し、TypeScript ネイティブ プレビューによってインストールされる実行可能ファイルには依然として一般的な名前が付けられています。
`tsgo`。 `typeChecker.corsaPath`、または実行されるパッケージ スクリプトを使用します。
`vize check --corsa-path /path/to/tsgo`、そのランタイムを固定したい場合。
`typeChecker.tsgoPath` は、非推奨の互換性エイリアスのままです。

便利なパッケージ スクリプト ターゲット:

```json
{
  "scripts": {
    "vize:check": "vize check",
    "vize:check:app": "vize check --tsconfig tsconfig.app.json",
    "vize:check:virtual-ts": "vize check --show-virtual-ts src/components/App.vue",
    "vize:check:declarations": "vize check --declaration --declaration-dir dist/types"
  }
}
```

```bash
vp run vize:check
vp run vize:check:app
vp run vize:check:virtual-ts
vp run vize:check:declarations
```

## 共有 `vize.config.*`

npm package コマンドと `@vizejs/vite-plugin` は構成検出を共有します。

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

TypeScript 構成:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
  },
  linter: {
    preset: "opinionated",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    corsaPath: "./node_modules/.bin/tsgo",
  },
  formatter: {
    printWidth: 100,
  },
  musea: {
    include: ["src/**/*.art.vue"],
    basePath: "/__musea__",
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
});
```

PKL 構成:

```pkl
amends "node_modules/vize/pkl/vize.pkl"

linter {
  preset = "opinionated"
}

typeChecker {
  enabled = true
  strict = true
}

lsp {
  lint = true
  typecheck = false
  editor = false
  formatting = false
}
```

スキーマを含む JSON 構成:

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "linter": {
    "preset": "opinionated"
  }
}
```

## パッケージ

```bash
vp install -D @vizejs/vite-plugin
vp install @vizejs/native
vp install @vizejs/wasm
vp install @vizejs/unplugin
vp install @vizejs/rspack-plugin @rspack/core
vp install @vizejs/nuxt
vp install @vizejs/vite-plugin-musea
vp install @vizejs/musea-mcp-server
vp install -D oxlint oxlint-plugin-vize
```

注:

- `@vizejs/vite-plugin` は現在推奨されるバンドラー統合です。
- `@vizejs/unplugin` および `@vizejs/rspack-plugin` はまだ実験段階です。
- `@vizejs/native` および `@vizejs/wasm` は Rust バインディングを直接公開します。
- `@vizejs/vite-plugin-musea` は、Musea のギャラリーと開発サーバーのワークフローを提供します。

## Musea コンポーネント ギャラリー

Vue ネイティブ コンポーネントのサンプル、ドキュメント、トークン、VRT、および a11y チェックが必要な場合は、Musea を使用します。

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
    }),
  ],
});
```

Vite 開発サーバーを実行し、`/__musea__` を開きます。アート ファイルについては [Musea](./guide/musea.md) を参照してください。
プレビュー設定、デザイントークン、VRT、生成されたバリアント。

## Oxlint の統合

Oxlint 内で Vize の Vue 診断を実行します。

```bash
vp install -D oxlint oxlint-plugin-vize
```

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn"
  },
  "settings": {
    "vize": {
      "preset": "general-recommended",
      "helpLevel": "short"
    }
  }
}
```

ターミナルファーストで使用する場合は、次のことを優先します。

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

## エディターのサポート

日常の Vue 編集には、今のところ `vuejs/language-tools` を使用し続けてください。
Vize エディターの機能は、段階的なオプトイン用に設計されています。

VS コードの開始点:

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

ゼッドの出発点:

```json
{
  "languages": {
    "Vue": {
      "language_servers": ["vize", "..."]
    }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true
      }
    }
  }
}
```

## 地域開発

ローカルタスクはローカルのままです。 [CIパリティ](./contributing.md#common-checks)は`nix develop .#testbox`を使用します。

```bash
nix develop
vp install --frozen-lockfile
vp check
vp fmt
vp dev
vp build
```
