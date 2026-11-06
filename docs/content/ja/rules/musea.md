---
title: "Musea ルール"
description: "Musea の `.art.vue` ストーリー、variant、デザイントークンの使い方を検査します。"
---

# Musea ルール

Musea の `.art.vue` ストーリー、variant、デザイントークンの使い方を検査します。

各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。

## 個別に有効化 (6)

どのプリセットにも含まれません。`vue.linter.rules` でルール名を指定して有効にします。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="musea-no-empty-variant"></span>[`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | 内容のない variant ブロックを検出します。 |  |
| <span id="musea-prefer-design-tokens"></span>[`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | CSS に直接記述した値が登録済みのデザイントークンの値に一致する場合、対応する CSS 変数で表現します。 |  |
| <span id="musea-require-component"></span>[`musea/require-component`](./reference/musea-require-component.md) | art ブロックに対象の component を指定します。 |  |
| <span id="musea-require-title"></span>[`musea/require-title`](./reference/musea-require-title.md) | art ブロックに title を指定します。 |  |
| <span id="musea-unique-variant-names"></span>[`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | 同じ Art ファイル内の variant 名を一意にします。 |  |
| <span id="musea-valid-variant"></span>[`musea/valid-variant`](./reference/musea-valid-variant.md) | variant ブロックに name を指定します。 |  |

[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)
