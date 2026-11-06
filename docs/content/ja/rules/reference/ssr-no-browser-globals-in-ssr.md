---
title: "ssr/no-browser-globals-in-ssr"
description: "SSR で実行されるコードのブラウザー専用グローバル参照を検出します。"
---

# `ssr/no-browser-globals-in-ssr`

SSR で実行されるコードのブラウザー専用グローバル参照を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

setup の実行時に `window.innerWidth` を直接読みますが、サーバー上のコンポーネント実行時には `window` が存在しません。

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

## 良い例

幅を ref(0) で初期化し、ブラウザー API の参照をクライアントで実行される `onMounted` に移します。これにより、SSR の setup ではブラウザー API を参照しません。

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ssr/no-browser-globals-in-ssr": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [SSR ルール](../ssr.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L157)
