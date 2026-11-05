---
title: "vue/no-this-in-before-route-enter"
description: "beforeRouteEnter で未生成のインスタンスを this で参照する箇所を検出します。"
---

# `vue/no-this-in-before-route-enter`

beforeRouteEnter で未生成のインスタンスを this で参照する箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`beforeRouteEnter` の実行時点ではコンポーネントのインスタンスを利用できず、`this.load()` で参照できません。

```vue annotate="remove:4,5"
<script lang="ts">
export default {
  beforeRouteEnter(to, from, next) {
    this.load();
    next();
  },
  methods: { load() {} },
};
</script>
```

## 良い例

`next` のコールバックへ渡されるインスタンスを使い、コンポーネントの生成後に `vm.load()` を呼びます。

```vue annotate="add:4"
<script lang="ts">
export default {
  beforeRouteEnter(to, from, next) {
    next((vm) => vm.load());
  },
  methods: { load() {} },
};
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
        "vue/no-this-in-before-route-enter": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_this_in_before_route_enter.rs#L24)
