---
title: "type/require-typed-emits"
description: "defineEmits に型定義を指定します。"
---

# `type/require-typed-emits`

defineEmits に型定義を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

配列だけの `defineEmits(["save"])` はイベント名を宣言するだけで、ペイロードの型が宣言されていません。

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

## 良い例

`defineEmits<{ save: [] }>()` で、ペイロードを空のタプル型として宣言します。これにより、`save` イベントがペイロードの引数を受け取らないことを明示します。

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-emits": "warn"
      },
      "typeAware": true
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54)
