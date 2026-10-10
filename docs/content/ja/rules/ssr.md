---
title: "SSR ルール"
---

# SSR ルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](https://vizejs.dev/ja/rules/ssr.html#ssr-no-browser-globals-in-ssr) | [悪い](https://vizejs.dev/ja/rules/ssr.html#ssr-no-browser-globals-in-ssr-bad) · [良い](https://vizejs.dev/ja/rules/ssr.html#ssr-no-browser-globals-in-ssr-good) | SSR で実行されるコードのブラウザー専用グローバル参照を検出します。 |
| [`ssr/no-hydration-mismatch`](https://vizejs.dev/ja/rules/ssr.html#ssr-no-hydration-mismatch) | [悪い](https://vizejs.dev/ja/rules/ssr.html#ssr-no-hydration-mismatch-bad) · [良い](https://vizejs.dev/ja/rules/ssr.html#ssr-no-hydration-mismatch-good) | サーバーとクライアントで一致しないテンプレート値を検出します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
