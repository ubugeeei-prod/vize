---
title: "ecosystem/vue-test-utils-no-html-snapshot"
---

# `ecosystem/vue-test-utils-no-html-snapshot`

wrapper.html() 全体の snapshot に依存するテストを検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

期待する振る舞いを検査する代わりに、wrapper の HTML 全体を snapshot にしています。

```vue
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

## 良い

表示された文字に Saved が含まれることを直接検査します。

```vue
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) · [全ルール](../all.md)
