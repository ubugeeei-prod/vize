---
title: "type/require-typed-props"
---

# `type/require-typed-props`

defineProps に型定義を指定します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-props": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

## 悪い

配列だけの `defineProps(["title"])` は `title` の名前だけを宣言し、型を指定していません。

```vue
<script setup lang="ts">
defineProps(["title"]);
</script>
```

## 良い

`defineProps<{ title: string }>()` で、名前だけの実行時宣言に代えて `title` の string 型を明示します。

```vue
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) · [全ルール](../all.md)
