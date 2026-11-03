---
title: "HTML ルール"
description: "テンプレートの HTML の妥当性と、意味に沿ったマークアップを検査します。"
---

# HTML ルール

テンプレートの HTML の妥当性と、意味に沿ったマークアップを検査します。

各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。

## 基本 (1)

`essential` プリセットに含まれる、正しさを守るルールです。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="html-id-duplication"></span>[`html/id-duplication`](./reference/html-id-duplication.md) | 同じテンプレート内の ID 重複を検出します。 |  |

## 既定 (7)

既定のプリセット `happy-path` で追加されるルールです。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="html-deprecated-attr"></span>[`html/deprecated-attr`](./reference/html-deprecated-attr.md) | 非推奨の HTML 属性を検出します。 |  |
| <span id="html-deprecated-element"></span>[`html/deprecated-element`](./reference/html-deprecated-element.md) | 非推奨の HTML 要素を検出します。 |  |
| <span id="html-no-consecutive-br"></span>[`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | 連続する br 要素による余白指定を検出します。 |  |
| <span id="html-no-duplicate-class"></span>[`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | 静的 class 属性内のクラス名重複を検出します。 |  |
| <span id="html-no-duplicate-dt"></span>[`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | dl 内の dt の名前重複を検出します。 |  |
| <span id="html-no-empty-palpable-content"></span>[`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | 可視コンテンツを期待する要素が空の場合に検出します。 |  |
| <span id="html-require-datetime"></span>[`html/require-datetime`](./reference/html-require-datetime.md) | time 要素に機械可読の datetime を指定します。 |  |

## より厳しいプリセット (1)

`opinionated`・`nuxt`・`ecosystem` のいずれかにだけ含まれます。どれに含まれるかはプリセット列を参照してください。

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="html-no-dupe-style-properties"></span>[`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | 静的 style 属性内のプロパティ重複を検出します。 | `nuxt`, `opinionated` |  |

[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)
