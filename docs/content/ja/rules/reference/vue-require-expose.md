---
title: "vue/require-expose"
description: "Options API のコンポーネントが公開するプロパティを expose で明示します。"
---

# `vue/require-expose`

Options API のコンポーネントが公開するプロパティを expose で明示します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

Options API のコンポーネントオブジェクトが対象です。空の `expose` 一覧を明示してもかまいません。setup の `expose` 呼び出しや描画関数の返却でも条件を満たします。mixin は対象外です。この script の検査は併記された `<script setup>` ブロックを参照せず、`defineExpose` を対象にするルールでもありません。

## 悪い例

Options API のコンポーネントに `focus` メソッドがありますが、公開するインスタンスのインターフェースを明示していません。

```vue annotate="remove:2"
<script lang="ts">
export default { methods: { focus() {} } };
</script>
```

## 良い例

`expose` の一覧に `focus` を指定し、コンポーネントの公開インスタンスから使えることを明示します。

```vue annotate="add:2"
<script lang="ts">
export default { expose: ["focus"], methods: { focus() {} } };
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-expose": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_expose.rs#L24)
