---
title: "script/no-unstable-nested-components"
---

# `script/no-unstable-nested-components`

setup / render 内で毎回コンポーネントを定義する箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-unstable-nested-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

親の `setup()` 内で `defineComponent` を呼び、setup が実行されるたびに新しい `Child` のコンポーネント定義を作っています。

```vue
<script lang="ts">
import { defineComponent } from "vue";
export default { setup() { const Child = defineComponent({ render() { return null; } }); return { Child }; } };
</script>
```

## 良い

`Child` の定義をモジュール直下に移し、`setup()` は作り直さず既存の定義を返すようにします。

```vue
<script lang="ts">
import { defineComponent } from "vue";
const Child = defineComponent({ render() { return null; } });
export default { setup() { return { Child }; } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) · [全ルール](../all.md)
