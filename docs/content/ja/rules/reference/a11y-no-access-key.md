---
title: "a11y/no-access-key"
description: "環境のショートカットと衝突し得る accesskey を検出します。"
---

# `a11y/no-access-key`

環境のショートカットと衝突し得る accesskey を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`accesskey="s"` がブラウザーや支援技術のショートカットと競合する可能性があります。

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

## 良い例

accesskey を取り除き、通常の Save ボタンは残します。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L21)
