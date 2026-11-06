---
title: "script/no-export-in-script-setup"
description: "script setup 内の export 文を検出します。"
---

# `script/no-export-in-script-setup`

script setup 内の export 文を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`export const count` が `<script setup>` からモジュールの値をエクスポートしようとしています。このブロックでは実行時の export は禁止されています。

```vue annotate="remove:2"
<script setup lang="ts">
export const count = 1;
</script>
```

## 良い例

`export` を取り除き、`count` をモジュールの export ではなく setup の変数にします。

```vue annotate="add:2"
<script setup lang="ts">
const count = 1;
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-export-in-script-setup": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49)
