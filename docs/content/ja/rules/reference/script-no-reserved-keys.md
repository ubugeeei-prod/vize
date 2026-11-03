---
title: "script/no-reserved-keys"
description: "Options API のキーに Vue の予約名を使う箇所を検出します。"
---

# `script/no-reserved-keys`

Options API のキーに Vue の予約名を使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

data の `$el` が Vue 組み込みのインスタンスプロパティと重なり、予約された `$` 接頭辞も使っています。

```vue annotate="remove:2"
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

## 良い例

アプリケーションのデータ名を `elementLabel` に変え、組み込みのインスタンス API と予約接頭辞を避けます。

```vue annotate="add:2"
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
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
        "script/no-reserved-keys": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L30)
