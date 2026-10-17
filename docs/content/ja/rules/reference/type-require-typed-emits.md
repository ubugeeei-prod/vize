---
title: "type/require-typed-emits"
---

# `type/require-typed-emits`

defineEmits に型定義を指定します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型情報を使う検査には、ネイティブの Corsa ランタイムと TypeScript プロジェクトが必要です。typeAware を有効にするだけでは、個別に有効化する必要があるルールは有効になりません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-emits": "warn"
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

配列だけの `defineEmits(["save"])` はイベント名を宣言するだけで、ペイロードの型が宣言されていません。

```vue
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

## 良い

`defineEmits<{ save: [] }>()` で、ペイロードを空のタプル型として宣言します。これにより、`save` イベントがペイロードの引数を受け取らないことを明示します。

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) · [全ルール](../all.md)
