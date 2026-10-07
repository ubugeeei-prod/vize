---
title: "script/no-restricted-members"
---

# `script/no-restricted-members`

設定で禁止した object.property へのアクセスを検出します。

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: [型付きオプションと既定値](../options.md)を参照してください。

この例では window.localStorage を禁止しています。既定の禁止リストはなく、有効にするだけでは検出されません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage"
            }
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

```vue
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

## 良い

```vue
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) · [全ルール](../all.md)
