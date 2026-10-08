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

typeAware とルールを明示的に有効にします。既定では null を含み得る数値は許可されず、通常の数値は許可されます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
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

`if (count)` が nullable な数値の truthiness に依存し、明示的な真偽値の検査を使っていません。ゼロと未指定も同じ偽として扱います。

```vue
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

## 良い

`count !== undefined && count > 0` で存在と正の値を別々に検査し、省略可能な値を絞り込んだうえで明示的な真偽値の条件を作ります。

```vue
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) · [全ルール](../all.md)
