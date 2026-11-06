---
title: "vue/valid-model-definition"
description: "Vue 2 の model オプションに有効なキーを指定します。"
---

# `vue/valid-model-definition`

Vue 2 の model オプションに有効なキーを指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

Vue 2 の `model` オプションを検査する、個別に有効化するルールです。静的なキーは `prop` と `event` のみ許可し、それぞれの値は検証しません。Vue 3 ではこのオプションに代わる新しいコンポーネントの `v-model` 規約を使います。

## 悪い例

`events` は Vue 2 のコンポーネントの `model` オプションで使えるキーではありません。

```vue annotate="remove:2"
<script lang="ts">
export default { model: { prop: "value", events: "input" } };
</script>
```

## 良い例

対応する単数形の `event` と `prop` を使い、Vue 2 の model の規約を指定します。

```vue annotate="add:2"
<script lang="ts">
export default { model: { prop: "value", event: "input" } };
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-model-definition": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_model_definition.rs#L21)
