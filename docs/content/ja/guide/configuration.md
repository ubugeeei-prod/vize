---
title: 構成
---

<!-- Generated translation; source: guide/configuration.md -->

# 構成

Vize は、共有 npm パッケージ コマンド、Vite プラグイン、および Rust CLI 設定に `vize.config.*` を使用します。

## 設定ファイル

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
