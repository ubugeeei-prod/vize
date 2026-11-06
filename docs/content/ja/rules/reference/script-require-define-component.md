---
title: "script/require-define-component"
description: "TypeScript SFC の default export を defineComponent() で型付けします。"
---

# `script/require-define-component`

TypeScript SFC の default export を defineComponent() で型付けします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

TypeScript SFC が Options API のオブジェクトをそのまま export しているため、`defineComponent()` が提供するコンポーネント向けの文脈型を受けられません。

```vue annotate="remove:2,7"
<script lang="ts">
export default {
  props: { count: Number },
  mounted() {
    console.log(this.count)
  }
}
</script>
```

## 良い例

オブジェクトを `defineComponent()` で包み、props、emits、`this` などのコンポーネントオプションを Vue 向けの TypeScript 検査につなげます。

```vue annotate="add:2,3,8"
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { count: Number },
  mounted() {
    console.log(this.count)
  }
})
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-define-component": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_define_component.rs#L48)
