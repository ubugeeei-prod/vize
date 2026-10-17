---
title: "script/return-in-emits-validator"
---

# `script/return-in-emits-validator`

Options API の emits validator に戻り値を用意します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

この例では、現在の SFC フィルターが対象にする、関数本体を波括弧で囲んだアロー関数を使います。バリデーター自体はメソッドの省略記法にも対応していますが、現在の SFC 事前フィルターでは、その形式の検査が確実に呼び出されません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/return-in-emits-validator": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`submit` のバリデーターがペイロードをログ出力するだけで検査結果を返さず、関数の戻り値が undefined になります。

```vue
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

## 良い

`return payload != null` を追加し、ペイロードが null でも undefined でもないかを検査した結果を、真偽値で返します。

```vue
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L61) · [全ルール](../all.md)
