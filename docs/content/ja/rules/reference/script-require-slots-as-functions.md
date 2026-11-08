---
title: "script/require-slots-as-functions"
description: "Vue 3 の $slots プロパティを関数として呼び出します。"
---

# `script/require-slots-as-functions`

Vue 3 の $slots プロパティを関数として呼び出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`render` が `this.$slots.default` を呼び出さずに返しています。これは Vue 2 の vnode 配列です。

```vue annotate="remove:4"
<script lang="ts">
export default {
  render() {
    return [this.$slots.default]
  }
}
</script>
```

## 良い例

`render` は `this.$slots.default()` を呼び出し、その vnode を使います。

```vue annotate="add:4"
<script lang="ts">
export default {
  render() {
    return this.$slots.default()
  }
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
      "script/require-slots-as-functions": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_slots_as_functions.rs#L43)
