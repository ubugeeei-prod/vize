---
title: HTML ルール
---

# HTML ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。全ルールの一覧に例と現在の対応範囲を同じページでまとめています。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`html/deprecated-attr`](./all.md#html-deprecated-attr) | [悪い例](./all.md#html-deprecated-attr-bad) · [良い例](./all.md#html-deprecated-attr-good) | 非推奨の HTML 属性を検出します。 |
| [`html/deprecated-element`](./all.md#html-deprecated-element) | [悪い例](./all.md#html-deprecated-element-bad) · [良い例](./all.md#html-deprecated-element-good) | 非推奨の HTML 要素を検出します。 |
| [`html/id-duplication`](./all.md#html-id-duplication) | [悪い例](./all.md#html-id-duplication-bad) · [良い例](./all.md#html-id-duplication-good) | 同じテンプレート内の ID 重複を検出します。 |
| [`html/no-consecutive-br`](./all.md#html-no-consecutive-br) | [悪い例](./all.md#html-no-consecutive-br-bad) · [良い例](./all.md#html-no-consecutive-br-good) | 連続する br 要素による余白指定を検出します。 |
| [`html/no-dupe-style-properties`](./all.md#html-no-dupe-style-properties) | [悪い例](./all.md#html-no-dupe-style-properties-bad) · [良い例](./all.md#html-no-dupe-style-properties-good) | 静的 style 属性内のプロパティ重複を検出します。 |
| [`html/no-duplicate-class`](./all.md#html-no-duplicate-class) | [悪い例](./all.md#html-no-duplicate-class-bad) · [良い例](./all.md#html-no-duplicate-class-good) | 静的 class 属性内のクラス名重複を検出します。 |
| [`html/no-duplicate-dt`](./all.md#html-no-duplicate-dt) | [悪い例](./all.md#html-no-duplicate-dt-bad) · [良い例](./all.md#html-no-duplicate-dt-good) | dl 内の dt の名前重複を検出します。 |
| [`html/no-empty-palpable-content`](./all.md#html-no-empty-palpable-content) | [悪い例](./all.md#html-no-empty-palpable-content-bad) · [良い例](./all.md#html-no-empty-palpable-content-good) | 可視コンテンツを期待する要素が空の場合に検出します。 |
| [`html/require-datetime`](./all.md#html-require-datetime) | [悪い例](./all.md#html-require-datetime-bad) · [良い例](./all.md#html-require-datetime-good) | time 要素に機械可読の datetime を指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)

子の要素を合成した入れ子は [html/cross-component-nesting](./project/html-cross-component-nesting.md) の対象です。
