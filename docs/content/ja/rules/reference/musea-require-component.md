---
title: "musea/require-component"
description: "art ブロックの対象 component を解決できるようにします。"
---

# `musea/require-component`

art ブロックの対象 component を解決できるようにします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Musea の .art.vue ファイルの art / variant / style。

## 悪い例

art に title はありますが、プレビューする component の指定がありません。

```vue annotate="remove:1,2"
<art title="Button">
  <variant name="primary"><div /></variant>
</art>
```

## 良い例

variant で import した Button を描画し、art の対象を特定します。

```vue annotate="add:1,2,3,4,5,6"
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
      "musea/require-component": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Musea ルール](../musea.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11)
