<!-- Reviewed translation; source: guide/formatter-jsx-quotes.md -->

# JSX 属性の引用符を設定する

JSX 属性の引用符を、通常の JavaScript 文字列とは別に選びます。
JSX/TSX のコードや Vue のスクリプトブロックで、文字列と属性に異なる規則を使いたいときに設定します。

共通の設定は `vite.config.*` の `vue.formatter` に置きます。CLI とエディターは同じ設定を読み込みます。[CLI・エディターとの設定共有](./configuration.md)を確認してください。

## JSX の引用符を設定する

`vue.formatter.jsxSingleQuote` を指定すると、JSX/TSX 属性でシングルクォートを優先できます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: { singleQuote: false, jsxSingleQuote: true },
});
```

`singleQuote` は通常の JavaScript 文字列の引用符を選びます。
`jsxSingleQuote` は JSX 属性の引用符を独立して選び、既定値は `false` です。
たとえば、上の設定ではスクリプトを次のように整形します。

```text
const label = "hello";
const view = <div title='world' />;
```

エスケープが少なくなる引用符を優先します。
`title="it's"` のようにアポストロフィーを含む属性は、文字参照を増やさないようにダブルクォートを保ちます。
引用符の数が同じ場合は設定に従い、`title="' &quot;"` は `title='&apos; "'` になります。

この設定は JSX/TSX のスクリプトに適用され、Vue の `<script lang="tsx">` と `<script lang="jsx">` も対象です。
通常の Vue テンプレートの HTML 属性は、従来の引用符の規則を保ちます。
CLI で直接上書きする場合は `vize fmt --jsx-single-quote` を使います。
Node の native API では `formatSfc` のオプションに `{ jsxSingleQuote: true }` を渡します。
設定、CLI、エディター、WASM、Node は同じ真偽値を使います。

[プロパティの引用符の設定](./formatter-property-quotes.md)では、どのオブジェクトキーに引用符を付けるかを選びます。
JSX 属性の引用符とは独立した設定です。
