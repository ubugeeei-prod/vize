---
title: "script/require-prop-type-constructor"
description: "prop の type に文字列ではなくコンストラクターを指定します。"
---

# `script/require-prop-type-constructor`

prop の type に文字列ではなくコンストラクターを指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

props の実行時の型に文字列 `"String"` と `"Number"` を使い、コンストラクターの配列にも文字列を入れています。これらの文字列はコンストラクター関数ではありません。

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
  props: {
    // The type should be the `String` constructor, not the string "String".
    name: "String",
    age: { type: "Number" },
    id: { type: ["String", "Number"] }
  }
}
</script>
```

## 良い例

型を実際の `String` と `Number` の識別子にし、共用型の配列も `[String, Number]` に変えます。

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
  props: {
    name: String,
    age: { type: Number },
    id: { type: [String, Number] }
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
      "script/require-prop-type-constructor": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L66)
