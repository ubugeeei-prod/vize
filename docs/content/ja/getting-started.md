---
title: はじめる
description: Vite+ と1つの設定ファイルで Vue に Vize を導入する。
---

# はじめる

[Vite+](https://viteplus.dev/guide/install) を使って Vue 3 アプリに Vize を導入します。
コンパイル、Vue の lint、フォーマット、型チェックの設定は **`vite.config.ts`** にまとめます。
Vize は開発中です。導入前に[サポート状況](./stability.md)を確認してください。

## 1. 統合パッケージをインストールする

Vite+ を導入済みの Vue プロジェクトで実行します。

```bash
vp install -D @vizejs/vite-plugin
```

プロジェクトの Vite+（0.2.3 以降）を利用します。通常の Vite は
[Vite プラグインの移行](./guide/migration.md#vite-plugin)、Nuxt は
[Nuxt 統合](./integrations/nuxt.md)を参照してください。

## 2. `vite.config.ts` を変更する

Vite+ helper が Vize のコンパイラと検査タスクを登録します。
既存の Vue プラグインの import と `plugins` 内の `vue()` を削除します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

alias、server、test、その他のプラグインの設定は残してください。
[移行ガイド](./guide/migration.md)に変更前後の差分と、対応するコンパイラ設定の移し方を示しています。

## 3. アプリをビルド・検査する

```bash
vp dev
vp build
vp run check
```

`vp run check` は Vize の Vue 型チェック・lint・フォーマットと、Oxlint・Oxfmt を実行します。
作業中は個別のタスクも利用できます。

```bash
vp run typecheck
vp run lint
vp run fmt:check
```

lint の修正とフォーマットを適用する場合は `vp run check -- --fix` を実行します。

**Vize 統合のタスクは `vp run` で実行します。** 組み込みの `vp check`・`vp lint`・`vp fmt` は
Vite+ 自身のツールを実行します。既存の `check` スクリプトは保持されるため、その場合は
`vp run vize:check` で実行します。既存の `lint`・`fmt`・`typecheck` script も同じ命名規則です。
[タスク名と上書き](./guide/vite-plus.md#tasks)も確認してください。

## 次にやりたいこと

- [ルールやコンパイラの設定を変える](./guide/configuration.md) — 統合ごとに設定場所を確認する。
- [既存ツールから移行する](./guide/migration.md) — 対応表とコピーできる差分を使う。
- [lint 診断を理解する](./rules/all.md) — 悪い例と良い例を比較する。
- [コンポーネントを探す](./guide/ui/index.md) — 用途別の UI とリファレンスを見る。
- [自分のコンポーネントをプレビューする](./guide/musea.md) — art ファイルとギャラリーを使う。
- [エディターを設定する（英語）](../guide/vite-plus-editor.md) — ツールの役割とネイティブ設定を確認する。

## Vite+ なしで利用する

[単独 CLI](./guide/cli.md)では `vize lint`・`vize fmt`・`vize check` を実行できます。
設定の場所は `vize.config.ts` です。[単独 CLI の設定](./guide/configuration.md#standalone-cli)を参照してください。

対話形式で既存の Vite・Vite+・Nuxt を設定する場合は `vpx vize init --dry-run` で変更予定を確認し、
`vpx vize init` で機能を選びます。
[プロジェクト設定（英語）](../guide/init.md)に検出規則・オプション・編集できない条件を記載しています。
