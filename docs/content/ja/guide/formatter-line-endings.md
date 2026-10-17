<!-- Reviewed translation; source: guide/formatter-line-endings.md -->

# フォーマッタの改行コードを設定する

Vue ファイル全体の改行コードをそろえます。
プロジェクトの規則が決まっている場合は明示的なモードを、入力の最初の改行コードに合わせたい場合は auto を選びます。
整形しない本文のバイト列は保たれます。

共通の設定は `vite.config.*` の `vue.formatter` に置きます。CLI とエディターは同じ設定を読み込みます。[Vue 設定ガイド](./configuration.md)を参照してください。

## 改行コードを設定する

既存の Vite+ 設定で `vue.formatter.endOfLine` を指定します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: { endOfLine: "auto" },
});
```

モードは `"lf"`（既定値）、`"crlf"`、`"cr"`、`"auto"` です。
auto は入力の最初の改行コードを選びます。隣り合う CRLF なら CRLF、単独の CR なら CR、
LF または改行がない入力なら LF を選びます。
Vue ファイルでは、テンプレート、スクリプト、スタイルの各ブロックに使うレイアウト上の改行コードを一度だけ選びます。
明示的なモードを指定した場合は、入力のレイアウトより優先します。整形しない本文は、元のバイト列を保ちます。

Node と WASM の既存のフォーマッタオプションでも、同じ値を指定できます。

```ts
import { formatSfc } from "@vizejs/native";

const source = "<template>\r\n<p>hello</p>\n</template>\n";
const result = formatSfc(source, { endOfLine: "auto" });
const again = formatSfc(result.code, { endOfLine: "auto" });
// again.changed === false
```

Node は、未対応のモード文字列を `InvalidArg`、文字列以外の値を `StringExpected` として拒否します。
どちらの診断も `endOfLine` フィールドを示します。
整形結果は、従来どおり `code` と `changed` フィールドで返します。

CLI とエディターは `vite.config.*` の `vue.formatter` を共有します。設定の探索方法と、廃止された専用設定ファイルからの移行については、[設定ガイド](./configuration.md)を参照してください。
