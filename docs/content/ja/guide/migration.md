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
| `vue-tsc` の script | `vp run typecheck` | ネイティブ Vue 型チェック | 同じ統合パッケージ |
| Vue の lint script | `vp run lint` | Vize の Vue ルールと Oxlint | 同じ統合パッケージ |
| Vue の format script | `vp run fmt` | Vue は Vize、その他は Oxfmt | 同じ統合パッケージ |

最後の3つは [Vite+ の設定ヘルパー](./vite-plus.md) が生成するカスタムタスクです。
組み込みの `vp lint`・`vp fmt`・`vp check` とは別に実行します。

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

本番ビルドと検査を実行し、結果を比較します。

```bash
vp build
vp run typecheck
vp run lint
vp run fmt:check
vp run check
```

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

設定ヘルパーを使う Vite+ プロジェクトでは、次のようにスクリプトを変更するとネイティブタスクを利用できます。
変更は1つずつ確認してください。

```json annotate="remove:1,3,5;add:2,4,6"
  "typecheck": "vue-tsc --noEmit",
  "typecheck": "vp run vize:typecheck",
  "lint": "eslint src",
  "lint": "vp run vize:lint",
  "format": "prettier --write src",
  "format": "vp run fmt"
```

既存の `typecheck`・`lint` スクリプトがあると、設定ヘルパーは `vize:typecheck`・`vize:lint` を生成します。
**実際の置き換えでは、その衝突しない名前を使ってください**。

```json
{
  "scripts": {
    "typecheck": "vp run vize:typecheck",
    "lint": "vp run vize:lint",
    "format": "vp run fmt"
  }
}
```

対応するルールのない独自 ESLint 設定は残し、型診断とフォーマットの差分を確認してから CI を変更します。
フォーマットはファイルを書き換えるため、先に `vp run fmt:check` で確認してください。

## 単独設定を Vite+ に移す

CLI とエディターも `vite.config.*` のトップレベルの `vize` 設定を共有できます。
対応範囲とリリース状況は[設定ガイド](./configuration.md)を参照してください。
Vite+ の統合タスク用の設定は、ツールごとの設定項目に移します。

```ts annotate="remove:1,5,6,7;add:2,8,9,10"
import { defineConfig } from "vize";
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  linter: { preset: "essential" },
  formatter: { printWidth: 100 },
  typeChecker: { strict: true },
  lint: { vize: { preset: "essential" } },
  fmt: { vize: { printWidth: 100 } },
  typecheck: { strict: true },
});
```

import だけでなく設定も移します。項目名が異なる点に注意してください。
既存の専用設定は、その形式や優先順位が必要な場合に維持します。
[設定ガイド](./configuration.md)に統合ごとの設定場所をまとめています。
