<!-- Reviewed translation; source: guide/formatter-import-sorting.md -->

# Vue スクリプトの import を並べ替える

Vue スクリプトの import を、Vite+ プロジェクト内のほかのファイルと同じ規則で並べ替えます。
import のグループをそろえたいときに使います。副作用のある import の初期化順は、明示的に変更する場合を除き保たれます。

共通の設定は `vite.config.*` の `vue.formatter` に置きます。CLI とエディターは同じ設定を読み込みます。[Vue 設定ガイド](./configuration.md)を参照してください。

## import の並べ替えを設定する

既存の Vite+ 設定で `vue.formatter.sortImports` を指定すると、import の並べ替えを有効にできます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: {
    sortImports: {
      internalPattern: ["@/", "~/"],
      groups: ["builtin", "external", "internal", ["parent", "sibling", "index"]],
      newlinesBetween: true,
      order: "asc",
    },
  },
});
```

空のオブジェクトを指定すると、Oxfmt の既定のグループと並べ替え方を使います。
`false` にすると、元の import の順序を保ちます。共有の `fmt.sortImports` が未設定なら、省略しても元の順序を保ちます。
`<script>` と `<script setup>` の両方で、JSX/TSX を含めて native Oxc の import sorter を使います。
`vize fmt` を直接実行した場合も、単独の JS/TS/JSX/TSX ファイルに同じ規則を適用します。

Vite+ では `fmt.sortImports` を Vue のフォーマットにも自動的に共有します。
`vue.formatter.sortImports` で上書きでき、`false` を指定すると Vue だけ並べ替えを無効にできます。
Oxfmt には、元の設定がそのまま渡ります。

指定できるオプションは `groups`、`customGroups`、`internalPattern`、`order`、
`ignoreCase`、`newlinesBetween`、`partitionByNewline`、`partitionByComment`、`sortSideEffects` です。
グループの各項目には、名前、名前の配列、または `{ newlinesBetween: boolean }` による境界を指定できます。
カスタムグループでは、Oxfmt と同じ `groupName`、`elementNamePattern`、`selector`、`modifiers` を使えます。

副作用のある import は、既定では元の順序を保ちます。初期化順を意図的に変える場合にだけ `sortSideEffects: true` を指定してください。
`partitionByNewline: true` を使うには `newlinesBetween: false` が必要で、境界ごとの改行指定とは併用できません。
無効な組み合わせはファイルを書き換える前にエラーになります。`--no-config` を指定すると、設定された並べ替え規則を無視します。

native の `formatSfc` と WASM の `formatSfc` / `formatScript` でも、オプションに `sortImports` を指定できます。
既存の Rust の `FormatOptions` はソース互換性を保ちます。
追加の Rust API を使う場合は、`resolve_sort_imports` と `GlyphFormatter::with_sort_imports`、
または `format_script_with_sort_imports` を組み合わせてください。

CLI、エディタ、Node、WASM で同じ `endOfLine` のモードを使う方法は、[改行コードの設定](./formatter-line-endings.md)を参照してください。

スクリプトのオブジェクトキーについては、[プロパティの引用符の設定](./formatter-property-quotes.md)を参照してください。

JavaScript の文字列とは別に、[JSX 属性の引用符を設定](./formatter-jsx-quotes.md)できます。

スペースとタブによる字下げは、[インデント幅の設定](./formatter-indent-width.md)で説明しています。

CLI とエディターは `vite.config.*` の `vue.formatter` を共有します。設定の探索方法と、廃止された専用設定ファイルからの移行については、[設定ガイド](./configuration.md)を参照してください。
