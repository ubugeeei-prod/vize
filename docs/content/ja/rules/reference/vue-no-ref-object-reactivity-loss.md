---
title: "vue/no-ref-object-reactivity-loss"
description: "同じスコープで ref の値をコピーし、更新に追従しなくなる箇所を検出します。"
---

# `vue/no-ref-object-reactivity-loss`

同じスコープで ref の値をコピーし、更新に追従しなくなる箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

認識できる ref の生成元から、同じスコープで値をコピーする箇所を検査します。別の関数内での読み取りや `.value` への書き込みは対象外です。ソースを解析する、個別に有効化する検査で、ネイティブの型問い合わせは不要です。

## 悪い例

script のルートで `count.value` を `snapshot` に代入すると、その時点の数値を取り出し、その後の ref の更新に追従しません。

```vue annotate="remove:2,4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const snapshot = count.value;
</script>
```

## 良い例

computed の getter 内で `count.value` を読み、依存値の変化に応じて結果を再評価することで、ルートスコープへのコピーを避けます。

```vue annotate="add:2,4"
<script setup lang="ts">
import { computed, ref } from "vue";
const count = ref(0);
const snapshot = computed(() => count.value);
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
        "vue/no-ref-object-reactivity-loss": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_object_reactivity_loss.rs#L39)
