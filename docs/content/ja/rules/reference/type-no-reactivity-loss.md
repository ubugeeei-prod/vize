---
title: "type/no-reactivity-loss"
---

# `type/no-reactivity-loss`

リアクティブな値を代入や呼び出しで取り出し、元の更新に追従しないスナップショットになる箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型情報を使う検査には、ネイティブの Corsa ランタイムと TypeScript プロジェクトが必要です。typeAware を有効にしても、個別の有効化が必要なルールは自動では有効になりません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-reactivity-loss": "warn"
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

`const count = state.count` がリアクティブなプロパティから数値のスナップショットを取り、その後の `state.count` の更新が変数へ反映されなくなります。

```vue
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0 });
const count = state.count;
</script>
```

## 良い

`toRef(state, "count")` で現在のプリミティブ値をコピーせず、`count` を元のリアクティブなプロパティにつなげます。

```vue
<script setup lang="ts">
import { reactive, toRef } from "vue";
const state = reactive({ count: 0 });
const count = toRef(state, "count");
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) · [全ルール](../all.md)
