---
title: "script/no-deprecated-vue-config-keycodes"
description: "削除された Vue.config.keyCodes を検出します。"
---

# `script/no-deprecated-vue-config-keycodes`

削除された Vue.config.keyCodes を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

削除された `Vue.config.keyCodes.enter` にキーコードを代入しています。

```vue annotate="remove:2"
<script setup lang="ts">
Vue.config.keyCodes.enter = 13
</script>
```

## 良い例

キーコードは通常の定数 `const enter = 13` にします。

```vue annotate="add:2"
<script setup lang="ts">
const enter = 13
</script>
```

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

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_vue_config_keycodes.rs#L22)
