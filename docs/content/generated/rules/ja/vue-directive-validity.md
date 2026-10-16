---
title: "ディレクティブの構文と配置"
---

<span id="vue-rules-directive-validity"></span>

# ディレクティブの構文と配置

このルール群では、v-bind・v-if・v-for・v-model などのディレクティブに必要な値や引数、配置を確認します。

条件分岐や繰り返し、入力との連動、イベント処理、スロットを追加したときに確認してください。v-else の前後関係や、v-for の書式など、各ルールが検査する範囲を掲載しています。

まずは [v-for の繰り返し式と key の例](./reference/vue-valid-v-for.md)と[スロットの宣言の例](./reference/vue-valid-v-slot.md)で、悪い例・良い例の違いを確認してください。採用するルールだけを掲載された設定で有効にし、`vp run lint` を実行します。[Vue の全ルール](./vue.md)では、テンプレートの関連ルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [悪い](#vue-valid-v-bind-bad) · [良い](#vue-valid-v-bind-good) | v-bind の引数・値・修飾子を検査します。 |
| [`vue/valid-v-else`](#vue-valid-v-else) | [悪い](#vue-valid-v-else-bad) · [良い](#vue-valid-v-else-good) | v-else の位置・引数・値を検査します。 |
| [`vue/valid-v-for`](#vue-valid-v-for) | [悪い](#vue-valid-v-for-bad) · [良い](#vue-valid-v-for-good) | v-for の式と変数宣言を検査します。 |
| [`vue/valid-v-if`](#vue-valid-v-if) | [悪い](#vue-valid-v-if-bad) · [良い](#vue-valid-v-if-good) | v-if に有効な条件式を指定します。 |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [悪い](#vue-valid-v-memo-bad) · [良い](#vue-valid-v-memo-good) | v-memo の値を配列の式にします。 |
| [`vue/valid-v-model`](#vue-valid-v-model) | [悪い](#vue-valid-v-model-bad) · [良い](#vue-valid-v-model-good) | v-model の値・引数・修飾子を検査します。 |
| [`vue/valid-v-on`](#vue-valid-v-on) | [悪い](#vue-valid-v-on-bad) · [良い](#vue-valid-v-on-good) | v-on のイベント名・式・修飾子を検査します。 |
| [`vue/valid-v-show`](#vue-valid-v-show) | [悪い](#vue-valid-v-show-bad) · [良い](#vue-valid-v-show-good) | v-show に有効な条件式を指定します。 |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [悪い](#vue-valid-v-slot-bad) · [良い](#vue-valid-v-slot-good) | v-slot の適用先・宣言・修飾子を検査します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/valid-v-bind`

v-bind の引数・値・修飾子を検査します。

[悪い例](#vue-valid-v-bind-bad) · [良い例](#vue-valid-v-bind-good)

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
        "vue/valid-v-bind": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-bind-bad"></span>

**悪い**

引数のない `v-bind` にオブジェクト式がなく、空の引数形式には属性名がありません。

```vue annotate="remove:2,3"
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

<span id="vue-valid-v-bind-good"></span>

**良い**

属性と式を指定するか、オブジェクトをバインドします。Vue 3.4 以降では `:loading` の同名省略形も使えます。

```vue annotate="add:2,3,4"
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) · [全ルール](all.md)

### `vue/valid-v-else`

v-else の位置・引数・値を検査します。

[悪い例](#vue-valid-v-else-bad) · [良い例](#vue-valid-v-else-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: 対応している診断で利用できます  
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
        "vue/valid-v-else": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-else-bad"></span>

**悪い**

`v-else` に式を渡す、`v-if` と併用する、直前の条件分岐がない、といった不正な組み合わせです。

```vue annotate="remove:2,3"
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

<span id="vue-valid-v-else-good"></span>

**良い**

対応する `v-if` の直後に、値のない `v-else` を置きます。

```vue annotate="add:2"
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L31) · [全ルール](all.md)

### `vue/valid-v-for`

v-for の式と変数宣言を検査します。

[悪い例](#vue-valid-v-for-bad) · [良い例](#vue-valid-v-for-good)

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
        "vue/valid-v-for": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-for-bad"></span>

**悪い**

繰り返し式がないか、対応していない `.stop` 修飾子を付けています。

```vue annotate="remove:2,3,4"
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

<span id="vue-valid-v-for-good"></span>

**良い**

`item in items` や `(item, index) of items` の完全な式と、例の key を使います。

```vue annotate="add:2,3"
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L39) · [全ルール](all.md)

### `vue/valid-v-if`

v-if に有効な条件式を指定します。

[悪い例](#vue-valid-v-if-bad) · [良い例](#vue-valid-v-if-good)

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
        "vue/valid-v-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-if-bad"></span>

**悪い**

条件式がないか、同じノードに `v-if` と else 分岐のディレクティブを併記しています。

```vue annotate="remove:2,3,4"
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

<span id="vue-valid-v-if-good"></span>

**良い**

`ready` や `count > 0` の式を各 `v-if` に指定し、競合する else 分岐のディレクティブを除きます。

```vue annotate="add:2,3"
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) · [全ルール](all.md)

### `vue/valid-v-memo`

v-memo の値を配列の式にします。

[悪い例](#vue-valid-v-memo-bad) · [良い例](#vue-valid-v-memo-good)

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
        "vue/valid-v-memo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-memo-bad"></span>

**悪い**

値のない `v-memo` では、サブツリーを再利用する判断に必要な依存式を渡せません。

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

<span id="vue-valid-v-memo-good"></span>

**良い**

`v-memo="[valueA, valueB]"` でメモ化に使う依存配列を渡します。

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [全ルール](all.md)

### `vue/valid-v-model`

v-model の値・引数・修飾子を検査します。

[悪い例](#vue-valid-v-model-bad) · [良い例](#vue-valid-v-model-good)

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
        "vue/valid-v-model": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-model-bad"></span>

**悪い**

ネイティブの `<div>` にはフォーム要素として `v-model` を使えません。また、input に値なしで指定したディレクティブには、書き込み先の式がありません。

```vue annotate="remove:2,3"
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

<span id="vue-valid-v-model-good"></span>

**良い**

input、select、textarea、カスタムコンポーネントを例の書き込み可能な変数にバインドします。

```vue annotate="add:2,3,4,5"
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) · [全ルール](all.md)

### `vue/valid-v-on`

v-on のイベント名・式・修飾子を検査します。

[悪い例](#vue-valid-v-on-bad) · [良い例](#vue-valid-v-on-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**修飾子の検証**

未対応の修飾子は、修飾子の名前ごとに報告します。長い `v-on:click`、動的なイベント引数、オブジェクト形式も対象です。受け付ける名前は `eslint-plugin-vue` 10.9.2 に合わせ、Vue 組み込みのイベント修飾子、完全なキー別名一覧、単一の Unicode コードポイント、有限な十進整数の接頭辞を持つ名前です。整数は JavaScript の `Number.parseInt(name, 10)` と同じように読み取るため、`.13suffix` もこのルールでは有効です。数値キーの非推奨判定と `.native` の移行診断は別のルールが担当します。

ハンドラーなしで動作するのは `.stop` と `.prevent` だけです。`.once`、`.enter`、設定した独自の名前にもハンドラー式が必要です。

独自のイベント処理で使う名前は `linter.ruleOptions` に追加できます。

```json
{
  "linter": {
    "rules": { "vue/valid-v-on": "error" },
    "ruleOptions": {
      "vue/valid-v-on": { "modifiers": ["custom"] }
    }
  }
}
```

これにより `@click.custom="handler"` を使えます。名前は大文字と小文字を区別して完全一致で比較し、組み込みの名前に追加します。ルール自体が無効な場合は有効にしません。範囲別の設定では継承した一覧を置き換え、`[]` でリセットできます。

キーの一覧は、MIT ライセンスの [eslint-plugin-vue のキー別名](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json) を元にしています。出典とライセンスは実装とともに保持しています。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-on": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-on-bad"></span>

**悪い**

イベント引数または必要なハンドラー・オブジェクト式がありません。

```vue annotate="remove:2,3,4,5,6,7"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
  <div @click.sync="handleClick"></div>
  <div @click.nativeish="handleClick"></div>
  <div @click.once></div>
</template>
```

<span id="vue-valid-v-on-good"></span>

**良い**

イベントとハンドラーを指定するか、引数なしの `v-on` にリスナーのオブジェクトを渡します。

```vue annotate="add:2,3,4,5,6"
<template>
  <div @click="handleClick"></div>
  <form @submit.prevent></form>
  <input @keyup.page-down="handlePageDown" />
  <input @keyup.あ="handleKey" />
  <div v-on="{ click: handleClick }"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [全ルール](all.md)

### `vue/valid-v-show`

v-show に有効な条件式を指定します。

[悪い例](#vue-valid-v-show-bad) · [良い例](#vue-valid-v-show-good)

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
        "vue/valid-v-show": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-show-bad"></span>

**悪い**

`v-show` に表示条件の式がないか、display を変更する DOM 要素のない `<template>` に付けています。

```vue annotate="remove:2,3"
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

<span id="vue-valid-v-show-good"></span>

**良い**

`<div>` のような描画される要素に表示条件を指定します。

```vue annotate="add:2,3"
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) · [全ルール](all.md)

### `vue/valid-v-slot`

v-slot の適用先・宣言・修飾子を検査します。

[悪い例](#vue-valid-v-slot-bad) · [良い例](#vue-valid-v-slot-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-slot": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-slot-bad"></span>

**悪い**

ネイティブの `<div>` にスロットのディレクティブを付けるか、他のデフォルトスロットや名前付きスロットの宣言と競合させています。

```vue annotate="remove:2,3,4"
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

<span id="vue-valid-v-slot-good"></span>

**良い**

デフォルトスロットはコンポーネント上、名前付きスロットは子の `<template #header>` に宣言します。

```vue annotate="add:2,3,4,5"
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) · [全ルール](all.md)
