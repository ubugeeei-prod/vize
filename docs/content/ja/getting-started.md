---
title: はじめる
description: 既存の Vite 設定と TypeScript プロジェクトで Vue に Vize を導入する。
---

<!-- Reviewed translation; source: getting-started.md -->

# はじめる

[Vite+](https://viteplus.dev/guide/install) を使って Vue 3 アプリに Vize を導入します。ツールの設定は `vite.config.ts` に、TypeScript のプロジェクト設定は `tsconfig.json` にまとめます。Vize は開発中です。導入前に[サポート状況](./stability.md)を確認してください。

## 構成を選ぶ

| プロジェクト                     | 始める場所                                              |
| -------------------------------- | ------------------------------------------------------- |
| Vite+ の Vue アプリ              | 以下の 3 ステップ                                       |
| 通常の Vite の Vue アプリ        | [コンパイラーだけを置き換える](./guide/migration.md#vite-plugin) |
| Nuxt                             | [Nuxt 統合](./integrations/nuxt.md)                     |
| Void（`@void/vue`）              | [Void 統合](./integrations/void.md)                     |
| ターミナルや CI だけで使う       | [Vite+ なしで利用する](#vite-なしで利用する)            |
| Oxlint を使っている              | [Oxlint プラグイン](./guide/oxlint.md)                  |
| エディターだけで使う             | [言語サーバー](./guide/language-server.md)              |

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

修正を適用するには `vp run check -- --fix` を実行します。

> [!WARNING]
> `vp check` ではなく `vp run check` を使ってください。組み込みの `vp check`・`vp lint`・`vp fmt` は Vite+ 自身のツールを実行し、Vize の Vue のチェックは行いません。

既存のスクリプトは保持され、生成タスクは `vize:<名前>` になります。[タスク名と上書き](./guide/vite-plus.md#tasks)を確認してください。

## 次にやりたいこと

- [Content Mapper で Vue ファイルを型チェックする](./guide/content-mapper.md)。TypeScript 7.1 と tsrs のどちらでも実行できます。コマンドは[型チェック](./guide/type-checking.md)にあります。
- [ルールやコンパイラーの設定を変更する](./guide/configuration.md)。
- [既存のツールから移行する](./guide/migration.md)。変更前後の例を確認できます。
- [lint の指摘を理解する](./rules/all.md)。Vue の問題例と修正例を掲載しています。
- [Vapor モードを試す](./guide/vapor.md)。コンポーネント単位で切り替えられます。
- [コンポーネントを探す](./guide/ui/index.md)。使い方と API を確認できます。
- [Musea でコンポーネントをプレビューする](./guide/musea.md)。
- [エディターを設定する](./guide/language-server.md)。VS Code、Zed、Neovim などに対応しています。
- [JSR から JavaScript API を使う](./guide/jsr.md)。Node・Deno・Bun で利用できます。

> [!NOTE]
> CLI・Vite 連携・エディターは `vite.config.*` の `vue:` 設定を共有し、`tsconfig.json` の TypeScript プロジェクト設定と `vueCompilerOptions` を利用します。専用の `vize.config.*` ファイルとトップレベルの `vize:` 設定は廃止されました。移行方法は[設定ガイド](./guide/configuration.md)を参照してください。

## Vite+ なしで利用する

[単独 CLI](./guide/cli.md) は、インストールせずにどの Vue プロジェクトでも試せます。

```bash
vpx vize lint src
vpx vize check
vpx vize fmt --check src
```

`vize lint`・`vize fmt`・`vize check` を実行できます。型チェックは TypeScript 7.1 を推奨します。Vize の content mapper を入れ、`tsc --runExternalCode --noEmit` を実行してください。tsrs でも同じプロジェクトを実行できます。対象パッケージのルートから実行し、TypeScript のプロジェクト設定は `tsconfig.json` に残します。ネイティブコマンドも `vite.config.*` のトップレベルの `vue` 設定を共有できます。検索規則と現在の範囲は [CLI の設定](./guide/configuration.md#standalone-cli)を参照してください。

対話形式で導入する場合は、`vpx vize init --dry-run` で変更予定を確認し、`vpx vize init` で機能を選びます。[プロジェクトの初期設定](./guide/init.md)に、プロジェクトの検出方法と、設定を自動編集できない場合の対応をまとめています。init は既存のプロジェクト設定と既定値を使い、専用の Vize 設定ファイルは作成しません。
