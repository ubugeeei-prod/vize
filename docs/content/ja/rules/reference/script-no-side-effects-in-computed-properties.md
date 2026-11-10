---
title: "script/no-side-effects-in-computed-properties"
description: "Options API の computed getter 内の副作用を検出します。"
---

# `script/no-side-effects-in-computed-properties`

Options API の computed getter 内の副作用を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`doubled` が `this.count` に代入し、`reversed` が `reverse()` で `this.items` を変更しています。どちらのゲッターも、値を計算する元の状態を変更してしまっています。

```vue annotate="remove:8,9,12"
<script lang="ts">
export default {
  data() {
    return { count: 0, items: [] }
  },
  computed: {
    doubled() {
      this.count = this.count * 2 // side effect: assigns to data
      return this.count
    },
    reversed() {
      return this.items.reverse() // side effect: mutates the array
    }
  }
}
</script>
```

## 良い例

`doubled` は代入せず乗算結果を返します。`reversed` は配列をコピーしてから反転し、getter が元のコンポーネント状態を変更しないようにします。

```vue annotate="add:8,11"
<script lang="ts">
export default {
  data() {
    return { count: 0, items: [] }
  },
  computed: {
    doubled() {
      return this.count * 2
    },
    reversed() {
      return [...this.items].reverse() // operate on a copy
    }
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
      "script/no-side-effects-in-computed-properties": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L81)
