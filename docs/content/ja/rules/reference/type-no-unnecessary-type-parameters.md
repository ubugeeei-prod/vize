---
title: "type/no-unnecessary-type-parameters"
description: "シグネチャで一度しか使われない型パラメーターを検出します。"
---

# `type/no-unnecessary-type-parameters`

シグネチャで一度しか使われない型パラメーターを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

T は logValue のシグネチャに一度しか現れないため、何も関連付けていません。

```vue annotate="remove:2"
<script setup lang="ts">
function logValue<T>(value: T): void {
  console.info(value);
}
logValue(1);
</script>
```

## 良い例

value を unknown にすると、型パラメーターなしで同じ引数を受け取れます。

```vue annotate="add:2"
<script setup lang="ts">
function logValue(value: unknown): void {
  console.info(value);
}
logValue(1);
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-parameters": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_parameters.rs#L5)
