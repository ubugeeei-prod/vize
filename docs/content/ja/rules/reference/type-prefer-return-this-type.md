---
title: "type/prefer-return-this-type"
description: "返す値が `this` だけのメソッドでは戻り値の型に `this` を使います。"
---

# `type/prefer-return-this-type`

返す値が `this` だけのメソッドでは戻り値の型に `this` を使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

setValue は this を返しますが、宣言された戻り値の型はクラス名 Builder です。

```vue annotate="remove:3"
<script setup lang="ts">
class Builder {
  setValue(): Builder {
    return this;
  }
}
</script>
```

## 良い例

戻り値の型 this は、メソッドが返す値と一致します。

```vue annotate="add:3"
<script setup lang="ts">
class Builder {
  setValue(): this {
    return this;
  }
}
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/prefer-return-this-type": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_return_this_type.rs#L5)
