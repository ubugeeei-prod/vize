---
title: アクセシビリティルール
---

<!-- Generated translation; source: rules/accessibility.md -->

# アクセシビリティ ルール

アクセシビリティ ルールは Patina の単一ファイル テンプレート ルールです。支援技術、
キーボード ナビゲーション、安定した要素関連付けで扱いにくいマークアップを検出します。

このページではすべてのルールを同じ粒度で説明します。プリセットへの所属は既定で有効になる
場所の違いを示すだけで、二軍扱いの "Additional" 区分はありません。現在のアクセシビリティ
ルールは `linter.ruleOptions` を受け取らないため、重大度は `linter.rules` で調整します。

## `a11y/alt-text`

画像 input、area、object、image など、代替テキストが必要なメディア要素にテキスト代替を
要求します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <input type="image" src="/submit.png" />
</template>
```

良い:

```vue
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

## `a11y/anchor-has-content`

アンカーに、表示テキスト、アクセシブルなラベル、またはラベル付けされた子要素による
アクセシブルな内容を要求します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <a href="/settings"></a>
</template>
```

良い:

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

## `a11y/anchor-is-valid`

アンカーに有効なリンク先を要求します。静的な `href` はスキーム正規化後に検査されるため、
大文字小文字が混じった `javascript:` URL や HTML デコード後の制御文字も検出されます。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <a href="#" @click="openPanel">Open panel</a>
  <a href="JaVaScRiPt:void(0)">Run action</a>
</template>
```

良い:

```vue
<template>
  <button type="button" @click="openPanel">Open panel</button>
  <a href="/docs/javascript:void">JavaScript URL guide</a>
</template>
```

## `a11y/aria-props`

無効な ARIA 属性を禁止します。スペルミスした属性がアクセシビリティ ツリーから黙って消える
前に検出します。

既定の重大度: `error`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

良い:

```vue
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

## `a11y/aria-role`

ARIA role が有効な具象 role であることを要求します。抽象 role や未知の role 名では、期待した
セマンティクスは作られません。

既定の重大度: `error`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <section role="datepicker">...</section>
</template>
```

良い:

```vue
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

## `a11y/aria-unsupported-elements`

ARIA セマンティクスをサポートしない要素で ARIA 属性を使うことを禁止します。

既定の重大度: `error`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

良い:

```vue
<template>
  <meta charset="utf-8" />
</template>
```

## `a11y/click-events-have-key-events`

非ネイティブのインタラクティブ要素に click ハンドラーを置く場合、対応するキーボード
ハンドラーを要求します。可能な場合はネイティブ コントロールを使います。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <div role="button" @click="save">Save</div>
</template>
```

良い:

```vue
<template>
  <button type="button" @click="save">Save</button>
</template>
```

## `a11y/form-control-has-label`

フォーム コントロールに表示ラベルまたはプログラム上のアクセシブル名を要求します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <input type="search" />
</template>
```

良い:

```vue
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

## `a11y/heading-has-content`

見出し要素にアクセシブルな内容を要求します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <h2></h2>
</template>
```

良い:

```vue
<template>
  <h2>Billing settings</h2>
</template>
```

## `a11y/heading-levels`

見出しレベルの飛び越しを禁止します。予測しやすい見出し構造は、キーボードや支援技術での
ページ走査を助けます。

既定の重大度: `warning`
プリセット: `nuxt`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

良い:

```vue
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

## `a11y/iframe-has-title`

iframe 要素に、埋め込む内容を説明する `title` 属性を要求します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <iframe src="/checkout"></iframe>
</template>
```

良い:

```vue
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

## `a11y/img-alt`

画像に `alt` 属性を要求します。装飾画像では属性を省略せず、空文字の `alt` を使います。

既定の重大度: `warning`
プリセット: なし（明示的に有効化）
オプション: なし

悪い:

```vue
<template>
  <img src="/avatar.png" />
</template>
```

良い:

```vue
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

## `a11y/interactive-supports-focus`

インタラクティブな ARIA role を持つ要素にフォーカス可能性を要求します。振る舞いに合う場合は
ネイティブのインタラクティブ要素を優先します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <span role="button" @click="open">Open</span>
</template>
```

良い:

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

## `a11y/label-has-for`

`for`/`id` またはコントロールを包む形で、ラベルとフォーム コントロールの関連付けを要求します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

良い:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## `a11y/landmark-roles`

ページ領域を確実に見つけられるよう、ランドマークの配置と一意性を検査します。

既定の重大度: `warning`
プリセット: `nuxt`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

良い:

```vue
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

## `a11y/media-has-caption`

音声を含む audio/video コンテンツに、キャプションまたはテキスト トラックを要求します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <video src="/demo.mp4" controls />
</template>
```

良い:

```vue
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
</template>
```

## `a11y/mouse-events-have-key-events`

マウス hover ハンドラーを使う場合に、対応する focus/blur ハンドラーも要求します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

良い:

```vue
<template>
  <button
    type="button"
    @focus="showPreview"
    @blur="hidePreview"
    @mouseenter="showPreview"
    @mouseleave="hidePreview"
  >
    Preview
  </button>
</template>
```

## `a11y/no-access-key`

ブラウザーや支援技術のショートカットと競合しやすい `accesskey` 属性を禁止します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <button accesskey="s">Save</button>
</template>
```

良い:

```vue
<template>
  <button>Save</button>
</template>
```

## `a11y/no-aria-hidden-on-focusable`

フォーカス可能な要素に `aria-hidden="true"` を付けることを禁止します。アクセシビリティ
ツリーから消えたまま、キーボード フォーカスだけ受け取る状態を防ぎます。

既定の重大度: `error`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

良い:

```vue
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

## `a11y/no-autofocus`

ページ読み込みやダイアログ表示時にフォーカスが予期せず移動するため、`autofocus` を禁止します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <input autofocus name="query" />
</template>
```

良い:

```vue
<template>
  <input name="query" />
</template>
```

## `a11y/no-distracting-elements`

`<marquee>` や `<blink>` などの注意を逸らす旧式要素を禁止します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <marquee>Limited offer</marquee>
</template>
```

良い:

```vue
<template>
  <p>Limited offer</p>
</template>
```

## `a11y/no-i-for-icon`

アイコン専用要素として `<i>` を使うことを禁止します。アイコンが意味を持つ場合は、ニュートラルな
要素とアクセシブルなテキスト経路を組み合わせます。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

良い:

```vue
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

## `a11y/no-redundant-roles`

要素が暗黙にもつ role と重複する ARIA role を禁止します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <button role="button">Save</button>
</template>
```

良い:

```vue
<template>
  <button>Save</button>
</template>
```

## `a11y/no-refer-to-non-existent-id`

ARIA の ID 参照やラベル関連付けが、同じテンプレート内に存在する要素を指すことを要求します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

良い:

```vue
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

## `a11y/no-role-presentation-on-focusable`

フォーカス可能な要素で `role="presentation"` または `role="none"` を使うことを禁止します。

既定の重大度: `error`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

良い:

```vue
<template>
  <a href="/billing">Billing</a>
</template>
```

## `a11y/no-static-element-interactions`

対応するインタラクティブ role やネイティブ コントロールを持たない静的要素のイベント
ハンドラーを禁止します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

良い:

```vue
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

## `a11y/placeholder-label-option`

`<select>` 内のプレースホルダー `<option>` に `disabled` または `hidden` を要求し、実際の選択肢
として送信されないようにします。

既定の重大度: `warning`
プリセット: `nuxt`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <select v-model="country">
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

良い:

```vue
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

## `a11y/role-has-required-aria-props`

ARIA の state/property 属性を必要とする role に、その必須属性を要求します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

良い:

```vue
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
</template>
```

## `a11y/tabindex-no-positive`

予測しにくい独自のタブ順序を作ってしまうため、正の `tabindex` 値を禁止します。

既定の重大度: `warning`
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <button tabindex="3">Save</button>
</template>
```

良い:

```vue
<template>
  <button>Save</button>
</template>
```

## `a11y/use-list`

箇条書きのようなテキストにリスト要素を使うことを提案します。スクリーン リーダーがリスト構造を
通知できるようにするためです。

既定の重大度: `warning`
プリセット: `nuxt`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

良い:

```vue
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

## `vue/use-unique-element-ids`

静的な要素 ID と ID 参照に、リテラルではなく `useId()` を使うことを要求します。同じ
コンポーネントが複数回レンダーされたときの重複 ID を防ぎ、label や ARIA の関連付けを安定させます。

既定の重大度: `warning`
プリセット: `nuxt`, `opinionated`
オプション: なし

悪い:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

良い:

```vue
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```
