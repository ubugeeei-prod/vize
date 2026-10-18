---
title: "テンプレートの安全性と HTML の構造"
---

<span id="vue-rules-template-safety"></span>

# テンプレートの安全性と HTML の構造

このルール群では、HTML の挿入や URL の指定、属性と ID の重複、textarea・コンポーネントへのテキストの渡し方、HTML の入れ子を確認します。

外部から受け取った値を表示するときや、リンク・フォームを追加するとき、同じコンポーネントを複数表示するときに確認してください。

たとえば、[HTML を挿入せずテキストとして表示する例](./reference/vue-no-v-html.md)と[フォームの ID を生成して関連付ける例](./reference/vue-use-unique-element-ids.md)では、悪い例と良い例の差を確認できます。採用するルールだけを掲載された設定で有効にし、`vp run lint` を実行してください。[Vue の全ルール](./vue.md)から関連するテンプレートの検査も確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [悪い](#vue-no-duplicate-attributes-bad) · [良い](#vue-no-duplicate-attributes-good) | 同じ要素の属性重複を検出します。 |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [悪い](#vue-no-textarea-mustache-bad) · [良い](#vue-no-textarea-mustache-good) | textarea 内の mustache を検出し、v-model の使用を勧めます。 |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [悪い](#vue-no-unsafe-url-bad) · [良い](#vue-no-unsafe-url-good) | 危険なスキームを含む可能性がある URL 属性やバインディングを検出します。 |
| [`vue/no-v-html`](#vue-no-v-html) | [悪い](#vue-no-v-html-bad) · [良い](#vue-no-v-html-good) | XSS を防ぐため、v-html を使用した箇所に警告を出します。 |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [悪い](#vue-no-v-text-v-html-on-component-bad) · [良い](#vue-no-v-text-v-html-on-component-good) | コンポーネントでの v-text / v-html を検出します。 |
| [`vue/permitted-contents`](#vue-permitted-contents) | [悪い](#vue-permitted-contents-bad) · [良い](#vue-permitted-contents-good) | HTML の要素ごとのコンテンツモデルを検査します。 |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [悪い](#vue-use-unique-element-ids-bad) · [良い](#vue-use-unique-element-ids-good) | 静的 ID の代わりに useId() で一意な ID を生成します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/no-duplicate-attributes`

同じ要素の属性重複を検出します。

[悪い例](#vue-no-duplicate-attributes-bad) · [良い例](#vue-no-duplicate-attributes-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-duplicate-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-duplicate-attributes-bad"></span>

**悪い**

一つの button に class を二回指定しています。

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

<span id="vue-no-duplicate-attributes-good"></span>

**良い**

二つのクラスを一つの class 属性にまとめます。

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [全ルール](all.md)

### `vue/no-textarea-mustache`

textarea 内の mustache を検出し、v-model の使用を勧めます。

[悪い例](#vue-no-textarea-mustache-bad) · [良い例](#vue-no-textarea-mustache-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-textarea-mustache": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-textarea-mustache-bad"></span>

**悪い**

textarea の値をバインドせず、要素内のテキスト補間に message を記述しています。

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

<span id="vue-no-textarea-mustache-good"></span>

**良い**

v-model で、編集可能な textarea の値と message を関連付けます。

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [全ルール](all.md)

### `vue/no-unsafe-url`

危険なスキームを含む可能性がある URL 属性やバインディングを検出します。

[悪い例](#vue-no-unsafe-url-bad) · [良い例](#vue-no-unsafe-url-good)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unsafe-url": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unsafe-url-bad"></span>

**悪い**

a の移動先に、JavaScript を実行する javascript: スキームを使っています。

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

<span id="vue-no-unsafe-url-good"></span>

**良い**

移動先を、通常のローカルパス /next に変更します。

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [全ルール](all.md)

### `vue/no-v-html`

XSS を防ぐため、v-html を使用した箇所に警告を出します。

[悪い例](#vue-no-v-html-bad) · [良い例](#vue-no-v-html-good)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-html": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-html-bad"></span>

**悪い**

v-html で content を通常の文字ではなく HTML として扱っています。

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

<span id="vue-no-v-html-good"></span>

**良い**

mustache の補間で、HTML を挿入せず content をエスケープしたテキストとして表示します。

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [全ルール](all.md)

### `vue/no-v-text-v-html-on-component`

コンポーネントでの v-text / v-html を検出します。

[悪い例](#vue-no-v-text-v-html-on-component-bad) · [良い例](#vue-no-v-text-v-html-on-component-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-text-v-html-on-component": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-text-v-html-on-component-bad"></span>

**悪い**

コンポーネントに、要素の内容を置き換える v-html や v-text を指定しています。

```vue annotate="remove:2,3"
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

<span id="vue-no-v-text-v-html-on-component-good"></span>

**良い**

標準の HTML 要素にはディレクティブを使えます。MyComponent にはデフォルトスロットから内容を渡します。

```vue annotate="add:2,3,4"
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) · [全ルール](all.md)

### `vue/permitted-contents`

HTML の要素ごとのコンテンツモデルを検査します。

[悪い例](#vue-permitted-contents-bad) · [良い例](#vue-permitted-contents-good)

既定の重大度: `error`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/permitted-contents": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-permitted-contents-bad"></span>

**悪い**

p にブロック要素を入れたり、tbody を省略したりしています。操作要素の入れ子や、ul の直下に div を置く例もあります。

```vue annotate="remove:2,3,4,5"
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

<span id="vue-permitted-contents-good"></span>

**良い**

p はインラインの内容、table は tbody、ul は li を使います。独自コンポーネントの MyItem は、ul の子となる既知の HTML 要素としては検査されません。

```vue annotate="add:2,3,4"
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) · [全ルール](all.md)

### `vue/use-unique-element-ids`

静的 ID の代わりに useId() で一意な ID を生成します。

[悪い例](#vue-use-unique-element-ids-bad) · [良い例](#vue-use-unique-element-ids-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-unique-element-ids": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-unique-element-ids-bad"></span>

**悪い**

固定の `email` ID がコンポーネントの各インスタンスで重複し、複数表示時に label の参照先が曖昧になります。

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**良い**

`useId()` の `emailId` を label の `for` と input の `id` の両方にバインドします。

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [全ルール](all.md)
