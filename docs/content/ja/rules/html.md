---
title: HTML ルール
---

# HTML ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。個別ページが現在の対応範囲を示す参照先です。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`html/deprecated-attr`](./reference/html-deprecated-attr.md) | [悪い例](./reference/html-deprecated-attr.md#悪い) · [良い例](./reference/html-deprecated-attr.md#良い) | 非推奨の HTML 属性を検出します。 |
| [`html/deprecated-element`](./reference/html-deprecated-element.md) | [悪い例](./reference/html-deprecated-element.md#悪い) · [良い例](./reference/html-deprecated-element.md#良い) | 非推奨の HTML 要素を検出します。 |
| [`html/id-duplication`](./reference/html-id-duplication.md) | [悪い例](./reference/html-id-duplication.md#悪い) · [良い例](./reference/html-id-duplication.md#良い) | 同じテンプレート内の ID 重複を検出します。 |
| [`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | [悪い例](./reference/html-no-consecutive-br.md#悪い) · [良い例](./reference/html-no-consecutive-br.md#良い) | 連続する br 要素による余白指定を検出します。 |
| [`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | [悪い例](./reference/html-no-dupe-style-properties.md#悪い) · [良い例](./reference/html-no-dupe-style-properties.md#良い) | 静的 style 属性内のプロパティ重複を検出します。 |
| [`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | [悪い例](./reference/html-no-duplicate-class.md#悪い) · [良い例](./reference/html-no-duplicate-class.md#良い) | 静的 class 属性内のクラス名重複を検出します。 |
| [`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | [悪い例](./reference/html-no-duplicate-dt.md#悪い) · [良い例](./reference/html-no-duplicate-dt.md#良い) | dl 内の dt の名前重複を検出します。 |
| [`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | [悪い例](./reference/html-no-empty-palpable-content.md#悪い) · [良い例](./reference/html-no-empty-palpable-content.md#良い) | 可視コンテンツを期待する要素が空の場合に検出します。 |
| [`html/require-datetime`](./reference/html-require-datetime.md) | [悪い例](./reference/html-require-datetime.md#悪い) · [良い例](./reference/html-require-datetime.md#良い) | time 要素に機械可読の datetime を指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)

子の要素を合成した入れ子は [html/cross-component-nesting](./project/html-cross-component-nesting.md) の対象です。
