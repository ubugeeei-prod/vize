---
title: "Vapor ルール"
description: "Vapor モードでコンパイルするコンポーネントのテンプレートと script の制約です。"
---

# Vapor ルール

Vapor モードでコンパイルするコンポーネントのテンプレートと script の制約です。

各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。

## 既定 (1)

既定のプリセット `happy-path` で追加されるルールです。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="vapor-no-vue-lifecycle-events"></span>[`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | Vapor では未対応の、要素ごとに指定する @vue:* ライフサイクルイベントを検出します。 |  |

## より厳しいプリセット (5)

`opinionated`・`nuxt`・`ecosystem` のいずれかにだけ含まれます。どれに含まれるかはプリセット列を参照してください。

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Vapor で null を返す getCurrentInstance() を検出します。 | `opinionated` |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Vapor で Options API を使用する箇所を検出します。 | `opinionated` |  |
| <span id="vapor-no-inline-template"></span>[`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | Vapor で削除済みの inline-template 属性を検出します。 | `nuxt`, `opinionated` |  |
| <span id="vapor-prefer-static-class"></span>[`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | 文字列リテラルの :class を静的 class に置き換えます。 | `nuxt`, `opinionated` | ✓ |
| <span id="vapor-require-vapor-attribute"></span>[`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | Vapor 向けの script setup に vapor 属性を付ける規約を示します。 | `nuxt`, `opinionated` |  |

## 個別に有効化 (1)

どのプリセットにも含まれません。`vue.linter.rules` でルール名を指定して有効にします。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 |  |

[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)
