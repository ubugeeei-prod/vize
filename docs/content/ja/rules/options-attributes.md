---
title: 属性名のオプション
---

# 属性名のオプション

<span id="vue-attribute-hyphenation"></span>

## `vue/attribute-hyphenation`：コンポーネントに渡す属性名

カスタムコンポーネントに渡す、静的な props の属性名を指定するためのオプションです。
`always` は camelCase で書かれた属性を報告し、`never` はハイフン区切りで書かれた属性を報告します。
通常の要素の属性、`aria-*`、`data-*`、SVG の大文字と小文字が混在する属性、
動的な引数、属性として解析されるディレクティブの省略記法は対象外です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "vue/attribute-hyphenation": "warn"
      },
      "ruleOptions": {
        "vue/attribute-hyphenation": "never"
      }
    }
  }
});
```

上の設定で報告される例：

```vue
<template>
  <UserCard user-name="Ada" />
</template>
```

設定に沿った例：

```vue
<template>
  <UserCard userName="Ada" aria-label="Ada Lovelace" />
</template>
```

### 属性名の例外

文字列の設定はそのまま使えます。追加の例外は `{"style":"always","ignore":["libraryProp"]}` で指定します。`ignore` は大文字小文字を区別する部分文字列で、正規表現や glob ではありません。`data-`、`aria-`、`slot-scope`、SVG などの既存の例外も保持します。

`style` の省略時は `always` になります。空の配列や空のオブジェクトは追加の例外をリセットします。空白だけの値、`:exit` で終わる値、重複する値は拒否します。ルールの有効化と severity は `linter.rules` で指定します。

Oxlint のルール設定では `["warn", "always", {"ignore":["libraryProp"]}]` も使えます。ネイティブ API では `attributeHyphenationIgnore` または同じオブジェクトを含む `ruleOptionsJson` に渡します。
