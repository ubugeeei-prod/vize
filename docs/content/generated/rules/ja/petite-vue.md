---
title: "petite-vue ルール"
---

# petite-vue ルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [悪い](#petite-vue-no-unsupported-directive-bad) · [良い](#petite-vue-no-unsupported-directive-good) | petite-vue が対応しないディレクティブを検出します。 |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [悪い](#petite-vue-valid-v-effect-bad) · [良い](#petite-vue-valid-v-effect-good) | v-effect に空でない式を指定します。 |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [悪い](#petite-vue-valid-v-scope-bad) · [良い](#petite-vue-valid-v-scope-good) | v-scope の値にオブジェクトリテラルを指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `petite-vue/no-unsupported-directive`

petite-vue が対応しないディレクティブを検出します。

[悪い例](#petite-vue-no-unsupported-directive-bad) · [良い例](#petite-vue-no-unsupported-directive-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-no-unsupported-directive-bad"></span>

**悪い**

`v-memo`、`v-slot:header`、カスタムの `v-my-directive` は petite-vue の対応ディレクティブ一覧にありません。petite-vue の script によって、この HTML が対象の方言として判定されます。

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-no-unsupported-directive-good"></span>

**良い**

対応している `v-scope`、`v-effect`、`v-if`、`v-bind`、`v-on` を使い、未対応のディレクティブへの依存を取り除きます。

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) · [全ルール](all.md)

### `petite-vue/valid-v-effect`

v-effect に空でない式を指定します。

[悪い例](#petite-vue-valid-v-effect-bad) · [良い例](#petite-vue-valid-v-effect-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-effect": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-effect-bad"></span>

**悪い**

各 `v-effect` の値が未指定、空文字、空白のみであり、実行する式がありません。

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-effect-good"></span>

**良い**

一方は `el.textContent` を更新し、もう一方は `count` を増やす式を指定しています。このルールが確認するのは式が空でないことであり、処理内容の妥当性ではありません。

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) · [全ルール](all.md)

### `petite-vue/valid-v-scope`

v-scope の値にオブジェクトリテラルを指定します。

[悪い例](#petite-vue-valid-v-scope-bad) · [良い例](#petite-vue-valid-v-scope-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-scope": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-scope-bad"></span>

**悪い**

値を持つ四つの `v-scope` は識別子、関数呼び出し、算術式、数値であり、いずれもオブジェクトリテラルではありません。

```html annotate="remove:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-scope-good"></span>

**良い**

値を省略した `v-scope` はルートスコープを使います。ほかは括弧付きのものも含めてオブジェクトリテラルであり、このルールで許可されます。

```html annotate="add:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) · [全ルール](all.md)
