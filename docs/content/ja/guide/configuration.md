---
title: 設定
description: 既存の Vite 設定と TypeScript プロジェクトで Vize を設定する。
---

<!-- Reviewed translation; source: guide/configuration.md -->

# 設定

Vize の設定は `vite.config.ts` に、TypeScript のプロジェクト設定は `tsconfig.json` にまとめます。既定の設定で使う場合、専用の Vize 設定ファイルは不要です。

> [!NOTE]
> ネイティブ CLI・エディターの Vite 設定の読み込みと、専用設定を作らない init は、次のリリースに向けて準備中です。それまでは、公開済みのネイティブツールで共通設定を変更するには既存の専用形式を使います。[リファレンス](./configuration-reference.md)にその形式をまとめています。

## プロジェクトとエディターの既定値

次回リリースでは、専用の `vize.config.*` がないプロジェクトに以下の既定値を適用する予定です。公開済みリリースは従来の動作を維持します。

| 機能 | 専用設定なし（`vite.config.*` を含む） | 既存の専用設定あり |
| --- | --- | --- |
| `vize check`・`vize lsp` の Vue JSX 型チェック | 明示的に無効化しなければ有効 | `typeChecker.jsxTypecheck: true` で有効化 |
| LSP の文書・範囲・入力時フォーマット | プロジェクトやエディターで無効化しなければ有効 | 既存のオプトイン動作を維持 |
| フォーマットの書式設定 | 既存の既定値と明示的な設定を維持 | 既存の既定値と明示的な設定を維持 |

React 用、または React と Vue JSX が混在するプロジェクトでは、Vite 設定の `vize.typeChecker.jsxTypecheck` を `false` にします。JSX の構文からフレームワークは判別しません。Vue SFC の型チェックは引き続き利用できます。[JSX の型チェック](./jsx.md#型チェック)を参照してください。

明示的なエディター設定は、プロジェクトの既定値より優先されます。VS Code の lint 専用プロファイルはフォーマットを無効にし、`vp run editor:setup` は既存の選択を維持しながら明示的に有効化します。[エディター設定（英語）](../../guide/vite-plus-editor.md)に担当範囲をまとめています。

## Vite+ の設定

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: { sourceMap: true },
  lint: { vize: { preset: "essential" } },
  fmt: { vize: { printWidth: 100 } },
  typecheck: { strict: true },
});
```

| 設定場所 | 対象 | 実行 |
| --- | --- | --- |
| `compiler` | Vue のコンパイル | `vp dev`, `vp build` |
| `lint.vize` | Vue の lint ルール | `vp run lint` |
| `fmt.vize` | Vue のフォーマット | `vp run fmt:check` |
| `typecheck` | Vue の型チェック | `vp run typecheck` |
| `pack.vize` | ライブラリの型宣言 | `vp run pack` |

まとめて検査するには `vp run check` を使います。組み込みの `vp check`・`vp lint`・`vp fmt` は Vite+ 自身の動作を維持します。既存スクリプトがある場合、生成タスクは `vize:<名前>` になります。[タスク名と上書き](./vite-plus.md#tasks)を参照してください。

## ルールを1つ変更する

Vue のルールは `lint.vize.rules`、Oxlint のルールは `lint.rules` に指定します。[ルールのオプション](../rules/options.md)で設定値を、[ルール一覧](../rules/all.md)で具体例を確認できます。

```ts
export default defineConfig({
  lint: {
    vize: { rules: { "vue/no-v-html": "error" } },
    rules: { "no-debugger": "error" },
  },
});
```

<span id="lint-rule-options"></span>

### ルールのオプション

[ルールのオプション](../rules/options.md)に、設定オブジェクトの型と悪い例・良い例をまとめています。未対応のオプション名は拒否されます。

## 導入する機能を選ぶ

`compiler`・`typecheck`・`lint.vize`・`fmt.vize` を `false` にすると、その機能を無効にできます。Vue は Vize、その他のファイルは Oxfmt がフォーマットします。[担当範囲と重複の扱い](./vite-plus.md#lint-and-formatter-ownership)も確認してください。

## 通常の Vite

コンパイルにはプラグインを使い、CLI やエディターと共有する設定はトップレベルの `vize` に置きます。プラグインの import で Vite の設定の型も追加されます。

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  vize: {
    linter: { preset: "essential" },
    formatter: { printWidth: 100 },
    typeChecker: { strict: true },
  },
});
```

<span id="standalone-cli"></span>

## 単独 CLI

`vize` をインストールし、対象パッケージのルートから実行します。CLI も `vite.config.*` と TypeScript プロジェクトを読み込みます。Vite+ の `compiler`・`typecheck`・`lint.vize`・`fmt.vize` もネイティブコマンド用に変換されます。トップレベルの `vize` に明示した設定は、この変換より優先されます。

```bash
vp install -D vize
vp exec vize check
```

CLI の設定の検索は、最も近い `package.json`・`tsconfig.json`・`jsconfig.json` のあるディレクトリで止まります。モノリポジトリでは対象パッケージから実行するか `vize.entries` を使い、エディターのワークスペースフォルダーも明示してください。

次回リリースのエディターは、選択したワークスペースの範囲内で、ファイルごとに最も近いプロジェクトのルートを選びます。`package.json`・`tsconfig*.json`・`jsconfig.json`・`vite.config.*`・`vize.config.*` のあるディレクトリがルートになります。そこに Vite・Vize の設定がなければ、新規プロジェクトの既定値を使います。たとえば `package/src/tsconfig.json` があれば、`package/src` を独立したプロジェクトとして扱います。設定を読み込めなければ、従来のエディターの既定値を維持します。CLI で `--config` を明示した場合、指定したファイルがない、または読み込めないときはエラーになります。

Vite 設定を使い、入力引数なしで `build`・`lint`・`fmt`・`check` を実行すると、選択した Vite の `root` が対象になります。相対指定の `root` は設定ファイルのディレクトリを基準に解決します。Vite の `typeChecker` 内のパスとスコープの `basePath` はその root を基準にし、専用設定内のパスは従来どおり設定ファイルのディレクトリを基準にします。CLI へ明示するファイル・glob・`--tsconfig` は、コマンドを実行したディレクトリが基準です。

共有の全体 ignore は、CLI のファイル検索とエディターの lint 対象からファイルを除外します。除外したファイルでも、エディターで開けば構文・型・ナビゲーションの診断を利用できます。`!` による再指定を含め、ignore の順序とパターンの意味を保持します。

## 専用設定ファイルを使う場合

既存の `vize.config.*` も使えます。同じディレクトリでは Vite の設定より優先されます。CLI の `--config` でファイルを明示できます。プラグインに直接渡したオプションやエディターで明示した機能の設定は、共通設定より優先されます。`config: false` でプラグインの自動読み込みを無効にできます。

## 詳細リファレンス

[共通設定のリファレンス](./configuration-reference.md)に、検索規則、専用の TypeScript・JSON・PKL 設定、スコープ別の設定、Vue の型解決、LSP・Musea の設定をまとめています。コンパイラの設定と構文は[コンパイラのリファレンス](./compiler-configuration-reference.md)、実験的な機能は [Experimentals](./experimentals.md)を参照してください。
