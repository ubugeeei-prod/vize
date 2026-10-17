---
title: "script/no-export-in-script-setup"
---

# `script/no-export-in-script-setup`

script setup 内の export 文を検出します。

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
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`export const count` が `<script setup>` からモジュールの値をエクスポートしようとしています。このブロックでは実行時の export は禁止されています。

```vue
<script setup lang="ts">
export const count = 1;
</script>
```

## 良い

`export` を取り除き、`count` をモジュールの export ではなく setup の変数にします。

```vue
<script setup lang="ts">
const count = 1;
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) · [全ルール](../all.md)
