---
title: "script/no-computed-properties-in-data"
---

# `script/no-computed-properties-in-data`

初期化前の data() で算出プロパティを読む箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-computed-properties-in-data": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`data()` が算出プロパティ `doubled` を `this.doubled` として返しています。算出プロパティはまだ初期化されていません。

```vue
<script lang="ts">
export default {
  data() {
    return { value: this.doubled }
  },
  computed: {
    doubled() { return 1 }
  }
}
</script>
```

## 良い

`data()` はただの `value` を返し、`doubled` は初期化後に `this.value` を読みます。

```vue
<script lang="ts">
export default {
  data() {
    return { value: 1 }
  },
  computed: {
    doubled() { return this.value * 2 }
  }
}
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_computed_properties_in_data.rs#L44) · [全ルール](../all.md)
