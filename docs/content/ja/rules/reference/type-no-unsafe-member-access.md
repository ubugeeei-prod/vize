---
title: "type/no-unsafe-member-access"
description: "any 型の値のメンバーアクセスを検出します。"
---

# `type/no-unsafe-member-access`

any 型の値のメンバーアクセスを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

anyValue は any なので、member の読み取りは検査されないプロパティアクセスです。

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
console.log(anyValue.member);
</script>
```

## 良い例

box には member プロパティがあるため、アクセスは検査されます。

```vue annotate="add:2,3"
<script setup lang="ts">
const box = { member: 1 };
console.log(box.member);
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-member-access": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_member_access.rs#L5)
