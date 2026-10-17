---
title: "type/strict-boolean-expressions"
---

# `type/strict-boolean-expressions`

script とテンプレートの条件式で安全な真偽判定を使います。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: [型付きオプションと既定値](../options.md)を参照してください。

typeAware とこのルールを明示的に有効にします。既定では null または undefined を含み得る数値は許可されず、どちらも含まない数値は許可されます。

型情報を使う検査には、ネイティブの Corsa ランタイムと TypeScript プロジェクトが必要です。typeAware を有効にするだけでは、個別に有効化する必要があるルールは有効になりません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "type/strict-boolean-expressions": "warn"
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

`if (count)` は、undefined を含み得る数値をそのまま真偽判定に使い、明示的な真偽値の比較をしていません。数値のゼロと値の未指定を、どちらも偽として扱ってしまいます。

```vue
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

## 良い

`count !== undefined && count > 0` で、値が undefined でないことと正の数であることを別々に検査します。省略可能な値を絞り込んでから、真偽値を返す条件式で判定します。

```vue
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) · [全ルール](../all.md)
