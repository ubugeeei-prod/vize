---
title: "script/no-restricted-members"
description: "設定で禁止した object.property へのアクセスを検出します。"
---

# `script/no-restricted-members`

設定で禁止した object.property へのアクセスを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

この例では window.localStorage を禁止しています。既定の禁止リストはなく、有効にするだけでは検出されません。

## 悪い例

`ruleOptions` に `{ object: "window", property: "localStorage" }` を設定した場合、`window.localStorage` は禁止されたオブジェクトとメンバーの組み合わせです。このルールに既定の禁止メンバーはありません。

```vue annotate="remove:2"
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

## 良い例

`authStorage.read("token")` でアプリケーションのストレージヘルパーに処理を任せ、設定で禁止した `window.localStorage` を参照しなくなります。

```vue annotate="add:2"
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
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
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53)
