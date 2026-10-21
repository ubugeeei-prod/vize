<!-- Reviewed translation; source: guide/formatter-property-quotes.md -->

# プロパティの引用符を設定する

スクリプトのオブジェクトキーに引用符を付ける条件を選びます。
元のキーに引用符があったかを保ちたい場合は preserve、オブジェクトの構造に合わせてそろえたい場合はほかのモードを使います。

共通の設定は `vite.config.*` の `vue.formatter` に置きます。CLI とエディターは同じ設定を読み込みます。[CLI・エディターとの設定共有](./configuration.md)を確認してください。

## プロパティの引用符を設定する

既存の Vite+ 設定で `vue.formatter.quoteProps` を指定し、スクリプトのオブジェクトキーの引用符を選びます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: { quoteProps: "preserve" },
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

CLI とエディターは `vite.config.*` の `vue.formatter` を共有します。
CLI で直接上書きする場合は `vize fmt --quote-props preserve` を使います。
Node の native API では、`formatSfc` の `quoteProps` オプションに同じ 3 つの文字列を指定できます。
設定の探索方法と、廃止された専用設定ファイルからの移行については、[設定ガイド](./configuration.md)を参照してください。
