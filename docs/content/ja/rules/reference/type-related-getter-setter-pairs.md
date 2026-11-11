---
title: "type/related-getter-setter-pairs"
description: "対になる getter と setter の型が互換であることを必須にします。"
---

# `type/related-getter-setter-pairs`

対になる getter と setter の型が互換であることを必須にします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

getter は string を返し、setter は number を受け取るため、対の型が互換ではありません。

```vue annotate="remove:4"
<script setup lang="ts">
interface Box {
  get size(): string;
  set size(value: number);
}
</script>
```

## 良い例

getter と setter はどちらも string を使います。

```vue annotate="add:4"
<script setup lang="ts">
interface Box {
  get size(): string;
  set size(value: string);
}
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/related-getter-setter-pairs": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/related_getter_setter_pairs.rs#L5)
