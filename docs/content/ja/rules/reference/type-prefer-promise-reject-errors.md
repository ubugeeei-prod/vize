---
title: "type/prefer-promise-reject-errors"
description: "Promise の拒否理由を Error 相当の値にすることを必須にします。"
---

# `type/prefer-promise-reject-errors`

Promise の拒否理由を Error 相当の値にすることを必須にします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

Promise.reject は文字列を受け取っており、Error の理由ではありません。

```vue annotate="remove:2"
<script setup lang="ts">
const rejected = Promise.reject("failure");
</script>
```

## 良い例

Promise.reject は同じ失敗の文言に対する Error を受け取ります。

```vue annotate="add:2"
<script setup lang="ts">
const rejected = Promise.reject(new Error("failure"));
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/prefer-promise-reject-errors": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_promise_reject_errors.rs#L5)
