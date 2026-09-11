---
title: アクセシビリティルール
---

<!-- Generated translation; source: rules/accessibility.md -->

# アクセシビリティ ルール

アクセシビリティ ルールは、Patina の単一ファイル テンプレート ルールです。困難なマークアップをキャッチします
支援技術またはキーボード ナビゲーションと組み合わせて使用します。

## `a11y/img-alt`

`<img>` には `alt` 属性が必要です。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <img src="/avatar.png" />
</template>
```

良い：

```vue
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

## `a11y/alt-text`

代替テキストが必要なメディア要素には代替テキストが必要です。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <input type="image" src="/submit.png" />
</template>
```

良い：

```vue
<template>
  <input type="image" src="/submit.png" alt="Submit" />
</template>
```

## `a11y/click-events-have-key-events`

キーボード ハンドラーが存在しない場合、非ネイティブのインタラクティブ要素のクリック ハンドラーをレポートします。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <div role="button" @click="save">Save</div>
</template>
```

良い：

```vue
<template>
  <button type="button" @click="save">Save</button>
</template>
```

## `a11y/interactive-supports-focus`

インタラクティブな役割を持つ要素がフォーカス可能である必要があります。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <span role="button" @click="open">Open</span>
</template>
```

良い：

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

## `a11y/label-has-for`

ラベルをフォーム コントロールに関連付ける必要があります。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

良い：

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## `a11y/form-control-has-label`

コントロールには表示ラベルまたはプログラムラベルが必要です。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <input type="search" />
</template>
```

良い：

```vue
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

## `a11y/no-aria-hidden-on-focusable`

支援技術に隠されたフォーカス可能な要素をレポートします。

デフォルトの重大度: `error`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

良い：

```vue
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

## `a11y/no-static-element-interactions`

静的要素のマウスまたはキーボードのハンドラーを報告します。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <section @click="select">Select</section>
</template>
```

良い：

```vue
<template>
  <button type="button" @click="select">Select</button>
</template>
```

## `a11y/tabindex-no-positive`

予測が難しいカスタム タブ オーダーが作成されるため、正の `tabindex` 値が報告されます。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <button tabindex="3">Save</button>
</template>
```

良い：

```vue
<template>
  <button>Save</button>
</template>
```

## `a11y/anchor-is-valid`

アンカーには有効なリンク ターゲットが必要です。
静的な `href` 値はスキームの正規化後にチェックされるため、`JaVaScRiPt:` と HTML デコードされた
`java&#x0A;script:` 内の制御文字は、同様の不一致スキームでも引き続き報告されます。
許可されたままにします。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <a href="#" @click="open">Open</a>
  <a href="JaVaScRiPt:void(0)">Open</a>
</template>
```

良い：

```vue
<template>
  <button type="button" @click="open">Open</button>
  <a href="/docs/javascript:void">Docs</a>
</template>
```

## `a11y/mouse-events-have-key-events`

マウス hover のハンドラーを使う場合は、対応する focus と blur のハンドラーも必要です。

デフォルトの重大度: `warning`。プリセット: `happy-path`、`nuxt`、`opinionated`。

```vue
<!-- 悪い -->
<div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>

<!-- 良い -->
<div tabindex="0" @mouseenter="showPreview" @mouseleave="hidePreview" @focus="showPreview" @blur="hidePreview">Preview</div>
```

## `a11y/no-i-for-icon`

よく使われる icon 用 CSS class によって、`<i>` 要素が icon だけに使われている場合に報告します。

デフォルトの重大度: `warning`。プリセット: `happy-path`、`nuxt`、`opinionated`。

```vue
<!-- 悪い -->
<i class="material-icons">home</i>

<!-- 良い -->
<span class="material-icons" aria-hidden="true">home</span>
<span class="sr-only">Home</span>
```

## `a11y/no-refer-to-non-existent-id`

`for`、`aria-labelledby`、`aria-describedby` などの静的な ID 参照が、同じ template 内に存在する
ID を指していることを要求します。

デフォルトの重大度: `warning`。プリセット: `happy-path`、`nuxt`、`opinionated`。

```vue
<!-- 悪い -->
<label for="email">Email</label>
<input id="user-email" />

<!-- 良い -->
<label for="email">Email</label>
<input id="email" />
```

## 追加のアクセシビリティ ルール

この分け方はドキュメントの詳しさだけの違いです。このページに載っている `a11y/*` はすべて通常の
Patina アクセシビリティルールです。上のセクションは例付きで詳説済み、下の項目は同じ扱いで今後
例を追加していくコンパクトな一覧です。

`a11y/anchor-has-content` では、アンカー要素にアクセス可能なコンテンツが必要です。デフォルト: `warning`。
プリセット: `happy-path`、`nuxt`、`opinionated`。

`a11y/aria-props` は無効な ARIA 属性を許可しません。デフォルト: `error`。プリセット: `happy-path`、
`nuxt`、`opinionated`。

`a11y/aria-role` には、有効な非抽象 ARIA ロールが必要です。デフォルト: `error`。プリセット: `happy-path`、
`nuxt`、`opinionated`。

`a11y/aria-unsupported-elements` は、ARIA 属性をサポートしていない要素の ARIA 属性を許可しません。
デフォルト: `error`。プリセット: `happy-path`、`nuxt`、`opinionated`。

`a11y/heading-has-content` では、見出し要素にアクセス可能なコンテンツが必要です。デフォルト: `warning`。
プリセット: `happy-path`、`nuxt`、`opinionated`。

`a11y/heading-levels` は、見出しレベルのスキップを許可しません。デフォルト: `warning`。プリセット: `nuxt`、
`opinionated`。

`a11y/iframe-has-title` には、`<iframe>` に `title` が必要です。デフォルト: `warning`。プリセット:
`happy-path`、`nuxt`、`opinionated`。

`a11y/landmark-roles` は、ランドマークの役割の配置と一意性を検証します。デフォルト: `warning`。
プリセット: `nuxt`、`opinionated`。

`a11y/media-has-caption` にはメディア要素のキャプションが必要です。デフォルト: `warning`。プリセット:
`happy-path`、`nuxt`、`opinionated`。

`a11y/no-access-key` は、`accesskey` 属性を禁止します。デフォルト: `warning`。プリセット:
`happy-path`、`nuxt`、`opinionated`。

`a11y/no-autofocus` は `autofocus` を許可しません。デフォルト: `warning`。プリセット: `happy-path`、`nuxt`、
`opinionated`。

`a11y/no-distracting-elements` は、`<marquee>` や `<blink>` などの気が散る要素を禁止します。
デフォルト: `warning`。プリセット: `happy-path`、`nuxt`、`opinionated`。

`a11y/no-redundant-roles` は、ネイティブ セマンティクスを複製する ARIA ロールを禁止します。デフォルト:
`warning`。プリセット: `happy-path`、`nuxt`、`opinionated`。

`a11y/no-role-presentation-on-focusable` は `role="presentation"` または `role="none"` を禁止します
フォーカス可能な要素。デフォルト: `error`。プリセット: `happy-path`、`nuxt`、`opinionated`。

`a11y/placeholder-label-option` では、プレースホルダーの `<option>` 値を無効にするか非表示にする必要があります。
デフォルト: `warning`。プリセット: `nuxt`、`opinionated`。

`a11y/role-has-required-aria-props` では、ロールに必要な ARIA 属性を含める必要があります。
デフォルト: `warning`。プリセット: `happy-path`、`nuxt`、`opinionated`。

`a11y/use-list` は、箇条書きのようなテキストのリスト要素を提案します。デフォルト: `warning`。プリセット: `nuxt`、
`opinionated`。
