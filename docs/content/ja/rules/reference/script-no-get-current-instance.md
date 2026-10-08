---
title: "script/no-get-current-instance"
---

# `script/no-get-current-instance`

Vapor で null を返す getCurrentInstance() を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

Vapor を指定した setup が `getCurrentInstance` をインポートして呼び出し、Vapor 向けコンポーネントでこのルールが禁止するインスタンス API に依存しています。

```vue
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

## 良い

`inject("app-config")` で明示的に提供された設定を受け取り、`getCurrentInstance` のインポートも呼び出しも使いません。

```vue
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [全ルール](../all.md)
