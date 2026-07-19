---
title: 実験的なバンドラー統合
---

<!-- Generated translation; source: guide/unplugin.md -->

# 実験的なバンドラー統合

> **⚠️ 実験版:**`@vizejs/unplugin` および `@vizejs/rspack-plugin` はまだ不安定です。
> `@vizejs/vite-plugin` は現在も推奨され、最もテスト済みのバンドラー統合です。

Vize は、`rollup`、`webpack`、および `esbuild` 用の実験的な [unplugin](https://unplugin.unjs.io/) パッケージと、専用の `Rspack` パッケージを提供します。

- `@vizejs/unplugin` — `rollup` / `webpack` / `esbuild`
- `@vizejs/rspack-plugin` — `Rspack` のみ

RSpack は意図的に共有アンプラグイン パスを**通過しません**。
そのローダー チェーン、`experiments.css`、および HMR の動作には、Rspack 固有の処理が必要です。

## インストール

[Vite+ インストール ガイド](https://viteplus.dev/guide/install) から `vp` を一度インストールし、パッケージを追加します。

```bash
vp install @vizejs/unplugin
```

Rspackの場合：

```bash
vp install -D @vizejs/rspack-plugin @rspack/core
```

## ロールアップ

```javascript
// rollup.config.mjs
import vize from "@vizejs/unplugin/rollup";

export default {
  plugins: [vize()],
};
```

## ウェブパック

```javascript
// webpack.config.mjs
import Vize from "@vizejs/unplugin/webpack";

export default {
  plugins: [Vize()],
};
```

## エスビルド

```javascript
// build.mjs
import { build } from "esbuild";
import vize from "@vizejs/unplugin/esbuild";

await build({
  entryPoints: ["src/main.ts"],
  bundle: true,
  plugins: [vize()],
});
```

## Rspack

`@vizejs/unplugin` の代わりに専用の `@vizejs/rspack-plugin` パッケージを使用します。

```javascript
// rspack.config.mjs
import { VizePlugin } from "@vizejs/rspack-plugin";

export default {
  experiments: {
    css: true,
  },
  module: {
    rules: [
      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
      },
    ],
  },
  plugins: [new VizePlugin()],
};
```

Rspack 構成の詳細については、パッケージの README を参照してください。

## 注意事項

- 最も完全で最もよくテストされた動作が必要な場合は、引き続き Vite が推奨される統合です。
- Vite 外部の CSS モジュールとスタイル プリプロセッサはホスト バンドラーの CSS パイプラインに依存しており、変更される可能性が高くなります。
- バンドラーが Vue ランタイムを外部化するのではなくインライン化する場合は、通常の Vue コンパイル時機能フラグがそのバンドラーに対して設定されていることを確認してください。
- これらの統合を実験的なものとして扱い、ロールアウトする前に独自のアプリケーションに対して検証してください。
