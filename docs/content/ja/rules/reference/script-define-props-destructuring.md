---
title: "script/define-props-destructuring"
description: "defineProps の分割代入スタイルを指定した方針に揃えます。"
---

# `script/define-props-destructuring`

defineProps の分割代入スタイルを指定した方針に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`defineProps` の結果を分割代入せず、単一の `props` 変数に代入しており、既定の分割代入の推奨に従っていません。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

## 良い例

オブジェクトパターンで `foo` と `bar` を直接取り出し、省略可能な `bar` に既定値を付けます。Vue 3.5 以降のリアクティブな props 分割代入を前提とし、設定を `never` にした場合は逆の形式を推奨します。

```vue annotate="add:2"
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
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
        "script/define-props-destructuring": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29)
