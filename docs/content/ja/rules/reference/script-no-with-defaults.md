---
title: "script/no-with-defaults"
description: "Vue 3.5 以降の props 分割代入の既定値を勧めます。"
---

# `script/no-with-defaults`

Vue 3.5 以降の props 分割代入の既定値を勧めます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

型付きの props 宣言を `withDefaults` で包んで `count` と `name` の既定値を指定しており、ここで推奨する Vue 3.5 以降の分割代入の既定値を使っていません。

```vue annotate="remove:2"
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

## 良い例

分割代入の変数に `count = 0` と `name = "Ada"` を直接指定し、`withDefaults` のラッパーを取り除きます。

```vue annotate="add:2"
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-with-defaults": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41)
