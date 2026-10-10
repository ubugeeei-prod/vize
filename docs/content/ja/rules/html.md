---
title: "HTML ルール"
---

# HTML ルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`html/deprecated-attr`](https://vizejs.dev/ja/rules/html.html#html-deprecated-attr) | [悪い](https://vizejs.dev/ja/rules/html.html#html-deprecated-attr-bad) · [良い](https://vizejs.dev/ja/rules/html.html#html-deprecated-attr-good) | 非推奨の HTML 属性を検出します。 |
| [`html/deprecated-element`](https://vizejs.dev/ja/rules/html.html#html-deprecated-element) | [悪い](https://vizejs.dev/ja/rules/html.html#html-deprecated-element-bad) · [良い](https://vizejs.dev/ja/rules/html.html#html-deprecated-element-good) | 非推奨の HTML 要素を検出します。 |
| [`html/id-duplication`](https://vizejs.dev/ja/rules/html.html#html-id-duplication) | [悪い](https://vizejs.dev/ja/rules/html.html#html-id-duplication-bad) · [良い](https://vizejs.dev/ja/rules/html.html#html-id-duplication-good) | 同じテンプレート内の ID 重複を検出します。 |
| [`html/no-consecutive-br`](https://vizejs.dev/ja/rules/html.html#html-no-consecutive-br) | [悪い](https://vizejs.dev/ja/rules/html.html#html-no-consecutive-br-bad) · [良い](https://vizejs.dev/ja/rules/html.html#html-no-consecutive-br-good) | 連続する br 要素による余白指定を検出します。 |
| [`html/no-dupe-style-properties`](https://vizejs.dev/ja/rules/html.html#html-no-dupe-style-properties) | [悪い](https://vizejs.dev/ja/rules/html.html#html-no-dupe-style-properties-bad) · [良い](https://vizejs.dev/ja/rules/html.html#html-no-dupe-style-properties-good) | 静的 style 属性内のプロパティ重複を検出します。 |
| [`html/no-duplicate-class`](https://vizejs.dev/ja/rules/html.html#html-no-duplicate-class) | [悪い](https://vizejs.dev/ja/rules/html.html#html-no-duplicate-class-bad) · [良い](https://vizejs.dev/ja/rules/html.html#html-no-duplicate-class-good) | 静的 class 属性内のクラス名重複を検出します。 |
| [`html/no-duplicate-dt`](https://vizejs.dev/ja/rules/html.html#html-no-duplicate-dt) | [悪い](https://vizejs.dev/ja/rules/html.html#html-no-duplicate-dt-bad) · [良い](https://vizejs.dev/ja/rules/html.html#html-no-duplicate-dt-good) | dl 内の dt の名前重複を検出します。 |
| [`html/no-empty-palpable-content`](https://vizejs.dev/ja/rules/html.html#html-no-empty-palpable-content) | [悪い](https://vizejs.dev/ja/rules/html.html#html-no-empty-palpable-content-bad) · [良い](https://vizejs.dev/ja/rules/html.html#html-no-empty-palpable-content-good) | 可視コンテンツを期待する要素が空の場合に検出します。 |
| [`html/require-datetime`](https://vizejs.dev/ja/rules/html.html#html-require-datetime) | [悪い](https://vizejs.dev/ja/rules/html.html#html-require-datetime-bad) · [良い](https://vizejs.dev/ja/rules/html.html#html-require-datetime-good) | time 要素に機械可読の datetime を指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
