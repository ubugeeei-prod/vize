---
title: "繰り返し・条件分岐とテンプレートの構造"
---

<span id="vue-rules-template-structure"></span>

# 繰り返し・条件分岐とテンプレートの構造

繰り返しや条件分岐、スロットの変数と、template 要素の使い方を確認するルールをまとめています。リストの表示や分岐を変更したとき、テンプレートの構造を整理したときに使います。

リストでは項目を識別する key、条件分岐では重複する条件、入れ子のループやスロットでは変数の参照を確認します。no-template-shadow はテンプレート内の v-for・v-slot・従来のスコープ付きスロットの束縛を比較します。兄弟要素のスコープは独立しており、script 側と同じ名前であるだけでは診断しません。各ルールの現在の検査範囲を確認してください。

まずは [繰り返す要素に key を指定する例](./reference/vue-require-v-for-key.md)と[v-if と v-for を分ける例](./reference/vue-no-use-v-if-with-v-for.md)で、悪い例と良い例の差を確認してください。採用するルールだけを掲載された設定で有効にし、`vp run lint` を実行します。[Vue の全ルール](./vue.md)から、構文や属性の関連ルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/no-child-content`](#vue-no-child-content) | [悪い](#vue-no-child-content-bad) · [良い](#vue-no-child-content-good) | v-html / v-text と子コンテンツを同時に指定する箇所を検出します。 |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [悪い](#vue-no-dupe-v-else-if-bad) · [良い](#vue-no-dupe-v-else-if-good) | v-if / v-else-if の条件重複を検出します。 |
| [`vue/no-lone-template`](#vue-no-lone-template) | [悪い](#vue-no-lone-template-bad) · [良い](#vue-no-lone-template-good) | 不要な template 要素を検出します。 |
| [`vue/no-template-key`](#vue-no-template-key) | [悪い](#vue-no-template-key-bad) · [良い](#vue-no-template-key-good) | v-for 用ではない template の key 指定を検出します。 |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [悪い](#vue-no-template-shadow-bad) · [良い](#vue-no-template-shadow-good) | テンプレート変数が外側の名前を隠す箇所を検出します。 |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [悪い](#vue-no-unused-vars-bad) · [良い](#vue-no-unused-vars-good) | v-for / v-slot に宣言しているのに使わない変数を検出します。 |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [悪い](#vue-no-use-v-if-with-v-for-bad) · [良い](#vue-no-use-v-if-with-v-for-good) | 同じ要素での v-if と v-for の併用を検出します。 |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [悪い](#vue-no-useless-template-attributes-bad) · [良い](#vue-no-useless-template-attributes-good) | template 要素の効果がない属性を検出します。 |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [悪い](#vue-require-v-for-key-bad) · [良い](#vue-require-v-for-key-good) | v-for に安定した :key を指定します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/no-child-content`

v-html / v-text と子コンテンツを同時に指定する箇所を検出します。

[悪い例](#vue-no-child-content-bad) · [良い例](#vue-no-child-content-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-child-content": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-child-content-bad"></span>

**悪い**

v-text が p の内容を置き換えるため、中に書いたフォールバック用のテキストは表示されません。

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

<span id="vue-no-child-content-good"></span>

**良い**

子のテキストを取り除き、p の内容を v-text だけで指定します。

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [全ルール](all.md)

### `vue/no-dupe-v-else-if`

v-if / v-else-if の条件重複を検出します。

[悪い例](#vue-no-dupe-v-else-if-bad) · [良い例](#vue-no-dupe-v-else-if-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-dupe-v-else-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-dupe-v-else-if-bad"></span>

**悪い**

最初と同じ ready の条件を else-if に書き、後の分岐に到達できません。

```vue annotate="remove:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

<span id="vue-no-dupe-v-else-if-good"></span>

**良い**

二つ目で別の loading の状態を検査し、else-if に到達できる条件にします。

```vue annotate="add:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) · [全ルール](all.md)

### `vue/no-lone-template`

不要な template 要素を検出します。

[悪い例](#vue-no-lone-template-bad) · [良い例](#vue-no-lone-template-good)

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
        "vue/no-lone-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-lone-template-bad"></span>

**悪い**

内側の template には、構造を指定するディレクティブやスロットとしての役割がありません。

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

<span id="vue-no-lone-template-good"></span>

**良い**

不要な template を取り除き、div に p を直接入れます。

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [全ルール](all.md)

### `vue/no-template-key`

v-for 用ではない template の key 指定を検出します。

[悪い例](#vue-no-template-key-bad) · [良い例](#vue-no-template-key-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-template-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-key-bad"></span>

**悪い**

繰り返しではない template に key を指定しています。

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

<span id="vue-no-template-key-good"></span>

**良い**

template の v-for に key を付け、繰り返す各フラグメントを識別します。

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) · [全ルール](all.md)

### `vue/no-template-shadow`

テンプレート変数が外側の名前を隠す箇所を検出します。

[悪い例](#vue-no-template-shadow-bad) · [良い例](#vue-no-template-shadow-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

検査はテンプレート内の `v-for`、`v-slot`（`#name` を含む）、従来のスコープ付きスロットの束縛を比較します。外側の束縛を隠す識別子を報告し、同じ要素内では属性の順番に比較します。兄弟要素のスコープは独立しています。プロパティのキーや既定値の参照は宣言ではなく、`v-pre` の部分木は検査しません。

script やコンポーネントのプロパティ名は現在の検査範囲に含みません。同じ名前の script 束縛があるだけで、単独のテンプレート束縛を報告することはありません。診断範囲は、エスケープや HTML エンティティを含む元の識別子を示します。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-shadow": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-shadow-bad"></span>

**悪い**

内側の v-for でも item を宣言し、外側の item を隠しています。

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

<span id="vue-no-template-shadow-good"></span>

**良い**

内側は child に変更し、外側の item と内側の child を分けます。

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [全ルール](all.md)

### `vue/no-unused-vars`

v-for / v-slot に宣言しているのに使わない変数を検出します。

[悪い例](#vue-no-unused-vars-bad) · [良い例](#vue-no-unused-vars-good)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-unused-vars": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-vars-bad"></span>

**悪い**

ループで使わない index と、スロットで使わない foo を宣言しています。

```vue annotate="remove:2,3,4"
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

<span id="vue-no-unused-vars-good"></span>

**良い**

index を参照するか、未使用を意図する _index という名前にし、スロットでは data を表示します。index の key は使用例であり、項目の安定した識別子として推奨するものではありません。

```vue annotate="add:2,3,4,5"
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) · [全ルール](all.md)

### `vue/no-use-v-if-with-v-for`

同じ要素での v-if と v-for の併用を検出します。

[悪い例](#vue-no-use-v-if-with-v-for-bad) · [良い例](#vue-no-use-v-if-with-v-for-good)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-use-v-if-with-v-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-use-v-if-with-v-for-bad"></span>

**悪い**

同じ li に v-if と v-for を指定し、ループの変数で表示条件を検査しています。

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

<span id="vue-no-use-v-if-with-v-for-good"></span>

**良い**

computed で表示する項目を先に絞り込み、テンプレートではその配列の各項目を繰り返し描画します。

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const visibleItems = computed(() => items.filter((item) => item.visible));
</script>

<template>
  <li v-for="item in visibleItems" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) · [全ルール](all.md)

### `vue/no-useless-template-attributes`

template 要素の効果がない属性を検出します。

[悪い例](#vue-no-useless-template-attributes-bad) · [良い例](#vue-no-useless-template-attributes-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-useless-template-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-template-attributes-bad"></span>

**悪い**

条件付きの template に class を指定していますが、この構造用のラッパーは DOM 要素を描画しません。

```vue annotate="remove:2"
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

<span id="vue-no-useless-template-attributes-good"></span>

**良い**

実際に表示する p に class を移し、構造を指定する template の v-if は残します。

```vue annotate="add:2"
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [全ルール](all.md)

### `vue/require-v-for-key`

v-for に安定した :key を指定します。

[悪い例](#vue-require-v-for-key-bad) · [良い例](#vue-require-v-for-key-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/require-v-for-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-v-for-key-bad"></span>

**悪い**

繰り返す `<li>` に key がなく、一覧更新時に対応する項目を識別できません。

```vue annotate="remove:2"
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

<span id="vue-require-v-for-key-good"></span>

**良い**

`:key="item.id"` で、現在の位置ではなく項目の識別子を各ノードに付けます。

```vue annotate="add:2"
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L46) · [全ルール](all.md)
