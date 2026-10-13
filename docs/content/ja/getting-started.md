---
title: はじめる
description: 既存の Vite 設定と TypeScript プロジェクトで Vue に Vize を導入する。
---

<!-- Reviewed translation; source: getting-started.md -->

# はじめる

[Vite+](https://viteplus.dev/guide/install) を使って Vue 3 アプリに Vize を導入します。ツールの設定は `vite.config.ts` に、TypeScript のプロジェクト設定は `tsconfig.json` にまとめます。Vize は開発中です。導入前に[サポート状況](./stability.md)を確認してください。

## 1. 統合パッケージをインストールする

Vite+ を導入済みの Vue プロジェクトで実行します。

```bash
vp install -D @vizejs/vite-plugin
```

プロジェクトの Vite+（0.2.3 以降）を使います。通常の Vite は [Vite プラグインの移行](./guide/migration.md#vite-plugin)、Nuxt は [Nuxt 統合](./integrations/nuxt.md)を参照してください。

## 2. `vite.config.ts` を変更する

統合用の `defineConfig` がコンパイラと検査タスクを追加します。既存の Vue プラグインの import と `plugins` 内の `vue()` を削除し、alias、server、test、その他のプラグインは残します。[移行ガイド](./guide/migration.md)に変更前後の例を載せています。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

## 3. アプリをビルド・検査する

```bash
vp dev
vp build
vp run check
```

`vp run check` は Vue の型チェック・lint・フォーマットと、Oxlint・Oxfmt をまとめて実行します。作業中は個別のタスクも使えます。

```bash
vp run typecheck
vp run lint
vp run fmt:check
```

修正を適用するには `vp run check -- --fix` を実行します。生成された Vize タスクには `vp run` を使ってください。組み込みの `vp check`・`vp lint`・`vp fmt` は Vite+ 自身のツールを実行します。既存のスクリプトは保持され、生成タスクは `vize:<名前>` になります。[タスク名と上書き](./guide/vite-plus.md#tasks)を確認してください。

## 次にやりたいこと

- [ルールやコンパイラーの設定を変更する](./guide/configuration.md)。
- [既存のツールから移行する](./guide/migration.md)。変更前後の例を確認できます。
- [lint の指摘を理解する](./rules/all.md)。Vue の問題例と修正例を掲載しています。
- [コンポーネントを探す](./guide/ui/index.md)。使い方と API を確認できます。
- [Musea でコンポーネントをプレビューする](./guide/musea.md)。
- [エディターを設定する（英語）](../guide/vite-plus-editor.md)。共通のネイティブ設定を利用できます。

> [!NOTE]
> ネイティブ CLI・エディターの Vite 設定の読み込みと、専用設定を作らない init は、次のリリースに向けて準備中です。それまでは、公開済みのネイティブツールで共通設定を変更するには既存の専用形式を使います。[リファレンス](./guide/configuration-reference.md)にその形式をまとめています。

## Vite+ なしで利用する

[単独 CLI](./guide/cli.md)では `vize lint`・`vize fmt`・`vize check` を実行できます。対象パッケージのルートから実行し、TypeScript のプロジェクト設定は `tsconfig.json` に残します。ネイティブコマンドも `vite.config.*` のトップレベルの `vize` 設定を共有できます。検索規則と現在の範囲は [CLI の設定](./guide/configuration.md#standalone-cli)を参照してください。

対話形式の設定は `vpx vize init --dry-run` で変更予定を確認し、`vpx vize init` で機能を選びます。[プロジェクト設定（英語）](../guide/init.md)に検出規則と編集できない条件をまとめています。init は既存のプロジェクト設定と既定値を使い、専用の Vize 設定ファイルは作成しません。
