---
title: SSR ルール
---

# SSR ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。個別ページが現在の対応範囲を示す参照先です。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | [悪い例](./reference/ssr-no-browser-globals-in-ssr.md#悪い) · [良い例](./reference/ssr-no-browser-globals-in-ssr.md#良い) | SSR で実行されるコードのブラウザー専用グローバル参照を検出します。 |
| [`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | [悪い例](./reference/ssr-no-hydration-mismatch.md#悪い) · [良い例](./reference/ssr-no-hydration-mismatch.md#良い) | サーバーとクライアントで一致しないテンプレート値を検出します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
