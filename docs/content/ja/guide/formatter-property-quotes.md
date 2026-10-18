<!-- Reviewed translation; source: guide/formatter-property-quotes.md -->

# プロパティの引用符を設定する

スクリプトのオブジェクトキーに引用符を付ける条件を選びます。
元のキーに引用符があったかを保ちたい場合は preserve、オブジェクトの構造に合わせてそろえたい場合はほかのモードを使います。

設定の置き場所を選ぶ前に、[CLI・エディターとの設定共有と、リリースごとの対応範囲](./configuration.md)を確認してください。

## プロパティの引用符を設定する

既存の Vite+ 設定で `fmt.vize.quoteProps` を指定し、スクリプトのオブジェクトキーの引用符を選びます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  fmt: {
    vize: { quoteProps: "preserve" },
  },
});
```

使えるモードは次のとおりです。

- `"as-needed"`（既定値）：引用符が必要なキーにだけ付けます。
- `"consistent"`：引用符が必要なキーが1つでもある場合は、そのオブジェクト内のすべてのキーに付けます。必要なキーがなければ、任意の引用符を外します。
- `"preserve"`：それぞれのキーに、元のコードで引用符が付いていたかどうかを保ちます。

たとえば、`preserve` は `{"name":1,plain:2}` を `{ "name": 1, plain: 2 }` に整形します。
`as-needed` と `consistent` は、どちらもこのオブジェクトを `{ name: 1, plain: 2 }` に整形します。
オブジェクトに `"needs-dash"` がある場合、`consistent` は同じオブジェクトのほかのキーにも引用符を付けます。

引用符の文字自体は、引き続き `singleQuote` で選びます。
`quoteProps` はどのキーに引用符を付けるかを指定するもので、元の引用符の文字を保つ設定ではありません。

CLI・エディターとの設定共有や、必要に応じて使う専用設定については、[設定ガイド](./configuration.md)でリリースごとの対応範囲を確認してください。
