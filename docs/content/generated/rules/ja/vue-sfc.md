---
title: "SFC ブロックの構成"
---

<span id="vue-rules-sfc-blocks"></span>

# SFC ブロックの構成

`<template>`、`<script>`、`<style>` の配置やスタイルの範囲を確認し、SFC の構成をそろえるためのルールをまとめています。ブロックを追加したときや、外部ファイルの `src`、`lang`、カスタムブロックを見直すときに使います。

ブロック順の既定値では script と template のどちらを先にしてもよく、style はその後に置きます。`lang` に関する 3 件は規約の説明で、現在の SFC lint では固有の診断を出しません。各ルールの対応範囲を確認し、プロジェクトに必要なものを選んでください。

まずは [SFC ブロックの並び順の例](./reference/vue-sfc-element-order.md)で違いを確かめてください。[Vue ルール](./vue.md)で設定方針を確認し、採用するルールを設定して `vp run lint` を実行します。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [悪い](#vue-no-preprocessor-lang-bad) · [良い](#vue-no-preprocessor-lang-good) | CSS プリプロセッサーより標準の CSS を使う方針を示します。 |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [悪い](#vue-no-script-non-standard-lang-bad) · [良い](#vue-no-script-non-standard-lang-good) | script の非標準の lang 指定を避ける方針を示します。 |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [悪い](#vue-no-src-attribute-bad) · [良い](#vue-no-src-attribute-good) | SFC ブロックの外部 src 指定を検出します。 |
| [`vue/no-template-lang`](#vue-no-template-lang) | [悪い](#vue-no-template-lang-bad) · [良い](#vue-no-template-lang-good) | template の lang 指定を避ける方針を示します。 |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [悪い](#vue-require-scoped-style-bad) · [良い](#vue-require-scoped-style-good) | style に scoped を指定する方針を適用します。 |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [悪い](#vue-sfc-element-order-bad) · [良い](#vue-sfc-element-order-good) | SFC のトップレベルブロックを設定した順に並べます。 |
| [`vue/single-style-block`](#vue-single-style-block) | [悪い](#vue-single-style-block-bad) · [良い](#vue-single-style-block-good) | SFC の style を一つのブロックにまとめます。 |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [悪い](#vue-warn-custom-block-bad) · [良い](#vue-warn-custom-block-good) | SFC のカスタムブロックを検出します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/no-preprocessor-lang`

CSS プリプロセッサーより標準の CSS を使う方針を示します。

[悪い例](#vue-no-preprocessor-lang-bad) · [良い例](#vue-no-preprocessor-lang-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-preprocessor-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-preprocessor-lang-bad"></span>

**悪い**

style の lang に SCSS を指定しています。プリプロセッサーを使わない規約の例で、現在の SFC 検査ではこのルール固有の診断は出ません。

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

<span id="vue-no-preprocessor-lang-good"></span>

**良い**

同じ CSS からプリプロセッサーの lang を取り除きます。規約の修正例で、現在の SFC 検査で診断の有無に差が出る例ではありません。

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L11) · [全ルール](all.md)

### `vue/no-script-non-standard-lang`

script の非標準の lang 指定を避ける方針を示します。

[悪い例](#vue-no-script-non-standard-lang-bad) · [良い例](#vue-no-script-non-standard-lang-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: SFC lint では未対応  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の対応: `no-sfc-finding`

このカタログ項目は現在の SFC lint では固有の診断を出しません。悪い例・良い例は意図した規約の説明であり、実行時にこのルールの診断が出る例ではありません。ID を設定しても未対応の SFC 検査は追加されません。

**設定できる ID（現在の SFC lint では診断されません）**

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

```sh
vp run lint
```

<span id="vue-no-script-non-standard-lang-bad"></span>

**悪い**

lang=coffee で CoffeeScript の構文を使っています。現在の SFC 検査では、この言語に対してこのルール固有の診断は出ません。

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

<span id="vue-no-script-non-standard-lang-good"></span>

**良い**

lang=ts と通常の TypeScript の宣言を使い、意図した言語の方針を示します。

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

良い例は意図する規約を示します。現在の SFC lint では、悪い例・良い例のどちらからもこのルール固有の診断は出ません。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [全ルール](all.md)

### `vue/no-src-attribute`

SFC ブロックの外部 src 指定を検出します。

[悪い例](#vue-no-src-attribute-bad) · [良い例](#vue-no-src-attribute-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-src-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-src-attribute-bad"></span>

**悪い**

SFC の template、script、style を src の外部ファイルに分けています。

```vue annotate="remove:1,2,3"
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

<span id="vue-no-src-attribute-good"></span>

**良い**

各ブロックに内容を記述し、外部の src 属性を使いません。

```vue annotate="add:1,2,3,4,5,6,7,8,9,10,11,12,13"
<template>
  <p>Hello</p>
</template>

<script setup lang="ts">
const label = "Hello";
</script>

<style scoped>
p {
  color: red;
}
</style>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) · [全ルール](all.md)

### `vue/no-template-lang`

template の lang 指定を避ける方針を示します。

[悪い例](#vue-no-template-lang-bad) · [良い例](#vue-no-template-lang-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-lang-bad"></span>

**悪い**

template の lang に Pug を指定しています。HTML のみを使う規約の例で、現在の SFC 検査ではこの ID の診断は出ません。

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

<span id="vue-no-template-lang-good"></span>

**良い**

lang を取り除き、通常の HTML の p を直接記述します。規約の例で、現在の SFC 検査で診断が出ることを示すものではありません。

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L11) · [全ルール](all.md)

### `vue/require-scoped-style`

style に scoped を指定する方針を適用します。

[悪い例](#vue-require-scoped-style-bad) · [良い例](#vue-require-scoped-style-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-scoped-style-bad"></span>

**悪い**

`.button` の style にスコープがなく、他のコンポーネントの一致する要素にも作用します。

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

<span id="vue-require-scoped-style-good"></span>

**良い**

`scoped` を style に付け、同じセレクタと宣言に Vue のコンポーネントスコープを適用します。

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
}
</style>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [全ルール](all.md)

### `vue/sfc-element-order`

SFC のトップレベルブロックを設定した順に並べます。

[悪い例](#vue-sfc-element-order-bad) · [良い例](#vue-sfc-element-order-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/sfc-element-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-sfc-element-order-bad"></span>

**悪い**

style が script より前にあり、設定した SFC ブロック順に合いません。

```vue annotate="remove:2,6,7,8"
<style scoped>
.panel {
  color: red;
}
</style>
<script setup lang="ts">
const label = "Save";
</script>
```

<span id="vue-sfc-element-order-good"></span>

**良い**

script → template → style の順に並べます。型付きオプションで別の順序を選べます。

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts">
const label = "Save";
</script>

<template>
  <p>{{ label }}</p>
</template>

<style scoped>
p {
  color: red;
}
</style>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) · [全ルール](all.md)

### `vue/single-style-block`

SFC の style を一つのブロックにまとめます。

[悪い例](#vue-single-style-block-bad) · [良い例](#vue-single-style-block-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/single-style-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-single-style-block-bad"></span>

**悪い**

panel と title の scoped スタイルを二つの style ブロックに分けています。

```vue annotate="remove:5,6,7"
<style scoped>
.panel {
  color: red;
}
</style>

<style scoped>
.title {
  color: blue;
}
</style>
```

<span id="vue-single-style-block-good"></span>

**良い**

両方のセレクタを一つの scoped style にまとめ、どちらのスタイルも残します。

```vue
<style scoped>
.panel {
  color: red;
}
.title {
  color: blue;
}
</style>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) · [全ルール](all.md)

### `vue/warn-custom-block`

SFC のカスタムブロックを検出します。

[悪い例](#vue-warn-custom-block-bad) · [良い例](#vue-warn-custom-block-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/warn-custom-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-block-bad"></span>

**悪い**

SFC に `<i18n>` のカスタムブロックがあり、通常の template / script / style 処理とは別の連携が必要です。

```vue annotate="remove:1,2,3,4"
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

<span id="vue-warn-custom-block-good"></span>

**良い**

標準の template と script setup を使います。移植性に関するこの任意の警告は、カスタムブロック全般が Vue で無効という意味ではありません。

```vue annotate="add:4,5,6,7"
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) · [全ルール](all.md)
