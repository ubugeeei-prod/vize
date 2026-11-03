---
title: "a11y/media-has-caption"
description: "音声・動画に字幕を用意します。"
---

# `a11y/media-has-caption`

音声・動画に字幕を用意します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

動画に再生操作はありますが、字幕の track がありません。

```vue annotate="remove:2"
<template>
  <video src="/demo.mp4" controls />
</template>
```

## 良い例

同じ動画に `kind="captions"` の track を追加して英語の字幕を指定します。

```vue annotate="add:2,3,4"
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
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
        "a11y/media-has-caption": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30)
