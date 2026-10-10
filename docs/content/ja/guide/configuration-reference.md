---
title: 共通設定のリファレンス
---

<!-- Reviewed translation; source: guide/configuration-reference.md; scope: introduction and discovery -->

<span id="単独-cli-の設定リファレンス"></span>

# 共通設定のリファレンス

まずは既存の `vite.config.*` と `tsconfig.json` を使ってください。[設定ガイド](./configuration.md)で導入方法を説明しています。このページでは、共通のネイティブ設定と、必要な場合に使える専用設定をまとめます。

## 設定ファイル

最も近いプロジェクトの同じディレクトリに専用設定がない場合は、Vite の設定を読み込みます。CLI の `--config` で明示することもできます。プラグインに直接渡したオプションやエディターで明示した機能の設定が優先されます。専用設定内の相対パスは設定ファイルのディレクトリを基準に解決します。Vite の `typeChecker` 内のパスとスコープの `basePath` は、選択した Vite の `root` が基準です。CLI へ明示した入力は実行ディレクトリを基準にします。パッケージの対象範囲は `vize.entries` と、そのパッケージの TypeScript プロジェクトで指定します。

npm パッケージ コマンドと `@vizejs/vite-plugin` は、このファイルのプロジェクト ルートからこれらのファイルをロードします。
優先順位:

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

Rust CLIは、次のようなコマンドネイティブ設定に対して、同じ構成ファイル名を上記の順序で読み取ります。
`check`、`lint`、`lsp`、および `fmt`。

## TypeScript 設定

```ts
import { defineConfig } from "vize";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  compiler: {
    sourceMap: mode !== "production",
    ssr: isSsrBuild,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: {
    include: [/\.vue$/],
    exclude: [/node_modules/],
    scanPatterns: ["src/**/*.vue"],
    ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
  },
  linter: {
    enabled: command !== "build",
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
  },
  formatter: {
    printWidth: 100,
    singleQuote: false,
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
  musea: {
    include: ["src/**/*.art.vue"],
    basePath: "/__musea__",
  },
}));
```

## Vue タイプの解決

Vize は、公開された `vize` パッケージから Vue のタイプ サーフェスを固定しません: `vize check`、言語
サーバーおよびパッケージのコマンドは、`vue`、`@vue/compiler-sfc`、および関連するアンビエント タイプを解決します。
プロジェクトが分析されるため、Vue 3 のパッチ、マイナー、プレリリースの選択はそのプロジェクトの制御下に残ります。
Vize の構築に使用されたバージョンではありません。予測可能な結果を得るには、サポートされている Vue を宣言してください
ユーザー プロジェクトのバージョン (Vize 内部経由ではない)、`vue`、`@vue/compiler-sfc` を保持し、
Nuxt などの統合をそこに配置し、プロジェクトのルートまたはポイントから `vize check` を実行します
ターゲット パッケージの `typeChecker.tsconfig`。 `typeChecker.corsaPath` はチェッカーを選択する場合にのみ使用してください
バイナリであり、Vue タイプのバージョンを決してオーバーライドしないでください。プロジェクトが複数の Vue 範囲をサポートしている場合は、それぞれをテストします。
独自のパッケージ マトリックスに組み込まれているため、Vize はハードコーディングされたタイプ パスではなく、アクティブな依存関係グラフに従います。

## 試験的なフラット エントリ

Monorepos では、`entries` を使用して、ルートのデフォルトとパッケージ スコープのオーバーライドを記述することができます。プレーンオブジェクト
構成は内部で 1 つのエントリに正規化され、配列のエクスポートは `defineConfig` によって受け入れられます。
ESLint- flat-config スタイルのオーサリング。

```ts
export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  entries: [
    {
      name: "web app",
      basePath: "apps/web",
      files: ["src/**/*.vue"],
      typeChecker: {
        tsconfig: "tsconfig.app.json",
      },
    },
    {
      name: "ui package",
      basePath: "packages/ui",
      files: ["src/**/*.vue"],
      formatter: {
        singleQuote: true,
      },
    },
  ],
});
```

## PKL 構成

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
  vapor = false
  customRenderer = false
  templateSyntax = "standard"
}

vite {
  scanPatterns = new Listing {
    "src/**/*.vue"
  }
}

linter {
  preset = "happy-path"
}

typeChecker {
  enabled = true
  strict = true
}

entries = new Listing {
  new ConfigEntry {
    name = "web app"
    basePath = "apps/web"
    files = new Listing { "src/**/*.vue" }
    typeChecker {
      tsconfig = "tsconfig.app.json"
    }
  }
}

lsp {
  lint = true
  typecheck = false
  editor = false
  formatting = false
}
```

## JSON 構成

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "compiler": {
    "sourceMap": true,
    "vapor": false,
    "customRenderer": false,
    "templateSyntax": "standard"
  },
  "vite": {
    "scanPatterns": ["src/**/*.vue"]
  },
  "linter": {
    "preset": "happy-path"
  },
  "typeChecker": {
    "enabled": true,
    "strict": true
  },
  "musea": {
    "include": ["src/**/*.art.vue"],
    "basePath": "/__musea__"
  }
}
```

## コンパイラの詳細

[コンパイラオプションと構文](./compiler-configuration-reference.md)に全オプション、
テンプレート構文、JSX/TSX の出力モード、単独 HTML の方言検出をまとめています。

## 静的解析オプション

npm lint パスには `linter` を使用します。

```ts
export default defineConfig({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
});
```

### Lint Rule Options

一部の rule は `linter.ruleOptions` に型付き設定を受け取ります。完全な一覧は
[ルール オプション](../rules/options.md) を参照してください。重大度は引き続き
`linter.rules` で設定します。

npm チェック パスには `typeChecker` を使用します。

```ts
export default defineConfig({
  typeChecker: {
    enabled: true,
    strict: true,
    checkProps: true,
    checkEmits: true,
    checkTemplateBindings: true,
    // Vue 3 Options API template bindings; default-on (matches vue-tsc).
    optionsApi: true,
  },
});
```

`typeChecker.optionsApi` は Vue 3 オプション API テンプレート バインディングを解決します
(プレーン `<script> export default { ... }` の `data`/`computed`/`methods`/`inject`/`setup`/`props`)。
標準ビルドで出荷され (`legacy` 機能ではない)、**デフォルトでオン**(`vue-tsc` と一致)、
また、`<script setup>` 以外のコンポーネントに対してのみ実行されるため、共通パスはゼロコストのままになります。セット
`optionsApi: false` でオプトアウトします。レガシー Vue 2.7 / Nuxt 2 のサポート (`typeChecker.legacyVue2`、追加)
Nuxt 2 テンプレート グローバル) は、別の `legacy` ビルド オプトインです。

`typeChecker.tsconfig` と `typeChecker.corsaPath` は共有スキーマの一部ですが、
プロジェクトに基づいた Corsa パスは、今日の Rust CLI サーフェスです。 `corsaPath` は `vize check` によって共有されます。
タイプ認識の `vize lint` および `vize lsp` (`typeChecker.tsgoPath` は非推奨のエイリアスです)。ランタイム
スタックは TypeScript 7 の native platform package (`typescript` / `@typescript/typescript-*`) と
Corsa/corsa-bind API レイヤーです。特定のインストール済み `lib/tsc` 実行ファイルを指定する必要がなければ、
`corsaPath` は未設定のままにしてください。アンビエント宣言、生成された自動インポート ファイル、パス エイリアス、および Vue を保持します
プロジェクト `tsconfig.json` 内の `ComponentCustomProperties` 宣言、およびパッケージ スクリプトの使用
`--tsconfig` または `--corsa-path` オーバーライドの場合は `vize:check:app` など。

```json
{
  "typeChecker": {
    "servers": 1
  }
}
```

`typeChecker.servers` は、将来の Corsa ワーカー プール用に予約されています。プロジェクトセッションの直接ランナー
現在、`1` のみをサポートしています。値を大きくすると、同時実行性を調整する代わりに失敗が早くなります。

## 美術館のオプション

共有構成は現在、ギャラリー ファイル セットとルートをカバーしています。

```ts
export default defineConfig({
  musea: {
    include: ["src/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
    basePath: "/__musea__",
    storybookCompat: false,
    inlineArt: false,
  },
});
```

`previewCss`、`previewSetup`、`tokensPath`、`theme`、および
`storybookOutDir` を `vite.config.ts` の `musea()` に直接変換します。

## 既存の専用設定を使うワークフロー

専用の `vize.config.*` をすでに使っているプロジェクトでは、次の設定例を引き続き利用できます。新しく導入する場合は、[日常の開発ワークフロー](./workflows.md)にある既存の Vite 設定を使います。公開済みリリースの対応状況は[設定ガイド](./configuration.md)を参照してください。

```ts
import { defineConfig } from "vize";

export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  linter: {
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    tsconfig: "tsconfig.json",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```
