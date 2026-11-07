---
title: "script/prefer-define-options"
description: "name / inheritAttrs だけの通常 script を defineOptions() にまとめます。"
---

# `script/prefer-define-options`

name / inheritAttrs だけの通常 script を defineOptions() にまとめます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

通常の script の処理が `name` と `inheritAttrs` だけを持つオブジェクトの export に限られ、`defineOptions` で表せるオプションだけを宣言しています。

```vue annotate="remove:2"
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

## 良い例

`data()` が Options API の処理を含むため、オプションの宣言だけを持つ script を対象とする、このルールの提案の範囲外です。この良い例は、通常の script を残せる例外を示しています。悪い例を直接移行する場合は `<script setup>` 内で `defineOptions({ name: 'MyComponent', inheritAttrs: false })` を使います。

```vue annotate="add:2,3,4,5,6"
<script lang="ts">
// Real options logic — keep the plain script.
export default {
  name: 'MyComponent',
  data() { return { count: 0 } },
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
      "script/prefer-define-options": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L44)
