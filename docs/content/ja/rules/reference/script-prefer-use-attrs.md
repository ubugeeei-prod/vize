---
title: "script/prefer-use-attrs"
description: "setup の context.attrs を useAttrs() に置き換えます。"
---

# `script/prefer-use-attrs`

setup の context.attrs を useAttrs() に置き換えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`setup` がコンテキスト引数の分割代入で `attrs` を受け取っています。このルールは Composition API のヘルパーへの置換を求めます。

```vue annotate="remove:2"
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

## 良い例

setup 内で `useAttrs()` から `attrs` を取得し、第二引数に依存せず `attrs.class` の参照を保ちます。

```vue annotate="add:2,3"
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-use-attrs": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44)
