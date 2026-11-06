---
title: "script/return-in-emits-validator"
description: "Options API の emits validator に戻り値を用意します。"
---

# `script/return-in-emits-validator`

Options API の emits validator に戻り値を用意します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

この例では、現在の SFC フィルターが対象にする、関数本体を波括弧で囲んだアロー関数を使います。バリデーター自体はメソッドの省略記法にも対応していますが、現在の SFC 事前フィルターでは、その形式の検査が確実に呼び出されません。

## 悪い例

`submit` のバリデーターがペイロードをログ出力するだけで検査結果を返さず、関数の戻り値が undefined になります。

```vue annotate="remove:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

## 良い例

`return payload != null` を追加し、ペイロードが null でも undefined でもないかを検査した結果を、真偽値で返します。

```vue annotate="add:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/return-in-emits-validator": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L61)
