---
title: "vue/no-mutating-props"
description: "親から受け取った prop を子コンポーネントで変更する箇所を検出します。"
---

# `vue/no-mutating-props`

親から受け取った prop を子コンポーネントで変更する箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

props.count の加算で、親から受け取った値を直接変更しています。

```vue annotate="remove:4"
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

## 良い例

次の値を update:count で通知し、prop の変更は親が行う形にします。

```vue annotate="add:3,5,6,7"
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
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
        "vue/no-mutating-props": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L36)
