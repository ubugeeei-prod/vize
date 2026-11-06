---
title: "musea/no-empty-variant"
description: "内容のない variant ブロックを検出します。"
---

# `musea/no-empty-variant`

内容のない variant ブロックを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Musea の .art.vue ファイルの art / variant / style。

## 悪い例

name を primary にしているものの、variant の中身が空で、プレビューする内容がありません。

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## 良い例

variant の中に tone="primary" の Button を置き、Save を表示します。

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/no-empty-variant": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Musea ルール](../musea.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8)
