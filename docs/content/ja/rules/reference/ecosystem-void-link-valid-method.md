---
title: "ecosystem/void-link-valid-method"
description: "Void Vue の Link に有効な静的 method を指定します。"
---

# `ecosystem/void-link-valid-method`

Void Vue の Link に有効な静的 method を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

DELETE の操作に、ページ移動のリクエスト向けの prefetch を指定しています。

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

## 良い例

prefetch を取り除き、DELETE のリクエストを事前取得しない形にします。

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/void-link-valid-method": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [エコシステムと Nuxt のルール](../ecosystem.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14)
