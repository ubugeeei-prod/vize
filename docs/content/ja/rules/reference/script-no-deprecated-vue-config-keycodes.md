---
title: "script/no-deprecated-vue-config-keycodes"
---

# `script/no-deprecated-vue-config-keycodes`

削除された Vue.config.keyCodes を検出します。

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
        "script/no-deprecated-vue-config-keycodes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

削除された `Vue.config.keyCodes.enter` にキーコードを代入しています。

```vue
<script setup lang="ts">
Vue.config.keyCodes.enter = 13
</script>
```

## 良い

キーコードは通常の定数 `const enter = 13` にします。

```vue
<script setup lang="ts">
const enter = 13
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_vue_config_keycodes.rs#L22) · [全ルール](../all.md)
