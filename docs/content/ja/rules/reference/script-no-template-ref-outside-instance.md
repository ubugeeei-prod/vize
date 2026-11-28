---
title: "script/no-template-ref-outside-instance"
description: "コンポーネントのインスタンスがない場所での useTemplateRef() 呼び出しを禁止します。"
---

# `script/no-template-ref-outside-instance`

コンポーネントのインスタンスがない場所での useTemplateRef() 呼び出しを禁止します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`focus()` はボタンのクリックハンドラーとしてだけ実行され、`setTimeout` のコールバックは setup が終わった後に実行されます。どちらにも現在のコンポーネントインスタンスがないため、作成した ref は設定されません。

```vue annotate="remove:2,4,6"
<script setup lang="ts">
import { useTemplateRef } from 'vue'
function focus() {
  useTemplateRef<HTMLInputElement>('input').value?.focus()
}
setTimeout(() => useTemplateRef('input'))
</script>
<template>
  <input ref="input" />
  <button @click="focus">Focus</button>
</template>
```

## 良い例

`useTemplateRef('input')` を setup で一度だけ呼びます。クリックハンドラーと `onMounted` のコールバックは `input.value` を読むだけです。

```vue annotate="add:2,3,5,7"
<script setup lang="ts">
import { onMounted, useTemplateRef } from 'vue'
const input = useTemplateRef<HTMLInputElement>('input')
function focus() {
  input.value?.focus()
}
onMounted(() => focus())
</script>
<template>
  <input ref="input" />
  <button @click="focus">Focus</button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-template-ref-outside-instance": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_template_ref_outside_instance.rs#L78)
