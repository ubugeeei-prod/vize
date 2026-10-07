---
title: 設定
description: Vite+ は vite.config.ts、単独コマンドは vize.config.ts で設定する。
---

# 設定

Vite+ プロジェクトでは、統合 helper を使って **`vite.config.ts`** に Vize の設定をまとめます。
必要な項目だけ追加してください。デフォルトで利用する場合、別の設定ファイルは不要です。

## Vite+ の設定

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: { sourceMap: true },
  lint: { vize: { preset: "essential" } },
  fmt: { vize: { printWidth: 100 } },
  typecheck: { strict: true },
});
```

設定は、その設定を使うツールごとに分けます。

| 設定場所 | 対象 | 実行 |
| --- | --- | --- |
| `compiler` | Vue のコンパイル | `vp dev`, `vp build` |
| `lint.vize` | Vize の Vue ルール | `vp run lint` |
| その他の `lint` | Oxlint | `vp run lint` |
| `fmt.vize` | Vize の Vue フォーマット | `vp run fmt:check` |
| その他の `fmt` | Oxfmt | `vp run fmt:check` |
| `typecheck` | ネイティブ Vue 型チェック | `vp run typecheck` |
| `pack.vize` | Vue ライブラリの型宣言 | `vp run pack` |

`vp run check` でまとめて検査できます。組み込みの `vp check`・`vp lint`・`vp fmt` は
Vite+ 自身の動作を維持するため、Vize には生成されたタスクを使います。
既存スクリプトがある場合、生成タスクは `vize:<名前>` になります。
[タスク名と統合オプション](./vite-plus.md#tasks)を参照してください。

## ルールを1つ変更する

Vize のルールは `lint.vize.rules`、Oxlint のルールは `lint.rules` に設定します。

```ts
export default defineConfig({
  lint: {
    vize: { rules: { "vue/no-v-html": "error" } },
    rules: { "no-debugger": "error" },
  },
});
```

[ルールと具体例](../rules/all.md)から選べます。この統合では Vize と Oxlint が一緒に動作するため、
`oxlint-plugin-vize` を別途登録する必要はありません。

### Lint Rule Options

ルールごとの設定値と具体例は [ルール オプション](../rules/options.md)を参照してください。

## 導入する機能を選ぶ

`compiler`・`typecheck`・`lint.vize`・`fmt.vize` を `false` にすると、その機能を無効化できます。
例えば `compiler: false` なら既存の Vue コンパイラプラグインを使い続けます。
その他の Vite+ 設定も、同じ `vite.config.ts` に残してください。

Vue ファイルは Vize、その他は Oxfmt がフォーマットします。
`fmt.ignorePatterns` は両方に適用されます。
[担当範囲と重複の扱い](./vite-plus.md#lint-and-formatter-ownership)を参照してください。

## 通常の Vite

`vite.config.ts` の plugin オプションで設定します。

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize({ sourceMap: true })],
});
```

[Vite プラグインのオプション](./vite-plugin.md#compiler-options)を参照してください。
CLI や LSP と設定を共有する場合に限り、単独の共有設定も利用できます。

<span id="standalone-cli"></span>

## 単独 CLI

単独コマンドや設定 helper を使う場合は `vize` をインストールします。

```bash
vp install -D vize
vp exec vize check
```

設定場所は **`vize.config.ts`** です。

```ts
import { defineConfig } from "vize";

export default defineConfig({
  linter: { preset: "essential" },
  formatter: { printWidth: 100 },
  typeChecker: { strict: true },
});
```

単独設定の `linter`・`formatter`・`typeChecker` は、Vite+ の `lint.vize`・`fmt.vize`・`typecheck`
とは名前が異なります。[CLI ガイド](./cli.md)で実行方法を確認してください。

## 詳細リファレンス

[単独 CLI の設定リファレンス](./configuration-reference.md)に設定ファイルの検索・優先順位、
JSON/PKL、スコープ別の設定、全コンパイラオプション、テンプレート構文、Vue の型解決、
LSP・Musea の設定を残しています。型宣言やエディター設定は [Vite+ 統合](./vite-plus.md)を参照してください。

実験的なコンパイラ機能は明示的に有効化します。[Experimentals](./experimentals.md)で
対応する項目と現在の範囲を確認してください。
