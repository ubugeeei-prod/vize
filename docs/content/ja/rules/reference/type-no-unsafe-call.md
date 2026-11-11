---
title: "type/no-unsafe-call"
description: "any 型の値を呼び出している箇所を検出します。"
---

# `type/no-unsafe-call`

any 型の値を呼び出している箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

anyValue は any なので、呼び出しても呼び出し先を検査できません。

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
anyValue();
</script>
```

## 良い例

plainNumber は関数なので、呼び出し先が分かっています。

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
function plainNumber(): number {
  return 1;
}
plainNumber();
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-call": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_call.rs#L5)
