---
title: 共有設定リファレンス
---

<!-- Reviewed translation; source: guide/configuration-reference.md -->

<span id="standalone-configuration-reference"></span>

# 共有設定リファレンス

既存の Vite 設定の `vue:` と、TypeScript プロジェクトの `vueCompilerOptions` を
使います。最小の例は[設定ガイド](./configuration.md)を参照してください。

## 設定ファイル

npm コマンドと `@vizejs/vite-plugin` は、対象プロジェクトの `vite.config.*` にある `vue:` を読みます。TypeScript と Vue テンプレートの設定は、同じプロジェクトの `tsconfig.json`・`vueCompilerOptions` から読みます。

CLI とエディタは、最も近いプロジェクトの `vite.config.ts`、`.mts`、`.cts`、
`.js`、`.mjs`、`.cjs` を探索します。`package.json`、`tsconfig.json`、`jsconfig.json`
がプロジェクトの境界です。CLI では `--config` で Vite 設定を明示できます。
`vize.config.*` は廃止しました。Vite 設定と同じ場所に残った専用ファイルは読みません。
専用設定しかないプロジェクトでは、移行エラーになります。

Vite 内の相対パスは、選択された Vite の `root` を基準にします。相対指定の `root`
自体は設定ファイルのディレクトリから解決します。CLI に直接渡した入力と `--tsconfig`
はコマンドの作業ディレクトリを基準にします。直接渡したプラグインのオプションと
明示的なエディタの機能スイッチは、共有設定を上書きします。

Rust CLI の `check`、`lint`、`lsp`、`fmt` も、コマンドの設定を上記のファイル名と優先順位で読み込みます。

<span id="typescript-設定"></span>

## TypeScript の設定

```ts
// vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  server: { port: 3000 },
}).withVue({
  compiler: { sourceMap: true, vapor: false, templateSyntax: "standard" },
  linter: { preset: "happy-path" },
  formatter: { printWidth: 100 },
  languageServer: { hover: true },
});
```

`vue` は `version`、`compiler`、`typeChecker`、`linter`、`formatter`、
`languageServer`、`globalTypes`、`files`、`ignores`、`basePath`、`lib`、`vite`、
`musea`、`experimentals`、`dialect`、`name`、`entries` を受け付けます。ネイティブのオプション名を使い、未知のキーは
エラーになります。コンパイルモードは[コンパイラのリファレンス](./compiler-configuration-reference.md)、
Lint の型付きオプションは[ルールオプション](../rules/options.md)を参照してください。

<span id="vue-タイプの解決"></span>

## Vue の型解決

`vize check`、言語サーバー、npm コマンドは、解析対象のプロジェクトから `vue`、`@vue/compiler-sfc`、関連するアンビエント型を解決します。Vue の型を、公開された `vize` パッケージや Vize のビルド時のバージョンに固定することはありません。Vue 3 のパッチ版・マイナー版・プレリリース版は、利用するプロジェクト側で選べます。

結果を再現できるよう、利用する Vue のバージョンを自分のプロジェクトに明示し、`vue`、`@vue/compiler-sfc`、Nuxt などの統合を対応するバージョンにそろえてください。`vize check` はプロジェクトのルートから実行するか、`vue.typeChecker.tsconfig` で対象パッケージの設定を指定します。`vue.typeChecker.corsaPath` は型チェッカーの実行ファイルを選ぶ設定であり、Vue の型のバージョンを変更する設定ではありません。

複数の Vue バージョンをサポートするプロジェクトでは、各バージョンをパッケージのテスト行列で検証してください。Vize は固定された型ファイルのパスではなく、その検証環境で有効な依存関係をたどります。

TypeScript 標準の `extends`（パッケージと配列の形式を含む）と `references` を使えます。

```json
{
  "extends": "./tsconfig.base.json",
  "compilerOptions": { "strict": true, "noEmit": true },
  "vueCompilerOptions": {
    "target": 3.5,
    "strictTemplates": true,
    "checkUnknownComponents": true
  },
  "include": ["src/**/*"]
}
```

対応する Vue オプションは `target`、`strictTemplates`、`checkUnknownProps`、
`checkUnknownComponents`、`checkUnknownEvents`、`checkUnknownDirectives`、
`strictVModel`、`strictCssModules`、`jsxSlots`、`inferComponentDollarEl`、
`inferComponentDollarRefs`、`inferTemplateDollarEl`、`inferTemplateDollarRefs`、
`inferTemplateDollarSlots`、`inferTemplateDollarAttrs`、`fallthroughAttributes`、
`checkRequiredFallthroughAttributes`、`resolveStyleImports`、`resolveStyleClassNames`、
`vapor` です。`vue-tsc` の全オプションに対応するという意味ではありません。

`target` は 2.7 までの Vue 2.x、Vue 3.x、`99`、`"auto"` を受け付けます。
自動選択はプロジェクトにインストールされた Vue に従い、Vite の明示的な `vue.version`
を優先します。未対応の値は設定エラーになります。TypeScript の厳密さは
`compilerOptions` で指定し、Vize の明示的な型チェック設定は対応する既定値を上書きします。

### TypeScript プロジェクトの所有関係と対応範囲

設定はソースファイルを所有する TypeScript プロジェクトから読みます。継承した JSONC、標準のパッケージ・配列形式の `extends`、プロジェクトの `references` も保持します。`target: "auto"` はそのプロジェクトにインストールされた Vue を解決し、見つからない場合や未対応のバージョンでは設定エラーになります。2.7 未満の数値の Vue 2 ターゲットは Vue 2、2.7 は Vue 2.7、Vue 3.x と `99` は現在の Vue 3 系列を選びます。Vite に `vue.version` を明示した場合は、`"3"` も含めて `target` より優先します。

| Vue compiler option | 対応する動作 |
| --- | --- |
| `target` | 上記の規則でプロジェクトの Vue ターゲットを選択します。 |
| `strictTemplates` | 対応する厳密なテンプレート検査の既定値を設定します。明示した個別の設定が優先します。 |
| `checkUnknownProps`, `checkUnknownComponents`, `checkUnknownEvents`, `checkUnknownDirectives` | 対応する未知のテンプレート要素の検査を設定します。 |
| `strictVModel`, `strictCssModules`, `strictComponentAttrs` | 各項目の検査を設定します。`strictComponentAttrs` は明示指定が必要で、`strictTemplates` だけでは有効になりません。 |
| `jsxSlots` | JSX のスロット検査を設定します。 |
| `inferComponentDollarEl`, `inferComponentDollarRefs`, `inferTemplateDollarEl`, `inferTemplateDollarRefs`, `inferTemplateDollarSlots`, `inferTemplateDollarAttrs` | 対応するコンポーネント・テンプレートの型推論を設定します。 |
| `fallthroughAttributes`, `checkRequiredFallthroughAttributes` | fallthrough 属性の扱いと検査を設定します。 |
| `resolveStyleImports`, `resolveStyleClassNames` | 対応するスタイルの import・クラス解決を設定します。 |
| `vapor` | 対応する Vapor のプロジェクト設定です。 |
| `plugins` | `vue-router/volar/sfc-typed-router` を認識します。任意の Vue 言語プラグインを読み込む機能ではありません。 |

`optionsWrapper` は未対応です。Vue ツールではコードを前後から包む prefix/suffix を指定するもので、Options API のスイッチではありません。Vize の既存の Options API 検査は別の `vue.typeChecker.optionsApi` で設定します。ここに挙げた対応は限定的なマッピングで、すべての `vue-tsc` オプションとの同等性を示すものではありません。

### Options API と旧 Vue の互換性

`vue.typeChecker.optionsApi` は、Vue 3 の Options API を使ったコンポーネントのテンプレートで、`data`・`computed`・`methods`・`inject`・`setup`・`props` のバインディングを解決します。通常の `<script> export default { ... }` が対象です。標準ビルドに含まれ、`vue-tsc` と同じく **デフォルトで有効**です。`<script setup>` のコンポーネントではこの処理を実行しないため、その経路に追加のコストはかかりません。無効にするには `optionsApi: false` を設定します。

Vue 2.7・Nuxt 2 への対応は、別途 `legacy` ビルドで有効にする機能です。`vue.typeChecker.legacyVue2` によって Nuxt 2 のテンプレートグローバルが追加されます。

<span id="試験的なフラット-エントリ"></span>

## 実験的なフラットエントリ

モノレポでは、パッケージごとの TypeScript プロジェクトと Vite 設定を使います。`vue.entries` には `basePath`、`files`、`ignores`、`linter` を指定でき、一致する Vue Lint の設定を上書きします。ネイティブの共有設定は Vite オブジェクトの `vue:` に置き、配列を Vite 設定としてエクスポートしないでください。スコープ別のコンパイラ設定が必要な JavaScript 統合では、プラグインのインライン `config` を使います。

```ts
export default defineConfig({
  root: "apps/web",
  vue: {
    entries: [
      {
        files: ["src/trusted/**/*.vue"],
        linter: { rules: { "vue/no-v-html": "off" } },
      },
    ],
  },
});
```

entry には `basePath`、`files`、`ignores`、`linter` を指定できます。
その他はプロジェクト全体の設定です。TypeScript のプロジェクト参照は移さないでください。

## 共有設定と extends

Vize 独自の `extends` はありません。通常の JavaScript オブジェクトを共有します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import vpConfig from "./build-options.ts";
import vueConfig from "./vue-options.ts";

export default defineConfig(vpConfig).withVue(vueConfig);
```

TypeScript 標準の `extends` と Vite の設定関数は引き続き利用できます。

## コンパイラの詳細

[コンパイラオプションと構文](./compiler-configuration-reference.md)に全オプション、
テンプレート構文、JSX/TSX の出力モード、単独 HTML の方言検出をまとめています。

## 静的解析のオプション

Vue Lint は `vue.linter`、言語サーバーは `vue.languageServer` に設定します。
Vue のルールは `vue.linter.rules`、Oxlint のルールは従来どおり `lint.rules` に置きます。
クロスファイル解析と型付き Lint は[Lint の実行](./lint-execution.md)、
型付きルールオプションは[ルールオプション](../rules/options.md)を参照してください。
未知のルールオプションはエラーになります。

<span id="lint-rule-options"></span>

一部のルールは、`vue.linter.ruleOptions` で型付きの設定を受け取ります。全項目は
[ルールのオプション](../rules/options.md)を参照してください。診断の重大度は
引き続き `vue.linter.rules` で指定します。

`vue.typeChecker.tsconfig` と `vue.typeChecker.corsaPath` は共有スキーマに含まれます。プロジェクトの型情報を使う Corsa の実行経路は、現在の Rust CLI で利用できます。`corsaPath` は `vize check`、型情報を使う `vize lint`、`vize lsp` で共有します。`vue.typeChecker.tsgoPath` は非推奨の別名です。実行時は TypeScript 7 のネイティブプラットフォームパッケージ（`typescript` / `@typescript/typescript-*`）と、Corsa / corsa-bind の API を使います。

インストール済みの特定の `lib/tsc` 実行ファイルを指定する必要がなければ、`corsaPath` は省略してください。アンビエント宣言、自動生成された auto-import ファイル、パスエイリアス、Vue の `ComponentCustomProperties` の宣言は、プロジェクトの `tsconfig.json` に残します。`--tsconfig` や `--corsa-path` をコマンドで上書きする場合は、`vize:check:app` などのパッケージスクリプトにまとめます。

`vue.typeChecker.servers` は Corsa プロセス数の上限です。`1`〜`32` を指定でき、`0` と `32` を超える値は設定エラーになります。省略時は、対象プロジェクトと利用可能な CPU に応じて並列度を調整します。

<span id="美術館のオプション"></span>

## Musea のオプション

探索、ルート、VRT は `vue.musea` にまとめます。`musea()` に直接渡すオプションは
対応する共有値を上書きします。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [musea({ storybookCompat: false })],
  vue: {
    musea: {
      include: ["src/**/*.art.vue"],
      exclude: ["node_modules/**", "dist/**"],
      basePath: "/__musea__",
      inlineArt: false,
    },
  },
});
```

`previewCss`、`previewSetup`、`tokensPath`、`theme`、`storybookOutDir` など、表示に関する設定は `vite.config.ts` の `musea()` に直接渡してください。

`musea-vrt` も同じ Vite 設定の `include`、`exclude`、
`vrt` を読みます。`vize musea new` は専用設定を作らず、ストーリーの例を生成します。

## 移行レシピ

専用の JSON・PKL と独自の `extends` は廃止されています。通常の JavaScript オブジェクトをインポートして再利用します。以下は既存設定の移行例です。

### 動的な Vite 設定

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  vue: {
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
    languageServer: {
      lint: true,
      typecheck: false,
      editor: false,
      formatting: false,
    },
    musea: {
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
    },
  },
}));
```

### 廃止された JSON・PKL 設定の移行

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: { scanPatterns: ["src/**/*.vue"] },
  linter: { preset: "happy-path" },
  typeChecker: { enabled: true, strict: true },
  languageServer: { lint: true, typecheck: false, editor: false, formatting: false },
  musea: { include: ["src/**/*.art.vue"], basePath: "/__musea__" },
});
```

### パッケージごとのプロジェクト

`entries` は lint のスコープだけを扱います。型チェッカーとフォーマッタのパッケージ別設定は、それぞれのパッケージの Vite・TypeScript プロジェクトに置きます。以下の 2 ファイルは別々に作成します。

```ts
// apps/web/vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  name: "web app",
  files: ["src/**/*.vue"],
  formatter: { printWidth: 100 },
  typeChecker: { tsconfig: "tsconfig.app.json" },
});
```

```ts
// packages/ui/vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
export default defineConfig({}).withVue({
  name: "ui package",
  files: ["src/**/*.vue"],
  formatter: { printWidth: 100, singleQuote: true },
});
```

### 静的解析とチェッカープロセス

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
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

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  typeChecker: { servers: 1 },
});
```

### 既存ワークフローの設定

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
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
