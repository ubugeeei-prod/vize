---
title: "ecosystem/vue-test-utils-no-html-snapshot"
description: "wrapper.html() 全体のスナップショットに依存するテストを検出します。"
---

# `ecosystem/vue-test-utils-no-html-snapshot`

wrapper.html() 全体のスナップショットに依存するテストを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

期待する振る舞いを検査する代わりに、wrapper の HTML 全体をスナップショットにしています。

```vue annotate="remove:2"
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

## 良い例

表示された文字に Saved が含まれることを直接検査します。

```vue annotate="add:2"
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
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
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [エコシステムと Nuxt のルール](../ecosystem.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15)
