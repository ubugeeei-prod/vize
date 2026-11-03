---
title: "vue/no-script-non-standard-lang"
description: "script の非標準の lang 指定を避ける方針を示します。"
---

# `vue/no-script-non-standard-lang`

script の非標準の lang 指定を避ける方針を示します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | SFC lint では未対応 | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

> [!WARNING]
> 現在の対応：`no-sfc-finding`。 このルール ID は設定できますが、現在の SFC lint はまだ診断を出しません。良い例は意図する書き方を示します。

このカタログ項目は現在の SFC lint では固有の診断を出しません。悪い例・良い例は意図した規約の説明であり、実行時にこのルールの診断が出る例ではありません。ID を設定しても未対応の SFC 検査は追加されません。

## 悪い例

lang=coffee で CoffeeScript の構文を使っています。現在の SFC 検査では、この言語に対してこのルール固有の診断は出ません。

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

## 良い例

lang=ts と通常の TypeScript の宣言を使い、意図した言語の方針を示します。

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

<span id="設定できる-id-現在の-sfc-検出なし"></span>

## 設定できる ID（現在の SFC lint では診断されません）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-script-non-standard-lang": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44)
