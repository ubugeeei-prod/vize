---
title: "script/require-typed-object-prop"
description: "Object / Array の prop に具体的な型を指定します。"
---

# `script/require-typed-object-prop`

Object / Array の prop に具体的な型を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`Object` と `Array` コンストラクターだけでは、実行時にオブジェクトか配列かを示すにとどまり、`user` の構造や `items` の要素の型を表す具体的な静的型がありません。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

## 良い例

`PropType<User>` と `PropType<User[]>` を付け、実行時のコンストラクターを保ったままオブジェクトと要素の型を指定します。

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
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
        "script/require-typed-object-prop": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63)
