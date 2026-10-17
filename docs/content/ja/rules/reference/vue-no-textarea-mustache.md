---
title: "vue/no-textarea-mustache"
---

# `vue/no-textarea-mustache`

textarea 内の mustache を検出し、v-model の使用を勧めます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-textarea-mustache": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

textarea の値をバインドせず、要素内のテキスト補間に message を記述しています。

```vue
<template>
  <textarea>{{ message }}</textarea>
</template>
```

## 良い

v-model で、編集可能な textarea の値と message を関連付けます。

```vue
<template>
  <textarea v-model="message"></textarea>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [全ルール](../all.md)
