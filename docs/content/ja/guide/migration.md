---
title: 既存ツールから移行
description: 対応表と具体的な設定差分を使って、Vue ツールを1つずつ移行する。
---

<!-- Reviewed translation; source: guide/migration.md; scope: complete document -->

# 既存ツールから移行する

最初に Vite プラグインを置き換え、アプリをビルドしてから検査を導入します。
プロジェクトで結果を比較するまでは、独自ルールと既存のエディターツールを残してください。
Vize は開発中です。全オプションや診断の完全な互換性を保証するものではありません。

## 置き換えるものを選ぶ

| 移行元 | 移行先 | 効果 | インストール |
| --- | --- | --- | --- |
| `@vitejs/plugin-vue` | Vize コンパイラ | Vite で Vue 3 SFC をコンパイル | `@vizejs/vite-plugin` |
| `vue-tsc` の script | `tsc --runExternalCode --noEmit` | TypeScript 7.1 の content mapper | `vize` と `typescript@beta` |
| Vue の lint script | `vp run lint` | Vize の Vue ルールと Oxlint | 同じ統合パッケージ |
| Vue の format script | `vp run fmt` | Vue は Vize、その他は Oxfmt | 同じ統合パッケージ |

[tsrs](https://github.com/oxc-project/tsrs) は、その型チェックの drop-in です。同じ `tsconfig.json` で `tsrs --runExternalCode --noEmit` を実行します。lint とフォーマットのコマンドは [Vite+ の設定ヘルパー](./vite-plus.md) が生成するカスタムタスクです。組み込みの `vp lint`・`vp fmt`・`vp check` とは別に実行します。`vp run typecheck` は同梱チェッカーのままです。マッパーの設定は[型チェック](./type-checking.md)を参照してください。

## Vite+ プロジェクト

既存の Vue/Vite+ プロジェクトに追加します。

```bash
vp install -D @vizejs/vite-plugin
```

`vite.config.ts` を変更します。

```ts annotate="remove:1,2,6;add:3,7"
import { defineConfig } from "vite-plus";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  plugins: [vue()],
  compiler: {},
  server: { port: 3000 },
});
```

設定ヘルパーが Vize コンパイラとネイティブタスクを登録します。ほかのプラグイン、エイリアス、
サーバー設定、テストはそのまま残してください。`compiler: {}` は既定値を使い、
`compiler` 自体を省略することもできます。

対応するプラグインオプションは `compiler` に移します。Vue コンパイラを二重に登録しないでください。

```ts annotate="remove:1;add:2"
  plugins: [vue({ template: { compilerOptions: { whitespace: "preserve" } } })],
  compiler: { whitespace: "preserve" },
```

その他のオプションは[対応範囲](./vite-plugin.md#drop-in-scope)を確認してから移します。
Vue 2 互換モードと実験的なバンドラーは、対応範囲が異なります。

本番ビルドを実行し、content mapper で型チェックして、lint とフォーマットを比較します。先に `tsconfig.json` へマッパーを追加します。tsrs は同じファイルを実行します。

```bash
vp build
tsc --runExternalCode --noEmit -p tsconfig.json
tsrs --runExternalCode --noEmit -p tsconfig.json
vp run lint
vp run fmt:check
```

`vp run typecheck` と `vp run check` は同梱チェッカーを実行します。

既存の `package.json` のスクリプトは保持されます。`check` がある場合は `vp run vize:check` を使います。
明示的な `run.tasks` も優先されます。CI のコマンドを変える前に[タスク名](./vite-plus.md#tasks)を確認してください。

<span id="vite-plugin"></span>

## 通常の Vite: コンパイラだけ置き換える

`vite` の `defineConfig` を維持し、プラグインだけ置き換えます。

```ts annotate="remove:2,6;add:3,7"
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vue()],
  plugins: [vize()],
});
```

```bash
vp install -D @vizejs/vite-plugin
vp exec vite build
```

lint・フォーマット・型チェックは既存のツールを使い続けます。
[Vite プラグインのオプション](./vite-plugin.md#compiler-options)は `vize({ ... })` に設定します。
まとめて実行するタスクが必要になったら、Vite+ の設定ヘルパーを導入できます。

## 結果を比較してから検査コマンドを置き換える

`vue-tsc` は content mapper に置き換えます。lint とフォーマットはネイティブタスクに切り替えます。
変更は1つずつ確認してください。

```json annotate="remove:1,3,5;add:2,4,6"
  "typecheck": "vue-tsc --noEmit",
  "typecheck": "tsc --runExternalCode --noEmit",
  "lint": "eslint src",
  "lint": "vp run vize:lint",
  "format": "prettier --write src",
  "format": "vp run fmt"
```

既存の `lint` スクリプトがあると、設定ヘルパーは `vize:lint` を生成します。
**実際の置き換えでは、その衝突しない名前を使ってください。** tsrs は、`tsc` を `tsrs` に
変えた同じ型チェックスクリプトです。`vp run vize:typecheck` は同梱チェッカーのままです。

```json
{
  "scripts": {
    "typecheck": "tsc --runExternalCode --noEmit",
    "lint": "vp run vize:lint",
    "format": "vp run fmt"
  }
}
```

対応するルールのない独自 ESLint 設定は残し、型診断とフォーマットの差分を確認してから CI を変更します。
フォーマットはファイルを書き換えるため、先に `vp run fmt:check` で確認してください。

<span id="without-vite-plus"></span>

## Vite+ を使わない場合: コマンドを 1 つずつ置き換える

スクリプトを 1 つだけ置き換えて、残りはそのまま使えます。

| 現在のコマンド                             | 置き換え先                                 | 先に読むページ                                                   |
| ------------------------------------------ | ------------------------------------------ | ---------------------------------------------------------------- |
| `vue-tsc --noEmit`                         | `tsc --runExternalCode --noEmit`           | [vue-tsc から移行する](./type-checking.md#vue-tsc-から移行する)  |
| `eslint-plugin-vue` を使う `eslint`        | `vize lint src`                            | [ESLint ルールの対応表](../rules/migration.md)                   |
| `.vue` ファイルへの `prettier --write`     | `vize fmt --preset prettier --write src`   | [Prettier から移行する](./formatter.md#prettier-から移行する)                            |
| Vue - Official の診断                      | `vize lsp`                                 | [言語サーバー](./language-server.md)                             |

```json
{
  "scripts": {
    "typecheck": "tsc --runExternalCode --noEmit",
    "lint:vue": "vize lint src",
    "format:vue": "vize fmt --preset prettier --write src"
  }
}
```

tsrs は、実行ファイルを `tsrs --runExternalCode --noEmit` に変えた同じ型チェックスクリプトです。先に `contentMappers` を追加してください。`vize check` は同梱チェッカーのままです。

Vize がまだ実装していないルールには ESLint を使い続けるか、[Oxlint プラグイン](./oxlint.md)で Vize の
ルールを Oxlint の中で実行してください。同じコミットで新旧のコマンドを並べて実行し、結果を比べてから
古いコマンドを削除してください。

## 単独設定を Vite+ に移す

`vize.config.*` とトップレベルの `vize:` のネイティブ設定を、既存の Vite 設定の
`vue:` に移します。TypeScript の厳密さと `vueCompilerOptions` は `tsconfig.json`
に置き、標準の `extends` と `references` を保ちます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

const vpConfig = { server: { port: 3000 } };

export default defineConfig(vpConfig).withVue({
  linter: { preset: "essential" },
  formatter: { printWidth: 100 },
});
```

CLI とエディタも同じ Vite と TypeScript のプロジェクト設定を読みます。
移行を確認してから古い専用ファイルを削除してください。専用ファイルと Vize 独自の
`extends` は廃止しました。[設定ガイド](./configuration.md)も参照してください。
