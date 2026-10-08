---
title: "type/no-floating-promises"
---

# `type/no-floating-promises`

処理しないまま放置された Promise を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
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
        "type/no-floating-promises": "warn"
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

async の `save` は Promise を返しますが、単独の `save()` 呼び出しが await も return もせず、意図的に捨てる指定もありません。

```vue
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

## 良い

`void save()` で実行結果を意図的に捨てる指定をし、このルールの条件を満たします。結果を破棄する明示的な印であり、rejection を処理するものではありません。

```vue
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) · [全ルール](../all.md)
