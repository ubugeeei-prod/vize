---
title: "script/require-render-return"
description: "render 関数が値を返すことを必須にします。"
---

# `script/require-render-return`

render 関数が値を返すことを必須にします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`render` 関数が分岐の中で `this.count` を読むだけで、値を返していません。

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
  render() {
    if (this.ready) {
      this.count
    }
  }
}
</script>
```

## 良い例

`render` 関数は `this.count` を返します。

```vue annotate="add:4"
<script lang="ts">
export default {
  render() {
    return this.count
  }
}
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
        "script/require-render-return": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_render_return.rs#L35)
