---
title: SSR ルール
---

# SSR ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。全ルールの一覧に例と現在の対応範囲を同じページでまとめています。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](./all.md#ssr-no-browser-globals-in-ssr) | [悪い例](./all.md#ssr-no-browser-globals-in-ssr-bad) · [良い例](./all.md#ssr-no-browser-globals-in-ssr-good) | SSR で実行されるコードのブラウザー専用グローバル参照を検出します。 |
| [`ssr/no-hydration-mismatch`](./all.md#ssr-no-hydration-mismatch) | [悪い例](./all.md#ssr-no-hydration-mismatch-bad) · [良い例](./all.md#ssr-no-hydration-mismatch-good) | サーバーとクライアントで一致しないテンプレート値を検出します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
