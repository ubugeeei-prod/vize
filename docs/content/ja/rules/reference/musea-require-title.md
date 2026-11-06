---
title: "musea/require-title"
description: "art ブロックに title を指定します。"
---

# `musea/require-title`

art ブロックに title を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | なし |

適用範囲：Musea の .art.vue ファイルの art / variant / style。

## 悪い例

art に Button.vue の指定はありますが、title がありません。

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

## 良い例

defineArt のオプションで title を Button にし、art ブロックのタイトルを指定します。

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
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

- [Musea と CSS のルール](../musea-and-css.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31)
