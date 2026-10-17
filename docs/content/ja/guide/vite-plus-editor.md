---
title: エディターの設定
---

<!-- Reviewed translation; source: guide/vite-plus-editor.md; scope: complete document -->

# エディターの設定

Vite+ の Oxc と Vitest の連携はそのまま使い、Vue の部分に Vize を追加します。おすすめの分担は次のとおりです。Vue の型チェック、ネイティブの lint、言語機能、Vue のフォーマットは Vize が担当します。JS/TS の lint（SFC 内のスクリプトの診断を含む）と、Vue 以外のファイルのフォーマットは、引き続き Oxc が担当します。[Vite+ 用の設定ヘルパー](./vite-plus.md)が、重複する Oxlint のルールを無効にし、Vue ファイルを Oxfmt の対象から外します。

## VS Code

`vp run editor:setup` を実行し、ワークスペースの推奨拡張機能をインストールします。このタスクは `.vscode/extensions.json` に `VoidZero.vite-plus-extension-pack` と `ubugeeei.vize` を追加します。すでにある推奨拡張機能やコメントは残ります。あわせて、足りない Vize の機能設定と、次の Vue 向けの既定値を追加します。

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": true,
  "vize.editor.enable": true,
  "vize.ecosystem.enable": true,
  "vize.formatting.enable": true,
  "[vue]": {
    "editor.defaultFormatter": "ubugeeei.vize",
    "editor.formatOnSave": true
  }
}
```

明示的に書かれている設定は、`false` や既存の Vue フォーマッタの指定も含めて、タスクが書き換えることはありません。機能が有効にならない場合は、既存の設定を見直してください。グローバルのフォーマッタを変更したり、Oxc を無効にしたりすることもありません。JavaScript と TypeScript のフォーマッタの設定は、[Vite+ の IDE ガイド](https://viteplus.dev/guide/ide-integration)のとおりにしておきます。

有効な機能の確認、言語サーバーのバイナリの選択、サーバーの再起動には、`Vize: Show Status` コマンドを使います。ネイティブの言語サーバーが利用できる状態である必要があります。バイナリの検出とダウンロードは拡張機能が行い、プロジェクトの Vite+ のバージョンとは独立しています。

Vue の言語サービスを提供する拡張機能は、ひとつに絞ってください。Vize のエディター機能をすべて使う場合は、このワークスペースで Vue - Official を無効にします。有効なままだと、Vue の型診断や補完が二重に表示されます。Vue - Official を使い続ける場合は、`Vize: Enable Lint-Only Profile` を実行して Vize の型チェックとエディター機能を無効のままにし、Vue のフォーマッタをどちらか一方に決めてください。どちらの構成でも、Oxc と Vitest は有効なままです。

インストール済みのすべての拡張機能を呼び出してしまう、範囲の広い `source.fixAll` を保存時のアクションに設定するのは避けてください。Vite+ が設定する `source.fixAll.oxc` はそのまま使い、ネイティブのルールには Vize の診断のクイックフィックスを使います。Vue のフォーマッタは、上の例のように言語別の `editor.defaultFormatter` で選びます。

## エディター共通のネイティブ設定

ネイティブの言語サーバーは、`vite.config.*` の `vue:` と、プロジェクトの `tsconfig.json` を読み込みます。Vite+ では `defineConfig(vpConfig).withVue(vueConfig)` を使い、既存の Vite+ 設定を保ちながらネイティブ設定を共有します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "essential" },
  formatter: { singleQuote: true },
});
```

TypeScript の設定は `tsconfig.json` の `compilerOptions`、Vue テンプレートの設定は `vueCompilerOptions` に置きます。標準の `extends` と `references` は引き続き使えます。ワークスペースフォルダーは明示的に選択してください。エディター側で明示した機能の有効・無効は、プロジェクトの設定より優先されます。専用の `vize.config.*` は廃止されました。既存の設定は `vue:` と TypeScript プロジェクトへ移してください。探索範囲と移行方法は[設定](./configuration.md)を参照してください。

```json
{
  "compilerOptions": { "strict": true, "noEmit": true },
  "vueCompilerOptions": { "strictTemplates": true }
}
```

## その他のエディター

Vue 以外のファイルには Oxc の連携をそのまま使い、Vue には Vize の言語サーバーを選びます。Vue のフォーマッタはひとつに決め、その他の言語には Vite+ の `fmt` の設定を使い続けてください。Zed では、Vite+ の Oxc の設定を残したまま、`Vue.js` に Vize のサーバーを選択できます。共通のネイティブ設定を使うので、どのエディターでも Vize の診断は同じになります。
