---
title: "a11y/role-img-has-name"
description: "role=\"img\" の要素にアクセシブルな名前を要求します。"
---

# `a11y/role-img-has-name`

role="img" の要素にアクセシブルな名前を要求します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

空の `alt` のあとに `role="img"` を指定しているため、名前のない画像として公開されます。

```vue annotate="remove:2"
<template>
  <img src="/a.png" alt="" role="img" />
</template>
```

## 良い例

`alt="Network diagram"` が画像の名前になるため、`role="img"` にアクセシブルな名前があります。

```vue annotate="add:2"
<template>
  <img src="/a.png" alt="Network diagram" role="img" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/role-img-has-name": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_img_has_name.rs#L14)
