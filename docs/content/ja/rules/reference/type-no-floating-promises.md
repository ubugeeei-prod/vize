---
title: "type/no-floating-promises"
description: "処理しないまま放置された Promise を検出します。"
---

# `type/no-floating-promises`

処理しないまま放置された Promise を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

async の `save` は Promise を返しますが、単独の `save()` 呼び出しが await も return もせず、意図的に捨てる指定もありません。

```vue annotate="remove:3"
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

## 良い例

`void save()` で Promise の完了を待たずに結果を破棄する意図を明示し、このルールの条件を満たします。この指定は Promise の拒否を処理するものではありません。

```vue annotate="add:3"
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-floating-promises": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13)
