---
title: "script/no-arrow-functions-in-watch"
---

# `script/no-arrow-functions-in-watch`

Options API の watch に this を持たないアロー関数を使う箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-arrow-functions-in-watch": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

Options API の watcher `value` と `other.handler` がアロー関数です。アロー関数の `this` は外側から引き継がれ、コンポーネントインスタンスとして束縛されません。

```vue
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

## 良い

両ハンドラーを通常のメソッドに変え、Vue が `this` をコンポーネントに束縛できるようにします。オブジェクト形式の `deep: true` オプションも維持できます。

```vue
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

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) · [全ルール](../all.md)
