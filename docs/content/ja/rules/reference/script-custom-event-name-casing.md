---
title: "script/custom-event-name-casing"
---

# `script/custom-event-name-casing`

emit するカスタムイベント名を指定した形式に揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: [型付きオプションと既定値](../options.md)を参照してください。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/custom-event-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

emit する文字列 `my-event` にハイフンが含まれ、既定の camelCase イベント命名規則に違反しています。

```vue
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

## 良い

宣言と呼び出しの両方を `myEvent` にそろえ、イベント名の一致を保ったまま既定の命名規則を満たします。kebab-case に設定した場合の期待値は異なります。

```vue
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) · [全ルール](../all.md)
