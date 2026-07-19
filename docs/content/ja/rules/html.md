---
title: HTML ルール
---

<!-- Generated translation; source: rules/html.md -->

# HTML ルール

これらのルールは、HTML の有効性と Vue テンプレート内のセマンティック マークアップをカバーします。それらは別のものです
Vue 固有のディレクティブ ルールとアクセシビリティ ルールにより、HTML 適合性チェックを有効にすることができます
または独自に説明します。

## `html/id-duplication`

1 つのテンプレート内の重複する静的 ID をレポートします。

デフォルトの重大度: `error`
プリセット: `essential`、`happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

良い：

```vue
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

## `html/deprecated-element`

非推奨の HTML 要素をレポートします。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <center>Profile</center>
</template>
```

良い：

```vue
<template>
  <section class="profile">Profile</section>
</template>
```

## `html/deprecated-attr`

非推奨の HTML 属性をレポートします。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <table border="1">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

良い：

```vue
<template>
  <table class="summary">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

## `html/no-consecutive-br`

レイアウトに使用される連続した `<br>` 要素をレポートします。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

良い：

```vue
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

## `html/require-datetime`

`<time>` には機械可読な `datetime` 値が必要です。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <time>May 13, 2026</time>
</template>
```

良い：

```vue
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

## `html/no-duplicate-dt`

同じ `<dl>` 内の重複する `<dt>` 用語をレポートします。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

良い：

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

## `html/no-empty-palpable-content`

可視またはその他の方法で認識可能なコンテンツを公開すると予想される空の要素を報告します。
テキスト、子コンテンツ、`aria-label`、`aria-labelledby`、`v-html`、または `v-text` を含む要素は、
受け入れられました。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

良い：

```vue
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```
