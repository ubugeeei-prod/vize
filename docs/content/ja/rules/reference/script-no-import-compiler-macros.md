---
title: "script/no-import-compiler-macros"
description: "自動的に使える Vue コンパイラーマクロの import を検出します。"
---

# `script/no-import-compiler-macros`

自動的に使える Vue コンパイラーマクロの import を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`vue` から `defineProps` と `defineEmits` をインポートしていますが、これらは `<script setup>` で直接使えるコンパイラーマクロです。

```vue annotate="remove:2"
<script setup lang="ts">
import { defineProps, defineEmits } from "vue";
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

## 良い例

マクロのインポートを除き、型付きのマクロ呼び出しは保ちます。これらの宣言に実行時のインポートは不要です。

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
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
        "script/no-import-compiler-macros": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39)
