---
title: "script/prefer-define-options"
---

# `script/prefer-define-options`

name / inheritAttrs だけの通常 script を defineOptions() にまとめます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
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
        "script/prefer-define-options": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

通常の script の処理が `name` と `inheritAttrs` だけを持つオブジェクトの export に限られ、`defineOptions` で表せるオプションだけを宣言しています。

```vue
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

## 良い

`data()` が Options API の処理を含むため、オプションの宣言だけを持つ script を対象とする、このルールの提案の範囲外です。この良い例は、通常の script を残せる例外を示しています。悪い例を直接移行する場合は `<script setup>` 内で `defineOptions({ name: 'MyComponent', inheritAttrs: false })` を使います。

```vue
<script lang="ts">
// Real options logic — keep the plain script.
export default {
  name: 'MyComponent',
  data() { return { count: 0 } },
}
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) · [全ルール](../all.md)
