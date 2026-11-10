---
title: "script/valid-next-tick"
description: "nextTick() の完了を await、then、またはコールバックで扱います。"
---

# `script/valid-next-tick`

nextTick() の完了を await、then、またはコールバックで扱います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

インポートした `nextTick()` をコールバックなしの式文で呼び、返された Promise を無視しています。DOM 更新後まで待つ処理がありません。

```vue annotate="remove:3"
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

## 良い例

`await nextTick()` で Promise を使い、後続の setup 処理へ進む前に次の DOM 更新を明示的に待ちます。

```vue annotate="add:3"
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/valid-next-tick": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L52)
