---
title: 共通設定のリファレンス
---

<!-- Reviewed translation; source: guide/configuration-reference.md; scope: introduction and discovery -->

<span id="単独-cli-の設定リファレンス"></span>

# 共通設定のリファレンス

まずは既存の `vite.config.*` と `tsconfig.json` を使ってください。[設定ガイド](./configuration.md)で導入方法を説明しています。このページでは、共通のネイティブ設定と、必要な場合に使える専用設定をまとめます。

## 設定ファイル

最も近いプロジェクトの同じディレクトリに専用設定がない場合は、Vite の設定を読み込みます。CLI の `--config` で明示することもできます。プラグインに直接渡したオプションやエディターで明示した機能の設定が優先されます。専用設定内の相対パスは設定ファイルのディレクトリを基準に解決します。Vite の `typeChecker` 内のパスとスコープの `basePath` は、選択した Vite の `root` が基準です。CLI へ明示した入力は実行ディレクトリを基準にします。パッケージの対象範囲は `vize.entries` と、そのパッケージの TypeScript プロジェクトで指定します。

npm コマンドと `@vizejs/vite-plugin` は、プロジェクトのルートで次の設定ファイルを探します。
複数ある場合の優先順位は次のとおりです。

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

Rust CLI の `check`、`lint`、`lsp`、`fmt` も、コマンドの設定を上記のファイル名と優先順位で読み込みます。

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

<span id="vue-タイプの解決"></span>

## Vue の型の解決

`vize check`、言語サーバー、npm コマンドは、解析対象のプロジェクトから `vue`、`@vue/compiler-sfc`、関連するアンビエント型を解決します。Vue の型を、公開された `vize` パッケージや Vize のビルド時のバージョンに固定することはありません。Vue 3 のパッチ版・マイナー版・プレリリース版は、利用するプロジェクト側で選べます。

結果を再現できるよう、利用する Vue のバージョンを自分のプロジェクトに明示し、`vue`、`@vue/compiler-sfc`、Nuxt などの統合を対応するバージョンにそろえてください。`vize check` はプロジェクトのルートから実行するか、`typeChecker.tsconfig` で対象パッケージの設定を指定します。`typeChecker.corsaPath` は型チェッカーの実行ファイルを選ぶ設定であり、Vue の型のバージョンを変更する設定ではありません。

複数の Vue バージョンをサポートするプロジェクトでは、各バージョンをパッケージのテスト行列で検証してください。Vize は固定された型ファイルのパスではなく、その検証環境で有効な依存関係をたどります。

<span id="試験的なフラット-エントリ"></span>

## 実験的なフラット形式のエントリー

モノレポでは、`entries` でルートの既定値とパッケージごとの上書きを指定できます。通常のオブジェクト形式の設定は、内部で1つのエントリーに正規化されます。`defineConfig` は配列のエクスポートにも対応するため、ESLint の flat config のように設定を書けます。

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

## `extends` で共通の設定を継承する

専用の設定ファイルは、ほかの設定ファイルを土台にできます。`extends` には指定子を1つ、または配列で複数指定します。

```json
{
  "extends": ["../../vize.base.json", "@acme/vize-config/strict"],
  "linter": {
    "rules": {
      "vue/no-v-html": "off"
    }
  }
}
```

- `./`、`../`、絶対パスは、`extends` を書いたファイルの場所から解決します。ディレクトリを指定すると、[設定ファイルの優先順](#設定ファイル)でその中の `vize.config.*` を読みます。
- それ以外の指定子はパッケージとして、`extends` を書いたファイルから Node の解決方法で探します。`@scope/pkg/subpath` はパッケージの `exports` に従います。
- 継承元は順番に適用され、`extends` を書いたファイルの設定が最後に優先されます。オブジェクトはキーごとにマージし、`entries` は継承元のエントリーの後ろに追加します。それ以外の値（文字列、数値、`files` などの配列）は置き換えます。
- `entries` の各項目も、`entries` を持たない継承元を `extends` できます。継承元の設定がそのエントリーに加わり、`name`、`basePath`、`files`、`ignores` はエントリー自身の値のままです。
- 循環参照、存在しない継承元、文字列以外の値はエラーになります。空の設定として扱われることはありません。

継承元には JSON、JavaScript、TypeScript、PKL を使えます。`extends` を合成するのは `vize` CLI（`check`、`lint`、`fmt`、`lsp`）と言語サーバーです。言語サーバーは `vize.config.*` の変更で再読み込みしますが、継承元ファイルの変更では再読み込みしません。Vite プラグインはまだ `extends` を読みません。共通の設定オブジェクトを TypeScript で import し、スプレッドしてください。

<span id="pkl-構成"></span>

## PKL の設定

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

<span id="json-構成"></span>

## JSON の設定

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

npm パッケージの lint コマンドには `linter` を使います。

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

<span id="lint-rule-options"></span>

### lint ルールのオプション

一部のルールは、`linter.ruleOptions` で型付きの設定を受け取ります。全項目は
[ルールのオプション](../rules/options.md)を参照してください。診断の重大度は
引き続き `linter.rules` で指定します。

npm パッケージの型チェックコマンドには `typeChecker` を使います。

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

`typeChecker.optionsApi` は、Vue 3 の Options API を使ったコンポーネントのテンプレートで、`data`・`computed`・`methods`・`inject`・`setup`・`props` のバインディングを解決します。通常の `<script> export default { ... }` が対象です。標準ビルドに含まれ、`vue-tsc` と同じく **デフォルトで有効**です。`<script setup>` のコンポーネントではこの処理を実行しないため、その経路に追加のコストはかかりません。無効にするには `optionsApi: false` を設定します。

Vue 2.7・Nuxt 2 への対応は、別途 `legacy` ビルドで有効にする機能です。`typeChecker.legacyVue2` によって Nuxt 2 のテンプレートグローバルが追加されます。

`typeChecker.tsconfig` と `typeChecker.corsaPath` は共有スキーマに含まれます。プロジェクトの型情報を使う Corsa の実行経路は、現在の Rust CLI で利用できます。`corsaPath` は `vize check`、型情報を使う `vize lint`、`vize lsp` で共有します。`typeChecker.tsgoPath` は非推奨の別名です。実行時は TypeScript 7 のネイティブプラットフォームパッケージ（`typescript` / `@typescript/typescript-*`）と、Corsa / corsa-bind の API を使います。

インストール済みの特定の `lib/tsc` 実行ファイルを指定する必要がなければ、`corsaPath` は省略してください。アンビエント宣言、自動生成された auto-import ファイル、パスエイリアス、Vue の `ComponentCustomProperties` の宣言は、プロジェクトの `tsconfig.json` に残します。`--tsconfig` や `--corsa-path` をコマンドで上書きする場合は、`vize:check:app` などのパッケージスクリプトにまとめます。

```json
{
  "typeChecker": {
    "servers": 1
  }
}
```

`typeChecker.servers` は、将来の Corsa ワーカープールのために予約された設定です。現在のプロジェクトセッションの直接実行では `1` のみをサポートします。より大きい値を指定すると、並列数が増えるのではなく、エラーになります。

<span id="美術館のオプション"></span>

## Musea のオプション

現在、共通設定ではギャラリーの対象ファイルとルートを指定できます。

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

`previewCss`、`previewSetup`、`tokensPath`、`theme`、`storybookOutDir` など、表示に関する設定は `vite.config.ts` の `musea()` に直接渡してください。

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
