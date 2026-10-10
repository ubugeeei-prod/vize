---
title: 実験的なバンドラー統合
---

<!-- Reviewed translation; source: guide/unplugin.md -->

# 実験的なバンドラー統合

> **⚠️ 実験版:** `@vizejs/unplugin` および `@vizejs/rspack-plugin` はまだ不安定です。
> 通常は、検証範囲が最も広い `@vizejs/vite-plugin` を使ってください。

Vize は、`rollup`、`webpack`、および `esbuild` 用の実験的な [unplugin](https://unplugin.unjs.io/) パッケージと、専用の `Rspack` パッケージを提供します。

- `@vizejs/unplugin` — `rollup` / `webpack` / `esbuild`
- `@vizejs/rspack-plugin` — `Rspack` のみ

Rspack では、共有の unplugin 実装を使わず、専用パッケージを使います。
ローダー、`experiments.css`、HMR に Rspack 固有の対応が必要なためです。

## インストール

[Vite+ インストール ガイド](https://viteplus.dev/guide/install) から `vp` を一度インストールし、パッケージを追加します。

```bash
vp install @vizejs/unplugin
```

Rspackの場合：

```bash
vp install -D @vizejs/rspack-plugin @rspack/core
```

<span id="ロールアップ"></span>

## Rollup

```javascript
// rollup.config.mjs
import vize from "@vizejs/unplugin/rollup";

export default {
  plugins: [vize()],
};
```

<span id="ウェブパック"></span>

## webpack

```javascript
// webpack.config.mjs
import Vize from "@vizejs/unplugin/webpack";

export default {
  plugins: [Vize()],
};
```

<span id="エスビルド"></span>

## esbuild

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

Rspack の設定の詳細については、パッケージの README を参照してください。

## 注意事項

- 対応範囲と検証実績を重視する場合は、Vite 統合を使ってください。
- Vite 以外での CSS Modules とスタイルのプリプロセッサ処理は、バンドラー側の CSS 処理に依存します。動作が変わる可能性があります。
- バンドラーが Vue ランタイムを外部化するのではなくインライン化する場合は、通常の Vue コンパイル時機能フラグがそのバンドラーに対して設定されていることを確認してください。
- これらの統合を実験的なものとして扱い、ロールアウトする前に独自のアプリケーションに対して検証してください。
