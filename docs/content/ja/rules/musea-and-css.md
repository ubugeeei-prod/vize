---
title: Musea と CSS ルール
---

<!-- Generated translation; source: rules/musea-and-css.md -->

# Musea と CSS ルール

Musea ルールは、`<art>` ブロックと `<variant>` ブロックを検証します。 CSS ルールはスタイルのコンテンツを検査し、推奨します
コンポーネントのスタイルをテーマ性、予測可能性、および Vue および Vapor との互換性を維持するパターン。

## `musea/require-title`

すべてのアート ファイルに表示タイトルを指定する必要があります。タイトルは `<art title="...">` から取得できます。
`defineArt("./Button.vue", { title: "..." })`、または `defineArt` コンポーネント ソース フォールバック。

デフォルトの重大度: `error`

悪い：

```vue
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

良い：

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

## `musea/require-component`

すべてのアート ファイルに、文書化されているコンポーネントの名前を付ける必要があります。 `defineArt("./Button.vue", ...)` を優先します。
`<art component="...">` は互換性のために引き続きサポートされます。

デフォルトの重大度: `warning`

悪い：

```vue
<art title="Button">
  <variant name="primary" />
</art>
```

良い：

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

## `musea/valid-variant`

`<variant>` ブロックに有効な `name` が必要です。

デフォルトの重大度: `error`

悪い：

```vue
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

良い：

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## `musea/unique-variant-names`

バリアント名は 1 つのアート ブロック内で一意である必要があります。

デフォルトの重大度: `error`

悪い：

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

良い：

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

## `musea/no-empty-variant`

プロップ、スロット、または視覚的な状態を文書化しない空のバリアントをレポートします。

デフォルトの重大度: `warning`

悪い：

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

良い：

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

## `musea/prefer-design-tokens`

Musea の例では、ハードコードされたプリミティブ値よりもデザイン トークン CSS 変数が優先されます。

デフォルトの重大度: `warning`

悪い：

```vue
<art title="Button" component="./Button.vue">
  <variant name="danger">
    <Button style="color: #d00">Delete</Button>
  </variant>
</art>
```

良い：

```vue
<art title="Button" component="./Button.vue">
  <variant name="danger">
    <Button class="danger">Delete</Button>
  </variant>
</art>

<style scoped>
.danger {
  color: var(--color-danger-text);
}
</style>
```

## `css/no-important`

`!important` を妨げます。

デフォルトの重大度: `warning`

悪い：

```vue
<style scoped>
.button {
  color: red !important;
}
</style>
```

良い：

```vue
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

## `css/no-hardcoded-values`

ハードコードされた色、間隔、サイズの値の代わりに CSS 変数を提案します。

デフォルトの重大度: `warning`

悪い：

```vue
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

良い：

```vue
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

## `css/no-id-selectors`

ID セレクターはオーバーライドや再利用が難しいため、コンポーネント スタイルの ID セレクターは推奨されません。

デフォルトの重大度: `warning`

悪い：

```vue
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

良い：

```vue
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

## `css/no-display-none`

CSS でコンポーネントのブランチを非表示にする代わりに、Vue の可視性プリミティブを使用することを提案します。

デフォルトの重大度: `warning`

悪い：

```vue
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

良い：

```vue
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

## `css/no-v-bind-performance`

ホット スタイルの CSS `v-bind()` の実行時コストについて警告します。

デフォルトの重大度: `warning`

悪い：

```vue
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

良い：

```vue
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

## `css/prefer-logical-properties`

国際化されたレイアウトの論理プロパティを推奨します。

デフォルトの重大度: `warning`

悪い：

```vue
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

良い：

```vue
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

## `css/prefer-slotted`

スロット コンテンツをスタイリングする場合は、`::v-slotted()` を推奨します。

デフォルトの重大度: `warning`

悪い：

```vue
<style scoped>
.content h2 {
  margin-block: 0;
}
</style>
```

良い：

```vue
<style scoped>
::v-slotted(h2) {
  margin-block: 0;
}
</style>
```

## `css/require-font-display`

`@font-face` 宣言には `font-display` が必要です。

デフォルトの重大度: `warning`

悪い：

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

良い：

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

## 追加の CSS ルール

`css/no-utility-classes` は、コンポーネント スタイル内にユーティリティ クラスを実装することに対して警告します。デフォルト:
`warning`。

`css/prefer-nested-selectors` では、子孫セレクターの CSS ネストを推奨しています。デフォルト: `warning`。
