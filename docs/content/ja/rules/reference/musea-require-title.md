---
title: "musea/require-title"
description: "art ブロックの表示 title を解決できるようにします。"
---

# `musea/require-title`

art ブロックの表示 title を解決できるようにします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | なし |

適用範囲：Musea の .art.vue ファイルの art / variant / style。

## 悪い例

art に明示 metadata も推論できる component title もありません。

```vue annotate="remove:2"
<art>
  <variant name="primary"><div /></variant>
</art>
```

## 良い例

import した Button を対象にすることで表示 title を推論します。

```vue annotate="add:1,2,3,4,6"
<script setup>
import Button from "./Button.vue";
</script>

<art>
  <variant name="primary"><Button /></variant>
</art>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/require-title": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Musea ルール](../musea.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L32)
