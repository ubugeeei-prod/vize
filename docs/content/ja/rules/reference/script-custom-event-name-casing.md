---
title: "script/custom-event-name-casing"
description: "emit するカスタムイベント名を指定した形式に揃えます。"
---

# `script/custom-event-name-casing`

emit するカスタムイベント名を指定した形式に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

emit する文字列 `transitionStart` に大文字が含まれ、区切りなしの lowercase を使う既定の命名規則に違反しています。

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['transitionStart'])
emit('transitionStart')         // camelCase → report
</script>
```

## 良い例

宣言と呼び出しの両方を `transitionstart` にそろえ、プラットフォーム風の lowercase 規則を満たします。camelCase または kebab-case を明示的に設定することもできます。

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['transitionstart'])
emit('transitionstart')
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
        "script/custom-event-name-casing": "error"
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
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L62)
