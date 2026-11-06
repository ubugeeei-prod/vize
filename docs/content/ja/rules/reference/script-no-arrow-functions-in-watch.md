---
title: "script/no-arrow-functions-in-watch"
description: "Options API の watch ハンドラーにアロー関数を使う箇所を検出します。"
---

# `script/no-arrow-functions-in-watch`

Options API の watch ハンドラーにアロー関数を使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

Options API の watcher `value` と `other.handler` がアロー関数です。アロー関数の `this` は外側から引き継がれ、コンポーネントインスタンスとして束縛されません。

```vue annotate="remove:4,5,9"
<script lang="ts">
export default {
  watch: {
    // `this` is not the component instance inside an arrow function.
    value: () => {
      this.doSomething()
    },
    other: {
      handler: () => {}
    }
  }
}
</script>
```

## 良い例

両ハンドラーを通常のメソッドに変え、Vue が `this` をコンポーネントインスタンスに束縛できるようにします。オブジェクト形式では `deep: true` オプションも指定できます。

```vue annotate="add:4,8,9"
<script lang="ts">
export default {
  watch: {
    value(newValue, oldValue) {
      this.doSomething()
    },
    other: {
      handler(newValue) {},
      deep: true
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
      "script/no-arrow-functions-in-watch": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59)
