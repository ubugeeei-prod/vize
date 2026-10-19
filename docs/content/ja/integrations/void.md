---
title: Void
description: Void アプリで Vize を使います。@void/vue のページ、workerd 上の SSR、Void が生成する型に対応します。
---

<!-- Reviewed translation; source: integrations/void.md; scope: complete document -->

# Void

[Void](https://void.cloud) は Vite の上でフルスタックアプリを構築し、Cloudflare の workerd で実行するフレームワークです。Vue 向けアダプターの `@void/vue` は、`pages/` をサーバーでレンダリングし、ブラウザでハイドレーションします。その隣に `vize()` を追加すると、すべての `.vue` ファイルを Vize がコンパイルします。ルーティング、ローダー、アイランド、バインディングはこれまでどおり Void が扱います。

> [!WARNING]
> Vize は開発中です。導入する前に、自分のアプリでビルド結果と診断を比較し、[サポート状況](../stability.md)を確認してください。

## セットアップ

### 1. プラグインをインストールする

```bash
npm install -D @vizejs/vite-plugin
```

### 2. `vite.config.ts` に `vize()` を追加する

```ts
import { voidVue } from "@void/vue/plugin";
import { vize } from "@vizejs/vite-plugin";
import { defineConfig } from "vite";
import { voidPlugin } from "void";

export default defineConfig({
  plugins: [voidPlugin(), vize(), voidVue()],
});
```

`voidVue()` は独自に `@vitejs/plugin-vue` を含んでいます。これを取り除く必要はありません。各 SFC をひとつのコンパイラだけが扱うように、`vize()` が置き換えます。`exclude` を指定した場合は plugin-vue が残り、Vize が対象外にしたファイルをコンパイルします。

### 3. いつもどおり Void を実行する

```bash
npx vite dev
npx vite build
```

ほかの変更は必要ありません。

- **workerd 上のサーバーレンダリング。** workerd には import できる `node_modules` がないため、Vue と `vue/server-renderer` をワーカーにバンドルします。
- **ルートの CSS とプリロード。** クライアントマニフェストのキーはソースパス（`pages/index.vue`）です。Void はこのキーでルートのスタイルシートを探し、サーバーレンダリングした HTML に追加します。
- **`vite dev` の HMR。** テンプレートを編集してもコンポーネントの状態は保たれます。ローダーの props もそのまま残ります。
- **ページの props。** `.server.ts` のローダーから import した `Props` を使う `defineProps<Props>()` は、plugin-vue と同じようにコンパイルされます。

## 型チェック

Void は、ルート、環境変数、バインディングの型を含む `.void/tsconfig.json` を生成します。プロジェクトの `tsconfig.json` はこれを継承します。生成ファイルを用意するために一度 `void prepare` を実行する（または開発サーバーを起動する）と、チェックできます。

```bash
npx void prepare
vize check
```

`vize check` は生成された `files` と `paths` を読み込みます。そのため `.vue` ファイルでも、次の型が `vue-tsc` と同じように付きます。

- `InferProps<typeof loader>` によるローダーの props
- `void/client` の `fetch`（ルートのパス、メソッド、レスポンスの型）
- `void/env` の `env` と、`void/kv` などのバインディングモジュール

## lint とフォーマット

`vize lint` と `vize fmt` はプロジェクトのルートからファイルを探し、`.gitignore` に書かれたものを除外します。Void の `.gitignore` には `dist`、`.void`、`.wrangler`、`.void-wrangler.jsonc` が含まれるため、生成ファイルは lint やフォーマットの対象になりません。

## エディター

[言語サーバー](../guide/language-server.md)も同じ `tsconfig.json` を使います。生成された型が存在するように、プロジェクトを開く前に `void prepare` を実行してください。

## テスト方法

Vite プラグインまたはネイティブコンパイラが変わるたびに、CI はバージョンを固定した Void 1.1.12 と `@void/vue` のアプリ（`tools/support/compat/void/fixtures/void-vue`）をビルドし、次の項目を確認します。

- 本番ビルドと、`vite preview` による workerd 上の SSR（ルートの CSS と KV バインディングを含む）
- 開発時の SSR、ハイドレーション、状態を保つ HMR
- `vize check` と `vue-tsc` が同じエラーを報告すること
- `vize lint` と `vize fmt --check`

Void 上の Nuxt（`nuxt.config.ts` で `voidPlugin()` を使う構成）は、このフィクスチャではまだ検証していません。その場合は [Nuxt 統合](./nuxt.md)を使ってください。
