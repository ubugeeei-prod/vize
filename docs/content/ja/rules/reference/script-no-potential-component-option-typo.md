---
title: "script/no-potential-component-option-typo"
description: "Options API のオプション名の入力ミスを検出します。"
---

# `script/no-potential-component-option-typo`

Options API のオプション名の入力ミスを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

オプション名が既知の `methods` から一文字欠けた `method` になり、意図したメソッド宣言として扱われません。

```vue annotate="remove:2"
<script lang="ts">
export default { method: { save() {} } };
</script>
```

## 良い例

キーを `methods` に修正し、`save()` を既知のコンポーネントオプション内に置きます。

```vue annotate="add:2"
<script lang="ts">
export default { methods: { save() {} } };
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
        "script/no-potential-component-option-typo": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19)
