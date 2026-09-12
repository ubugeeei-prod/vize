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

## `a11y/anchor-has-content`

アンカー要素に、アクセシブルネームになる内容を要求します。テキスト、補間、アクセシブルな子要素、
空ではない `alt` を持つ画像、`aria-label`、`aria-labelledby` があれば満たされます。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <a href="/settings"></a>
</template>
```

良い：

```vue
<template>
  <a href="/settings">Settings</a>
  <a href="/settings" aria-label="Settings"></a>
</template>
```

## `a11y/aria-props`

無効な `aria-*` 属性を禁止します。スペルミスや標準にない ARIA 名を、アクセシビリティツリーから
静かに消える前に検出します。

デフォルトの重大度: `error`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <button aria-lable="Close">x</button>
</template>
```

良い：

```vue
<template>
  <button aria-label="Close">x</button>
</template>
```

## `a11y/aria-role`

`role` の値が有効で具体的な ARIA ロールであることを要求します。不明なロールや抽象ロールは、
支援技術が意図どおりに扱えないため報告されます。

デフォルトの重大度: `error`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <div role="datepicker">Choose a date</div>
  <div role="command">Run</div>
</template>
```

良い：

```vue
<template>
  <button type="button">Choose a date</button>
  <div role="button" tabindex="0">Run</div>
</template>
```

## `a11y/aria-unsupported-elements`

アクセシビリティツリーに参加しない要素で ARIA 属性や `role` を使うことを禁止します。対象には
metadata 要素、`script`、`style` などが含まれます。

デフォルトの重大度: `error`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <meta aria-hidden="true" />
  <script role="presentation"></script>
</template>
```

良い：

```vue
<template>
  <meta name="viewport" content="width=device-width" />
  <script></script>
</template>
```

## `a11y/heading-has-content`

`<h1>` から `<h6>` までの見出しにアクセシブルな内容を要求します。表示テキスト、補間、アクセシブルな
子要素、ARIA による名前付けがあれば満たされます。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <h2></h2>
</template>
```

良い：

```vue
<template>
  <h2>Billing</h2>
  <h2 aria-label="Billing"></h2>
</template>
```

## `a11y/heading-levels`

見出しレベルのスキップを報告します。見出し順はページのアウトラインを表すため、`<h1>` から
`<h3>` へ、間の `<h2>` なしに飛ぶべきではありません。

デフォルトの重大度: `warning`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<template>
  <h1>Account</h1>
  <h3>Invoices</h3>
</template>
```

良い：

```vue
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
  <h3>Invoices</h3>
</template>
```

## `a11y/iframe-has-title`

すべての `<iframe>` に空ではない `title`、または動的値の `title` binding を要求します。スクリーン
リーダーはこの title を使って埋め込み文書を識別します。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <iframe src="/billing"></iframe>
</template>
```

良い：

```vue
<template>
  <iframe src="/billing" title="Billing dashboard"></iframe>
</template>
```

## `a11y/landmark-roles`

ランドマークの配置と一意性を検証します。重複した `main`、競合する入れ子ランドマーク、区別できる
ラベルを持たない複数の navigation/region ランドマークを報告します。

デフォルトの重大度: `warning`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<template>
  <main>Primary content</main>
  <main>Secondary content</main>
  <nav>Primary navigation</nav>
  <nav>Footer navigation</nav>
</template>
```

良い：

```vue
<template>
  <main>Primary content</main>
  <nav aria-label="Primary">Primary navigation</nav>
  <nav aria-label="Footer">Footer navigation</nav>
</template>
```

## `a11y/media-has-caption`

`<video>` と `<audio>` に、子要素の `<track kind="captions">` による caption を要求します。muted
media や明示的にラベル付けされた media は、caption が不要だとルールが判断できる場合に許可されます。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <video src="/launch.mp4"></video>
</template>
```

良い：

```vue
<template>
  <video src="/launch.mp4">
    <track kind="captions" src="/launch.en.vtt" />
  </video>
  <video src="/ambient.mp4" muted></video>
</template>
```

## `a11y/no-access-key`

ネイティブ要素の `accesskey` を禁止します。ブラウザ、OS、支援技術のショートカットは環境ごとに
異なるため、独自 access key は衝突しやすくなります。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <button accesskey="s">Save</button>
</template>
```

良い：

```vue
<template>
  <button type="button">Save</button>
</template>
```

## `a11y/no-autofocus`

`autofocus` を禁止します。自動で focus を移すと、スクリーンリーダーの読み上げ順やキーボード操作を
予期せず中断することがあります。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <input autofocus />
</template>
```

良い：

```vue
<template>
  <input />
</template>
```

## `a11y/no-distracting-elements`

`<marquee>` や `<blink>` などの要素を禁止します。これらのタグは動きや点滅を制御しづらく、
アクセシブルな現代的 UI には適しません。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <marquee>Sale ends soon</marquee>
  <blink>Unread</blink>
</template>
```

良い：

```vue
<template>
  <p>Sale ends soon</p>
  <strong>Unread</strong>
</template>
```

## `a11y/no-redundant-roles`

HTML のネイティブ semantics と重複する明示的な ARIA role を禁止します。静的に冗長だと判断できる
role は fixable で、削除しても公開される意味は変わりません。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`
自動修正: 対応

悪い：

```vue
<template>
  <button role="button">Save</button>
  <nav role="navigation">Sections</nav>
</template>
```

良い：

```vue
<template>
  <button type="button">Save</button>
  <nav>Sections</nav>
  <div role="navigation">Supplemental links</div>
</template>
```

## `a11y/no-role-presentation-on-focusable`

focus 可能な要素の `role="presentation"` と `role="none"` を禁止します。focus は残るのに semantics
だけが消えるため、focus 中の control がスクリーンリーダー利用者に分かりづらくなります。

デフォルトの重大度: `error`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <button role="presentation">Close</button>
  <div role="none" tabindex="0">Focusable panel</div>
</template>
```

良い：

```vue
<template>
  <button type="button">Close</button>
  <div tabindex="0">Focusable panel</div>
</template>
```

## `a11y/placeholder-label-option`

`<select>` の placeholder option に `disabled` または `hidden` を要求します。最初の empty-value
option は placeholder と扱われ、送信可能な選択肢として残すべきではありません。

デフォルトの重大度: `warning`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<template>
  <select>
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

良い：

```vue
<template>
  <select>
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

## `a11y/role-has-required-aria-props`

ARIA role が必要とする ARIA property を要求します。たとえば checkbox には `aria-checked`、slider
には現在値と範囲が必要です。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`ecosystem`、`opinionated`

悪い：

```vue
<template>
  <div role="checkbox">Subscribe</div>
  <div role="slider">Volume</div>
</template>
```

良い：

```vue
<template>
  <div role="checkbox" aria-checked="false">Subscribe</div>
  <div role="slider" aria-valuemin="0" aria-valuemax="100" aria-valuenow="50">Volume</div>
</template>
```

## `a11y/use-list`

箇条書きに見えるテキストには semantic list markup を提案します。実際の list にすると、支援技術が
item 数、list の境界、list navigation を扱えるようになります。

デフォルトの重大度: `warning`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<template>
  <p>- Create an account</p>
  <p>- Invite the team</p>
</template>
```

良い：

```vue
<template>
  <ul>
    <li>Create an account</li>
    <li>Invite the team</li>
  </ul>
</template>
```
