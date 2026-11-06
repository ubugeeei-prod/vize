---
title: "musea/valid-variant"
description: "variant ブロックに name を指定します。"
---

# `musea/valid-variant`

variant ブロックに name を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | なし |

適用範囲：Musea の .art.vue ファイルの art / variant / style。

## 悪い例

プレビューを識別する variant の name がありません。

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

## 良い例

variant に primary の name を指定します。

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
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
      "musea/valid-variant": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Musea と CSS のルール](../musea-and-css.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8)
