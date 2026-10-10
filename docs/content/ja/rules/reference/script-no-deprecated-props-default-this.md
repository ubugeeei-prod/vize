---
title: "script/no-deprecated-props-default-this"
---

# `script/no-deprecated-props-default-this`

prop の default / validator 内で使えなくなった this を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: _none_  
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
        "script/no-deprecated-props-default-this": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

prop の既定値関数と validator が `this` を参照していますが、Vue 3 ではこれらの関数からコンポーネントインスタンスに依存できません。

```vue
<script lang="ts">
export default {
  props: {
    size: {
      type: Number,
      // `this` is not the component instance in Vue 3.
      default() {
        return this.defaultSize
      }
    },
    value: {
      type: Number,
      validator() {
        return this.value > 0
      }
    }
  }
}
</script>
```

## 良い

既定値関数は引数の `props.baseSize` を使い、validator は引数の `value` を検査します。どちらも利用できないインスタンスの `this` に依存しなくなります。

```vue
<script lang="ts">
export default {
  props: {
    size: {
      type: Number,
      // Vue 3 passes the raw props as the first argument instead.
      default(props) {
        return props.baseSize
      }
    },
    value: {
      type: Number,
      validator(value) {
        return value > 0
      }
    }
  }
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) · [全ルール](../all.md)
