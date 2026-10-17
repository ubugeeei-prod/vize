<!-- Reviewed translation; source: guide/formatter-indent-width.md -->

# インデント幅を設定する

プロジェクトの規則に合わせて、インデントの幅を選びます。
スペースやタブの設定はテンプレートのレイアウトに反映されますが、整形しない領域の本文は保たれます。

共通の設定は `vite.config.*` の `vue.formatter` に置きます。CLI とエディターは同じ設定を読み込みます。[Vue 設定ガイド](./configuration.md)を参照してください。

## インデントを設定する

`tabWidth` は、インデント1段あたりのスペース数を指定します。既定値は `2` です。
既存のフォーマッタ設定で指定してください。

```ts
// vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: { tabWidth: 3 },
});
```

`<template><p>hello</p></template>` を整形すると、次のようになります。

```text
<template>
   <p>hello</p>
</template>
```

`useTabs: true` を指定すると、インデントにスペースの代わりにタブを使います。
引用符の選び方は変わりません。
`<pre>`、`<textarea>`、`v-pre` など、テンプレート内の整形しない領域の空白は、元のバイト列を保ちます。
