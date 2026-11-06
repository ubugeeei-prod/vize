---
title: "petite-vue ルール"
description: "petite-vue を使う HTML ページのディレクティブとスコープを検査します。"
---

# petite-vue ルール

petite-vue を使う HTML ページのディレクティブとスコープを検査します。

各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。

## 個別に有効化 (3)

どのプリセットにも含まれません。`withVue` に渡す `linter.rules` でルール名を指定して有効にします。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="petite-vue-no-unsupported-directive"></span>[`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | petite-vue が対応しないディレクティブを検出します。 |  |
| <span id="petite-vue-valid-v-effect"></span>[`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | v-effect に空でない式を指定します。 |  |
| <span id="petite-vue-valid-v-scope"></span>[`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | v-scope に値を指定する場合は、オブジェクトリテラルにします。 |  |

[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)
