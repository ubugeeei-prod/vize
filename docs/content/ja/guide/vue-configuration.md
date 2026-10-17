---
title: Vue の設定を共有する
---

<!-- Reviewed translation; source: guide/vue-configuration.md -->

# Vue の設定を共有する

ネイティブの Vue 設定を `vite.config.ts` の `vue` にまとめます。コンパイラ、
CLI、エディタで同じオプション名を使えます。ビルド、型チェック、Lint、整形を
まとめて実行するには、[Vite+ 連携](./vite-plus.md)から始めてください。

## 設定を一つにまとめる

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    version: "3",
    compiler: { sourceMap: true },
    typeChecker: { strict: true },
    linter: { preset: "essential" },
    formatter: { singleQuote: true },
    languageServer: { hover: true },
  },
  server: { port: 3000 },
});
```

`vue` では、ネイティブの `compiler`、`typeChecker`、`linter`、`formatter`、
`languageServer`、`globalTypes`、`files`、`ignores`、`basePath`、`lib` を使います。
`version`、`vite`、`musea`、`experimentals`、`dialect`、`name`、`entries` も指定できます。コンパイラと整形のセクションには
`enabled` はありません。型チェッカー、Linter、言語サーバーでは、ネイティブと
同じ `enabled` を使えます。未知のキーを canonical な `vue` に置くとエラーになります。

## 既存の Vite+ 設定を使う

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

const vpConfig = { server: { port: 3000 } };
const vueConfig = {
  compiler: { sourceMap: true },
  formatter: { singleQuote: true },
};

export default defineConfig(vpConfig).withVue(vueConfig);
```

既存の Vite+ オブジェクト、Promise、設定関数、async 関数をそのまま渡せます。
設定関数には Vite の環境を渡し、プラグイン、フック、Vite+ のフィールドを保ちます。
Vue プラグインの注入は1回だけ実行します。通常の JavaScript オブジェクトを import して
共有できるため、Vize 専用の設定ファイルや独自の `extends` は不要です。

## パスとスコープ

Vite の設定にあるネイティブの相対パスは、選択された Vite の `root` を基準にします。
`root` 自体は設定ファイルのディレクトリから解決します。CLI に直接渡したファイル名は、
従来どおりコマンドを実行したディレクトリを基準にします。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  root: "packages/app",
  vue: {
    files: ["src/**/*.vue"],
    entries: [
      {
        files: ["src/trusted/**/*.vue"],
        linter: { rules: { "vue/no-v-html": "off" } },
      },
    ],
  },
});
```

canonical な entry に置けるのは、現在 `basePath`、`files`、`ignores`、`linter` だけです。
ほかのセクションと `version` はプロジェクト全体に指定し、entry に置くとエラーになります。
CLI のスコープ付き Lint は従来の動作を保ちます。エディタでのファイル単位のスコープ適用と、
ほかのセクションのスコープ対応は未完了です。

## TypeScript プロジェクトの設定

TypeScript の `compilerOptions`、標準の `extends`、`references` は既存の
TypeScript プロジェクトに置きます。`strictTemplates` などの Vue テンプレート設定は
`vueCompilerOptions` に記述します。CLI とエディタはこのプロジェクト設定を使います。
[設定ガイド](./configuration.md#typescript-and-vue-template-checking)も参照してください。

## ワークスペース外の設定を編集する

エディタは、Vite 設定から import したヘルパーと TypeScript プロジェクトの設定依存を
追跡します。ワークスペース外のファイルも対象です。再読み込みに失敗した場合は、
最後に正常に読み込めた設定を保ちます。その依存グラフに加え、直近の失敗した
読み込みで具体的に判明した監視可能なファイルも対象にします。失敗した設定値は採用しません。

外部ファイルの変更を自動通知するには、LSP クライアントが `dynamicRegistration` と
`relativePatternSupport` の両方に対応し、監視の登録が成功する必要があります。
確認したクライアントの対応は次のとおりです。

| クライアントの基準版 | 外部設定の変更 |
| --- | --- |
| [VS Code 1.107.1 / languageclient 9.0.1](https://github.com/microsoft/vscode-languageserver-node/blob/release/client/9.0.1/client/src/common/fileSystemWatcher.ts#L40-L65) | 両方の機能に対応し、自動監視を登録できます。 |
| [Zed 1.16.1](https://github.com/zed-industries/zed/blob/v1.16.1/crates/lsp/src/lsp.rs#L812-L817) | 両方の対応を通知します。GUI での監視動作の検証は未完了です。 |
| [Neovim 0.11.4](https://github.com/neovim/neovim/blob/v0.11.4/runtime/lua/vim/lsp/protocol.lua#L561-L566) | macOS・Windows の既定値は両方に対応します。Linux の既定値では次の通常の言語操作時に再確認します。 |
| [Helix 25.07.1](https://github.com/helix-editor/helix/blob/25.07.1/helix-lsp/src/client.rs#L596-L599) | 相対パターンに非対応です。次の通常の言語操作時に再確認します。 |
| [Vim / vim-lsp 474c6566](https://github.com/prabirshrestha/vim-lsp/blob/474c656659b5fb51eec6770309e0211c8aa49b5b/autoload/lsp.vim#L686-L694) | 動的なファイル監視はありません。次の通常の言語操作時に再確認します。プロジェクト内のバッファ保存でも Vize に通知します。 |
| [Emacs 29.3 / Eglot 1.12.29](https://github.com/emacs-mirror/emacs/blob/emacs-29.3/lisp/progmodes/eglot.el#L3421-L3468) | プロジェクト内のディレクトリ監視は外部の対象をカバーしません。次の通常の言語操作時に再確認します。 |

ホバーや補完を要求するなど、通常の言語操作でも設定の変更を再確認します。
外部ファイルを監視できないクライアントでも、この操作で修復を検知できます。
別のプロセスが変更した直後に、自動で反映されるわけではありません。
Zed の自動テストは現在プロトコルの再生を使っており、GUI の動作を証明するものでは
ありません。クライアント側で対応機能を上書きした場合も、自動監視が無効になり得ます。

### 見つからない import 先を修復する

新しく見つからなくなったヘルパーは、Node が構造化されたローカルの `file:` URL を
返す場合や、Vite 8 が拡張子付きファイルへの明示的な相対・絶対パスを返す場合に
監視対象にできます。必要な監視にクライアントが対応していれば、そのファイルの
作成・修復をきっかけに再読み込みできます。

Vite のバンドルが失敗した場合、不足しているファイルが報告されても、途中のヘルパーが
すべて列挙されるとは限りません。列挙されなかったヘルパーへの編集は、次の通常の言語操作で
検出します。報告された不足ファイル自体は、自動監視の対象にできます。

具体的なファイル情報を持たない Vite 7 のエラー、裸のパッケージ名、拡張子を省いた
リクエスト、独自に throw したエラー、`%`・`#`・`?` を含む曖昧な Vite の指定では、
次の通常の言語操作で再試行します。既知の依存グラフの監視は続けますが、エラーの
文章から新しい監視対象を推測することはありません。

設定関数は読み込みごとに1回実行します。再読み込みでは、既知のファイルを前後で確認します。
Vite の公開バンドル API が新しいヘルパーを発見する前に、そのファイルが変更される
場合もあります。複数ファイルへの同時編集を、一括した同一時点の状態として取得する
仕組みではありません。新しく発見したファイルの観測結果は、次の通常の言語操作で
再確認します。

Vite のバンドル API は、import に使った symlink のパスを含めず、解決先のヘルパーだけを
返す場合があります。既知の解決先への編集は検出しますが、symlink の接続先だけを変えた
場合は言語サーバーを再読み込みしてください。依存グラフにない import 経路は、通常の言語操作でも
検出できません。Vite がインストールされていない場合、Node の resolve フックは、拡張子付きの
明示的なローカルファイル指定も監視専用のパスとして保持します。

## 既存の設定を移行する

トップレベルの `vize:` と `vize.config.*` のネイティブ設定を `vue:` に移します。
Vue ツールには `typeChecker`、`linter`、`formatter` を使い、Vite+ のタスク拡張には
`typecheck`、`lint.vue`、`fmt.vue`、`pack.vue` を使います。廃止した専用設定の形式と
Vize 独自の `extends` は読みません。共有する設定は通常の JavaScript オブジェクトとして
import してください。`vueCompilerOptions` を含む TypeScript の設定は
`tsconfig.json` に置きます。
