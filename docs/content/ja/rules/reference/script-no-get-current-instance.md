---
title: "script/no-get-current-instance"
description: "Vapor で null を返す getCurrentInstance() を検出します。"
---

# `script/no-get-current-instance`

Vapor で null を返す getCurrentInstance() を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `opinionated` | なし | なし |

適用範囲：Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。

## 悪い例

Vapor を指定した setup が `getCurrentInstance` をインポートして呼び出し、Vapor 向けコンポーネントでこのルールが禁止するインスタンス API に依存しています。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

## 良い例

`inject("app-config")` で明示的に提供された設定を受け取り、`getCurrentInstance` のインポートも呼び出しも使いません。

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-get-current-instance": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38)
