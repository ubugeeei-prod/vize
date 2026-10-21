---
title: "ディレクティブとイベントの書き方"
---

<span id="vue-rules-directive-conventions"></span>

# ディレクティブとイベントの書き方

このルール群では、v-bind・v-on・v-slot の表記、イベント名、修飾キー付きのイベント処理、カスタムディレクティブの使用方針を確認します。
チームで表記を揃えたり、クリック操作やスロット、独自のディレクティブを追加したりするときに確認してください。

たとえば、[修飾キーによるクリックの重複を分ける例](./reference/vue-use-v-on-exact.md)では、Ctrl キーを押したときに通常のハンドラーも動く指定と、`.exact` を使って分ける指定を比較できます。
採用するルールだけを掲載された設定で有効にし、`vp run lint` を実行してください。
[Vue の全ルール](./vue.md)では、ディレクティブの構文やテンプレートの関連ルールも確認できます。

このページに、各ルールの目的・前提・設定と、省略のない悪い例・良い例をまとめています。変更箇所は行の色で示します。コードをコピーすると、ソースコード全体を取得できます。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [悪い](#vue-scoped-event-names-bad) · [良い](#vue-scoped-event-names-good) | イベント名を context:event の形式に揃えます。 |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [悪い](#vue-use-v-on-exact-bad) · [良い](#vue-use-v-on-exact-good) | 修飾子付きのイベント操作と競合するハンドラーに .exact を指定します。 |
| [`vue/v-bind-style`](#vue-v-bind-style) | [悪い](#vue-v-bind-style-bad) · [良い](#vue-v-bind-style-good) | v-bind の表記形式を揃えます。 |
| [`vue/v-on-style`](#vue-v-on-style) | [悪い](#vue-v-on-style-bad) · [良い](#vue-v-on-style-good) | v-on の表記形式を揃えます。 |
| [`vue/v-slot-style`](#vue-v-slot-style) | [悪い](#vue-v-slot-style-bad) · [良い](#vue-v-slot-style-good) | v-slot の表記形式を揃えます。 |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [悪い](#vue-warn-custom-directive-bad) · [良い](#vue-warn-custom-directive-good) | 登録が必要なカスタムディレクティブを検出します。 |

[全ルール](./all.md) · [ルールのオプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/scoped-event-names`

イベント名を context:event の形式に揃えます。

[悪い例](#vue-scoped-event-names-bad) · [良い例](#vue-scoped-event-names-good)

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
        "vue/scoped-event-names": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-scoped-event-names-bad"></span>

**悪い**

`playAudio`、`pauseAudio`、`reloadAudio` は camelCase の末尾に対象を付けており、このルールのコロン区切りの規約に合いません。

```vue annotate="remove:3,4,5"
<template>
  <AudioPlayer
    @playAudio="play"
    @pauseAudio="pause"
    @reloadAudio="reload"
  />
</template>
```

<span id="vue-scoped-event-names-good"></span>

**良い**

`audio:play`、`audio:pause`、`audio:reload` に `audio:` のスコープを明示します。emit 側も同じ名前に合わせます。

```vue annotate="add:3,4,5"
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) · [全ルール](all.md)

### `vue/use-v-on-exact`

修飾子付きのイベント操作と競合するハンドラーに .exact を指定します。

[悪い例](#vue-use-v-on-exact-bad) · [良い例](#vue-use-v-on-exact-good)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**ハンドラーが重なる条件**

ネイティブ要素の同じイベントを処理するリスナーを比較します。独自コンポーネントのイベントは、両方に `.native` が付いている場合だけ対象です。イベント名が異なる場合や、キーフィルターに共通するキーがない場合は競合しません。

システム修飾子は、押している必要があるキーを指定します。`@keydown.alt.shift` は Ctrl+Alt+Shift でも動くため、`@keydown.alt.ctrl.shift` と重なる場合は広い方に `.exact` を追加します。`.exact` は指定したシステムキーを許可し、それ以外の Ctrl、Shift、Alt、Meta を拒否します。

複数のキーボード修飾子は選択肢です。`@keydown.up.down` は上・下のどちらでも動き、Ctrl+下では `@keydown.ctrl.down` と重なります。診断範囲は `.exact` が必要なディレクティブのキーで、ハンドラー式は含みません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/use-v-on-exact": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-v-on-exact-bad"></span>

**悪い**

通常の click ハンドラーも Ctrl キーを押しながらのクリックで動くため、別の `.ctrl` ハンドラーと重複します。

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

<span id="vue-use-v-on-exact-good"></span>

**良い**

`.exact` で通常のハンドラーを修飾キーのない click に限定し、Ctrl 専用のハンドラーと分けます。

```vue annotate="add:2,3,4,5,6"
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L31) · [全ルール](all.md)

### `vue/v-bind-style`

v-bind の表記形式を揃えます。

[悪い例](#vue-v-bind-style-bad) · [良い例](#vue-v-bind-style-good)

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
        "vue/v-bind-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-bind-style-bad"></span>

**悪い**

`v-bind:class` が長い形式で、設定で指定したコロンによる省略形の規約に合いません。

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

<span id="vue-v-bind-style-good"></span>

**良い**

同じ式を `:class` にします。値の型ではなくディレクティブの書き方を検査するルールです。

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L28) · [全ルール](all.md)

### `vue/v-on-style`

v-on の表記形式を揃えます。

[悪い例](#vue-v-on-style-bad) · [良い例](#vue-v-on-style-good)

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
        "vue/v-on-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-style-bad"></span>

**悪い**

`v-on:click` が長い形式で、このルールの省略形に合いません。

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

<span id="vue-v-on-style-good"></span>

**良い**

ハンドラーを変えずに `@click` の省略形を使います。

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) · [全ルール](all.md)

### `vue/v-slot-style`

v-slot の表記形式を揃えます。

[悪い例](#vue-v-slot-style-bad) · [良い例](#vue-v-slot-style-good)

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
        "vue/v-slot-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-slot-style-bad"></span>

**悪い**

コンポーネントで `#default`、template で `v-slot:header` を使い、場所ごとの規約と逆になっています。

```vue annotate="remove:2,4"
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

<span id="vue-v-slot-style-good"></span>

**良い**

コンポーネントのデフォルトスロットは `v-slot`、template の名前付きスロットは `#header` にします。

```vue annotate="add:2,4"
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [全ルール](all.md)

### `vue/warn-custom-directive`

登録が必要なカスタムディレクティブを検出します。

[悪い例](#vue-warn-custom-directive-bad) · [良い例](#vue-warn-custom-directive-good)

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
        "vue/warn-custom-directive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-directive-bad"></span>

**悪い**

`v-focus`、`v-mask`、`v-click-outside` はプロジェクト固有のディレクティブの実装を必要とし、この任意の規約で警告されます。

```vue annotate="remove:2,3,4"
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

<span id="vue-warn-custom-directive-good"></span>

**良い**

組み込みの `v-if`、`v-model`、`v-on` を使います。規約を無効にすれば、正しく登録したカスタムディレクティブは有効な Vue として使えます。

```vue annotate="add:2,3,4"
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) · [全ルール](all.md)
