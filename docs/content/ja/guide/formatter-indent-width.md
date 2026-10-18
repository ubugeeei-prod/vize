<!-- Reviewed translation; source: guide/formatter-indent-width.md -->

# インデント幅を設定する

プロジェクトの規則に合わせて、インデントの幅を選びます。
スペースやタブの設定はテンプレートのレイアウトに反映されますが、整形しない領域の本文は保たれます。

設定の置き場所を選ぶ前に、[CLI・エディターとの設定共有と、リリースごとの対応範囲](./configuration.md)を確認してください。

## インデントを設定する

`tabWidth` は、インデント1段あたりのスペース数を指定します。既定値は `2` です。
既存のフォーマッタ設定で指定してください。

```json
{
  "formatter": {
    "tabWidth": 3
  }
}
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
