---
title: Lint の実行設定
---

<!-- Reviewed translation; source: guide/lint-execution.md -->

<span id="native-lint-execution-settings"></span>

# Lint の実行設定

ファイル間の関係も調べたい場合や、警告を CI の失敗条件にしたい場合に使うガイドです。
CLI と Vite+ のタスクで同じ方針を使うには、実行時のオプションを設定ファイルにまとめてください。
最初の実行方法とコマンドのオプションは、[CLI の Lint](./cli.md#lint)を参照してください。
ファイル間の関係を示すレポートは、[コンポーネントツリーと複雑度](./cross-file-complexity.md)で説明しています。

`vite.config.*` では、`withVue` に渡す `linter` を CLI とエディタで共有します。
Oxlint の flat config はルートの `lint` に置き、Vize 固有の設定は次のように分けます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    preset: "opinionated",
    crossFile: true,
    strictReactivity: true,
    crossFileTree: true,
    crossFileComplexity: true,
    maxWarnings: 0,
    reportUnusedDisableDirectives: true,
  },
});
```

| 設定                            | 既定値       | 動作                                                                       |
| ------------------------------- | ------------ | -------------------------------------------------------------------------- |
| `preset`                        | `happy-path` | 有効にするルールの組み合わせを選びます。                                   |
| `crossFile`                     | `false`      | 選択した Vue ファイル間の関係を検査します。                                |
| `crossFileTree`                 | `false`      | provide/inject のツリーを表示し、ファイル間解析を有効にします。            |
| `crossFileComplexity`           | `false`      | 複雑度とホットスポットを表示し、ファイル間解析を有効にします。             |
| `strictReactivity`              | `false`      | `type/no-reactivity-loss` を有効にします。ルールの個別設定が優先されます。 |
| `maxWarnings`                   | 未指定       | 警告数が上限を超えると失敗します。`0` は警告を一つも許可しません。         |
| `reportUnusedDisableDirectives` | `false`      | Vize の診断を抑制しなかった、対応済みの抑制コメントを報告します。          |

CLI とエディタで共有する場合は、これらのキーを `withVue` の `linter` に指定します。
`crossFileTree` と `crossFileComplexity` は、ファイル間の解析も有効にします。
ファイル間のチェックは、Vize の CLI 実行で対象に選ばれたファイルに対して行います。

`strictReactivity` は、CLI とエディターの両方で `type/no-reactivity-loss` を有効にします。
ルールを個別に指定した場合は、そちらが優先されます。ファイルの範囲を指定した `off` も優先されます。

`vize lint` を直接実行する場合は、コマンドのオプションで必要なチェックを有効にする方法も引き続き使えます。
`--max-warnings` は設定ファイルの警告数の上限より優先され、`--no-config` は Lint の設定をすべて無視します。
警告数の上限を指定しない場合は、警告だけで実行を失敗とはしません。
`0` を指定すると、警告が一つでも残っている場合に失敗します。負の値は指定できません。

単独の JavaScript/TypeScript ファイルと SFC の script ブロックでは、
`// vize-disable-next-line script/rule-name`、行末の
`// vize-disable-line script/rule-name`、または
`/* vize-disable script/rule-name */` から `/* vize-enable script/rule-name */`
までの範囲で抑制できます。ルール名を省略するとすべてのルールが対象になり、
複数のルールはカンマで区切ります。Oxlint の未使用ディレクティブのチェックと併用できます。
文字列、テンプレートリテラル、正規表現の中のマーカーは抑制として扱いません。
テンプレートのコメントは従来の `eslint-` / `oxlint-` 構文を使います。
CSS の `vize-disable` コメントの動作と、Vize コメントを未使用として報告しない動作は変わりません。

Vite+ の Lint タスクには、対象のパスと `--fix` を渡せます。
Vize 固有のオプションは、Oxlint に渡すのではなく `withVue` の `linter` に指定してください。
たとえば `vp run lint -- src` を実行すると、それぞれの設定を使って両方の Lint が実行されます。

## 構文解析とプロジェクト設定の診断

Lint のレポートには、ソースの構文解析や型情報を使うプロジェクトの設定読み込みに
失敗した診断も含まれます。これらの診断 ID はプリセットのルール一覧には含まれません。
メッセージと参照ページに従って入力を修正し、Lint を再実行してください。

| 診断                         | 参照ページ                                                                        |
| ---------------------------- | --------------------------------------------------------------------------------- |
| `parser/template`            | [テンプレート構文とパーサーの回復](../rules/reference/parser-template.md)         |
| `parser/sfc`                 | [単一ファイルコンポーネントの構造](../rules/reference/parser-sfc.md)              |
| `parser/jsx`                 | [JSX/TSX の構文解析と変換](../rules/reference/parser-jsx.md)                      |
| `type/project-configuration` | [TypeScript プロジェクトの設定](../rules/reference/type-project-configuration.md) |

<span id="unused-disable-directives"></span>

## 使われていない抑制コメント

`reportUnusedDisableDirectives: true` を指定すると、使われていない抑制コメントを
`vue/comment-directive` の error として、そのコメントの位置に報告します。
これは明示的に有効化する実行設定で、プリセットのルールではありません。
診断 ID を `rules` に追加するだけでは有効になりません。既定は無効で、
設定の `false` は対象のファイル設定での報告を無効にします。
`--report-unused-disable-directives` を明示すると、設定が `false` でも有効になります。

テンプレートでは `eslint-disable`、`eslint-disable-line`、
`eslint-disable-next-line` と、それぞれの `oxlint-` 形式を検査します。
実際の診断を抑制したコメントは使用済みとして扱います。
対象は Vize が有効にして実行したルールです。無効なルールは数えません。
ほかのリンターが担当するルール名の未使用チェックは、そのリンターに任せます。

ファイル間解析を有効にした場合は、選択した Vue ファイルの、対応済みの名前付き
プロジェクトルールの抑制コメントも対象になります。対応する script ブロック内の
コメントも含みます。任意の script/style のディレクティブや、すべての
`vize-disable` 形式への一般的な対応を意味しません。文字列・属性値・カスタム
ブロック内のマーカーが、抑制コメントとして扱われることもありません。

抑制範囲は[コメント注釈](./comment-annotations.md)、プロジェクトの検査は
[ファイル間ルール](../rules/cross-file.md)を参照してください。このネイティブの
報告設定は、`eslint-plugin-vue` の comment-directive との完全な互換性を保証しません。

## 静的な class 属性の既定のチェック

`html/no-duplicate-class` は、`class="button button"` のように、静的な
`class` 属性で同じクラス名が繰り返されている場合に警告します。
動的なバインディングは対象にせず、自動修正も行いません。

| プリセット                 | 変更前の既定値 | 変更後の既定値 |
| -------------------------- | -------------- | -------------- |
| `happy-path`、`ecosystem`  | 無効           | 警告           |
| `opinionated`、`nuxt`      | 警告           | 警告           |
| `essential`、`incremental` | 無効           | 無効           |

`rules` で個別に指定すると、警告を無効にしたり、重大度を変更したりできます。
`incremental` は引き続き空の状態から始まります。採用する場合は、このルールを明示的に有効にしてください。
