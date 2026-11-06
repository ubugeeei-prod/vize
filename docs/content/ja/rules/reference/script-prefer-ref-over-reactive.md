---
title: "script/prefer-ref-over-reactive"
description: "状態管理に reactive() より ref() を使う方針を適用します。"
---

# `script/prefer-ref-over-reactive`

状態管理に reactive() より ref() を使う方針を適用します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

状態を `reactive` で作っており、ref を優先するこのルールの方針に合っていません。これはスタイルの推奨を示す例であり、reactive オブジェクト自体が不正という意味ではありません。

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
// reactive requires careful handling to avoid losing reactivity
const state = reactive({
  count: 0,
  name: 'foo'
})
</script>
```

## 良い例

スカラーとオブジェクトの状態をどちらも `ref` で作ります。関連するフィールドを個別の ref に分ける例も含め、推奨する状態の作成形式にそろえます。

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
// ref is more explicit and safer
const count = ref(0)
const name = ref('foo')

// For objects, ref still works
const user = ref({ name: 'foo', age: 20 })

// Or use multiple refs for related data
const userName = ref('foo')
const userAge = ref(20)
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-ref-over-reactive": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44)
