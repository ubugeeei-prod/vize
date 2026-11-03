---
title: "SSR ルール"
description: "サーバーレンダリングやハイドレーションで壊れる、または動作が変わるコードを検出します。"
---

# SSR ルール

サーバーレンダリングやハイドレーションで壊れる、または動作が変わるコードを検出します。

各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。

## 既定 (2)

既定のプリセット `happy-path` で追加されるルールです。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="ssr-no-browser-globals-in-ssr"></span>[`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | SSR で実行されるコードのブラウザー専用グローバル参照を検出します。 |  |
| <span id="ssr-no-hydration-mismatch"></span>[`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | サーバーとクライアントで一致しないテンプレート値を検出します。 |  |

[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)
