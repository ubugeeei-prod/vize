---
title: "script/return-in-computed-property"
description: "computed の getter に値を返す return を用意します。"
---

# `script/return-in-computed-property`

computed の getter に値を返す return を用意します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

ブロック形式の computed getter が `1 + 2` を計算するだけで返さず、computed の値が undefined になります。

```vue annotate="remove:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { 1 + 2; });
</script>
```

## 良い例

`return 1 + 2` で計算結果をゲッターの戻り値にします。式文があるだけでは、このルールの条件を満たしません。ゲッター本体に、値を返す return が必要です。

```vue annotate="add:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { return 1 + 2; });
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/return-in-computed-property": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31)
