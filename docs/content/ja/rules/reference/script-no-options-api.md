---
title: "script/no-options-api"
---

# `script/no-options-api`

Vapor で Options API を使用する箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

default export のオブジェクトに Options API の `data()` を宣言しており、このルールが禁止するコンポーネントオプションの形式です。

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

## 良い

状態を Vapor の `<script setup>` 内の Composition API `ref` に移し、Options API のオブジェクトと `data` オプションを取り除きます。

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [全ルール](../all.md)
