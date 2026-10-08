---
title: "script/no-potential-component-option-typo"
---

# `script/no-potential-component-option-typo`

Options API のオプション名の入力ミスを検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-potential-component-option-typo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

オプション名が既知の `methods` から一文字欠けた `method` になり、意図したメソッド宣言として扱われません。

```vue
<script lang="ts">
export default { method: { save() {} } };
</script>
```

## 良い

キーを `methods` に修正し、`save()` を既知のコンポーネントオプション内に置きます。

```vue
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) · [全ルール](../all.md)
