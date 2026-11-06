---
title: "script/no-options-api"
description: "Vapor で Options API を使用する箇所を検出します。"
---

# `script/no-options-api`

Vapor で Options API を使用する箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `opinionated` | なし | なし |

適用範囲：Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。

## 悪い例

default export のオブジェクトに Options API の `data()` を宣言しており、このルールが禁止するコンポーネントオプションの形式です。

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

## 良い例

状態を Vapor の `<script setup>` 内の Composition API `ref` に移し、Options API のオブジェクトと `data` オプションを取り除きます。

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-options-api": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43)
