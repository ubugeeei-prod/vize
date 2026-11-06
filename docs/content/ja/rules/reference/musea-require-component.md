---
title: "musea/require-component"
description: "art ブロックに対象の component を指定します。"
---

# `musea/require-component`

art ブロックに対象の component を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Musea の .art.vue ファイルの art / variant / style。

## 悪い例

art に title はありますが、プレビューする component の指定がありません。

```vue annotate="remove:1"
<art title="Button">
  <variant name="primary" />
</art>
```

## 良い例

defineArt で ./Button.vue を art の component として指定します。

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

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "musea/require-component": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Musea ルール](../musea.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11)
