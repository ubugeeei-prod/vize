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

ドキュメントの詳しさだけの違いで、この一覧のルールも同じ Patina template pipeline、同じ設定、
同じ severity で実行されます。

| ルール | デフォルト | 確認する内容 |
| --- | --- | --- |
| `a11y/anchor-has-content` | `warning` | アンカーには、テキスト、補間、アクセシブルな子要素、空でない画像 `alt`、`aria-label`、`aria-labelledby` のいずれかによるアクセシブルネームが必要です。 |
| `a11y/aria-props` | `error` | 有効な `aria-*` 属性だけを許可し、`aria-lable` のような typo がアクセシビリティツリーから静かに消える前に検出します。 |
| `a11y/aria-role` | `error` | `role` の値は具体的な WAI-ARIA role である必要があり、不明な role や抽象 role は拒否します。 |
| `a11y/aria-unsupported-elements` | `error` | metadata、script、style など支援技術に公開されない要素では、ARIA 属性や role を禁止します。 |
| `a11y/heading-has-content` | `warning` | `h1`-`h6` には、表示テキスト、補間、アクセシブルな子要素、または ARIA による名前付けが必要です。 |
| `a11y/heading-levels` | `warning` | 見出しは `h1` から直接 `h3` に飛ぶような outline level のスキップを避けます。 |
| `a11y/iframe-has-title` | `warning` | 各 `iframe` には、空でない静的 title または動的 title binding が必要です。 |
| `a11y/landmark-roles` | `warning` | `main` landmark は 1 つだけ許可し、`nav` や `region` など同じ role の landmark が複数ある場合は、ラベルの欠落や重複を報告します。 |
| `a11y/media-has-caption` | `warning` | `video` と `audio` には `track kind="captions"` が必要です。ただし muted の ambient media など、caption が不要だと判定できる場合は許可されます。 |
| `a11y/no-access-key` | `warning` | ネイティブ要素の `accesskey` は、ブラウザ、OS、支援技術の shortcut と衝突しやすいため禁止します。 |
| `a11y/no-autofocus` | `warning` | 自動 focus 移動は読み上げ順や keyboard flow を中断するため、`autofocus` を禁止します。 |
| `a11y/no-distracting-elements` | `warning` | `marquee` や `blink` のような、動きや点滅を発生させる古い要素を拒否します。 |
| `a11y/no-redundant-roles` | `warning` | `button role="button"` のように native semantics と重複する明示 role を報告し、role の削除で修正できます。 |
| `a11y/no-role-presentation-on-focusable` | `error` | focus 可能な要素で `role="presentation"` や `role="none"` を使うと、focus は残るのに semantics だけ消えるため禁止します。 |
| `a11y/placeholder-label-option` | `warning` | `select` の最初の `option` 子を確認し、その先頭 option の静的な `value` が空なら `disabled` または `hidden` を要求します。 |
| `a11y/role-has-required-aria-props` | `warning` | ARIA state が必須の role には、checkbox の `aria-checked` や slider の `aria-valuenow` など、実装が要求する required property が必要です。 |
| `a11y/use-list` | `warning` | 箇条書きに見えるテキストは semantic な `ul`/`ol` と `li` にして、支援技術が list の境界や item 数を扱えるようにします。 |
