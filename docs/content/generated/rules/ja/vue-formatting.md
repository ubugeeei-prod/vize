---
title: "テンプレートの属性と表記を揃える"
---

<span id="vue-rules-template-formatting"></span>

# テンプレートの属性と表記を揃える

このルール群では、属性名と並び順、引用符、空白、空の要素の書き方など、テンプレートの表記を確認します。チームで記述を揃えるときや、既存のテンプレートを整理するときに使ってください。

まずは [属性を並べる例](./reference/vue-attribute-order.md)と[同名の props を省略して渡す例](./reference/vue-prefer-props-shorthand.md)で、変更前後の違いを確認してください。同名の prop バインディングを省略する構文は Vue 3.4 以降が対象です。

採用するルールだけを掲載された設定で有効にし、`vp run lint` を実行してください。要素の種類ごとの自己終了タグの方針は[ルールのオプション](./options-vue.md)で指定できます。[Vue の全ルール](./vue.md)では、ディレクティブの構文やテンプレートの構造に関する検査も確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [悪い](#vue-attribute-hyphenation-bad) · [良い](#vue-attribute-hyphenation-good) | コンポーネントの prop 属性名を設定した形式に揃えます。 |
| [`vue/attribute-order`](#vue-attribute-order) | [悪い](#vue-attribute-order-bad) · [良い](#vue-attribute-order-good) | テンプレートの属性を一定の順に並べます。 |
| [`vue/html-quotes`](#vue-html-quotes) | [悪い](#vue-html-quotes-bad) · [良い](#vue-html-quotes-good) | HTML 属性値の引用符を揃えます。 |
| [`vue/html-self-closing`](#vue-html-self-closing) | [悪い](#vue-html-self-closing-bad) · [良い](#vue-html-self-closing-good) | 要素の種類ごとに自己終了タグの形式を揃えます。 |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [悪い](#vue-mustache-interpolation-spacing-bad) · [良い](#vue-mustache-interpolation-spacing-good) | mustache 内の空白を揃えます。 |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [悪い](#vue-no-boolean-attr-value-bad) · [良い](#vue-no-boolean-attr-value-good) | HTML の真偽属性に不要な値を指定した箇所を検出します。 |
| [`vue/no-inline-style`](#vue-no-inline-style) | [悪い](#vue-no-inline-style-bad) · [良い](#vue-no-inline-style-good) | インラインの style 属性を検出します。 |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [悪い](#vue-no-multi-spaces-bad) · [良い](#vue-no-multi-spaces-good) | 連続する不要な空白を検出します。 |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [悪い](#vue-prefer-props-shorthand-bad) · [良い](#vue-prefer-props-shorthand-good) | Vue 3.4 の同名 prop バインディングの省略形を使います。 |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [悪い](#vue-prop-name-casing-bad) · [良い](#vue-prop-name-casing-good) | 宣言する prop 名の形式を揃えます。 |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [悪い](#vue-valid-attribute-name-bad) · [良い](#vue-valid-attribute-name-good) | 有効な属性名を指定します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/attribute-hyphenation`

コンポーネントの prop 属性名を設定した形式に揃えます。

[悪い例](#vue-attribute-hyphenation-bad) · [良い例](#vue-attribute-hyphenation-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応している診断で利用できます  
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
        "vue/attribute-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-hyphenation-bad"></span>

**悪い**

コンポーネントの属性名を camelCase の firstName にしています。

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

<span id="vue-attribute-hyphenation-good"></span>

**良い**

first-name に変更し、設定されたハイフン区切りの属性名の方針に合わせます。

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L32) · [全ルール](all.md)

### `vue/attribute-order`

テンプレートの属性を一定の順に並べます。

[悪い例](#vue-attribute-order-bad) · [良い例](#vue-attribute-order-good)

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
        "vue/attribute-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-order-bad"></span>

**悪い**

構造を決める v-if と通常の id 属性より前に、イベントハンドラーを指定しています。

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

<span id="vue-attribute-order-good"></span>

**良い**

v-if、id、イベントハンドラーの順に並べ、ルールの順序に合わせます。

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [全ルール](all.md)

### `vue/html-quotes`

HTML 属性値の引用符を揃えます。

[悪い例](#vue-html-quotes-bad) · [良い例](#vue-html-quotes-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応している診断で利用できます  
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
        "vue/html-quotes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-quotes-bad"></span>

**悪い**

属性の値にシングルクォートを使うか、クォートを省略しています。

```vue annotate="remove:2,3,4"
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

<span id="vue-html-quotes-good"></span>

**良い**

通常の属性とディレクティブの式をダブルクォートで囲みます。

```vue annotate="add:2,3"
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [全ルール](all.md)

### `vue/html-self-closing`

要素の種類ごとに自己終了タグの形式を揃えます。

[悪い例](#vue-html-self-closing-bad) · [良い例](#vue-html-self-closing-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応している診断で利用できます  
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
        "vue/html-self-closing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-self-closing-bad"></span>

**悪い**

空のコンポーネントを開始タグと終了タグの組で書き、img と br に設定された自己終了タグの表記を使っていません。

```vue annotate="remove:2,3,4"
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

<span id="vue-html-self-closing-good"></span>

**良い**

コンポーネントと void 要素を自己終了タグで書きます。内容がある div は終了タグを残します。

```vue annotate="add:2,3,4,5,6,7"
<template>
  <MyComponent />
  <div></div>
  <div />
  <img />
  <br />
  <div>content</div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) · [全ルール](all.md)

### `vue/mustache-interpolation-spacing`

mustache 内の空白を揃えます。

[悪い例](#vue-mustache-interpolation-spacing-bad) · [良い例](#vue-mustache-interpolation-spacing-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応している診断で利用できます  
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
        "vue/mustache-interpolation-spacing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-mustache-interpolation-spacing-bad"></span>

**悪い**

補間の式と区切りの間で、片側または両側の空白がありません。

```vue annotate="remove:2,3,4"
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

<span id="vue-mustache-interpolation-spacing-good"></span>

**良い**

式と mustache の開始・終了の区切りの間に、それぞれ空白を入れます。

```vue annotate="add:2,3,4"
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L34) · [全ルール](all.md)

### `vue/no-boolean-attr-value`

HTML の真偽属性に不要な値を指定した箇所を検出します。

[悪い例](#vue-no-boolean-attr-value-bad) · [良い例](#vue-no-boolean-attr-value-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応している診断で利用できます  
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
        "vue/no-boolean-attr-value": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-boolean-attr-value-bad"></span>

**悪い**

真偽属性の disabled と checked に、不要な文字列の値を指定しています。

```vue annotate="remove:2,3,4"
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

<span id="vue-no-boolean-attr-value-good"></span>

**良い**

値を付けず、真偽属性があることだけで同じ状態を表します。

```vue annotate="add:2,3,4"
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [全ルール](all.md)

### `vue/no-inline-style`

インラインの style 属性を検出します。

[悪い例](#vue-no-inline-style-bad) · [良い例](#vue-no-inline-style-good)

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
        "vue/no-inline-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-inline-style-bad"></span>

**悪い**

静的な style 属性に色の宣言を直接書いています。

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

<span id="vue-no-inline-style-good"></span>

**良い**

固定した色にはクラスを使います。ratio に依存する幅の動的な style は、静的属性の検査の対象外です。

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [全ルール](all.md)

### `vue/no-multi-spaces`

連続する不要な空白を検出します。

[悪い例](#vue-no-multi-spaces-bad) · [良い例](#vue-no-multi-spaces-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応している診断で利用できます  
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
        "vue/no-multi-spaces": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multi-spaces-bad"></span>

**悪い**

属性同士や、要素名と最初の属性の間に空白が二つあります。

```vue annotate="remove:2,3"
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

<span id="vue-no-multi-spaces-good"></span>

**良い**

同じ class と id 属性を保ち、属性間の連続した空白を一つの空白に揃えます。

```vue annotate="add:2,3"
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [全ルール](all.md)

### `vue/prefer-props-shorthand`

Vue 3.4 の同名 prop バインディングの省略形を使います。

[悪い例](#vue-prefer-props-shorthand-bad) · [良い例](#vue-prefer-props-shorthand-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: 対応している診断で利用できます  
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
        "vue/prefer-props-shorthand": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prefer-props-shorthand-bad"></span>

**悪い**

ハイフン区切りに対応する camelCase も含め、属性と対応する変数名を式に重複して書いています。

```vue annotate="remove:2,3,4,5"
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
```

<span id="vue-prefer-props-shorthand-good"></span>

**良い**

Vue 3.4 以降の同名の省略形で式を省きます。bar のように別の変数を渡す式は明示します。

```vue annotate="add:2,3,4,5,6"
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <span :style />
  <div :aria-label />
  <MyComponent :foo="bar" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) · [全ルール](all.md)

### `vue/prop-name-casing`

宣言する prop 名の形式を揃えます。

[悪い例](#vue-prop-name-casing-bad) · [良い例](#vue-prop-name-casing-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

宣言した props の名前を検査します。子に渡す属性名の表記を検査するルールではありません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/prop-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prop-name-casing-bad"></span>

**悪い**

宣言した prop の user_name をアンダースコア区切りにしています。

```vue annotate="remove:2,4"
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

<span id="vue-prop-name-casing-good"></span>

**良い**

宣言とテンプレートの参照を、camelCase の userName に揃えます。

```vue annotate="add:2,4"
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) · [全ルール](all.md)

### `vue/valid-attribute-name`

有効な属性名を指定します。

[悪い例](#vue-valid-attribute-name-bad) · [良い例](#vue-valid-attribute-name-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

悪い例での診断: `parser/template`

不正な属性名は、この防御的なルールに届く前に parser/template で検出されます。悪い例で確認するのは parser/template の検出で、vue/valid-attribute-name が別に出るとは限りません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/valid-attribute-name": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-attribute-name-bad"></span>

**悪い**

`my"attr` の引用符で属性名が壊れています。この例の診断は `parser/template` で、別のルール診断が出ることを示すものではありません。

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

<span id="vue-valid-attribute-name-good"></span>

**良い**

`my-attr` は正しい属性名で、テンプレートパーサーが属性と値を読み取れます。

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [全ルール](all.md)
