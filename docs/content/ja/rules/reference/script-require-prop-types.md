---
title: "script/require-prop-types"
description: "各 prop の型を宣言します。"
---

# `script/require-prop-types`

各 prop の型を宣言します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

配列の要素は `status` という名前だけを宣言しています。値を `null` にした prop と、空の設定オブジェクトを指定した prop にも、実行時の型宣言がありません。

```vue annotate="remove:3,4,5,6,8,9"
<script lang="ts">
export default {
  props: ['status']            // array form: no types
}

export default {
  props: {
    status: null,              // no type
    other: {}                  // empty descriptor: no type
  }
}
</script>
```

## 良い例

`status: String` でコンストラクターを直接指定し、`other` の設定オブジェクトに `type: Number` を付けます。両方の props に型が宣言されます。

```vue annotate="add:4,5"
<script lang="ts">
export default {
  props: {
    status: String,
    other: { type: Number, default: 0 }
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
      "script/require-prop-types": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L62)
