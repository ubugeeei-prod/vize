---
title: コンパイラインスペクタ
---

<!-- Reviewed translation; source: guide/compiler-inspector.md -->

# コンパイラインスペクタ

プレイグラウンドの Compiler Inspector は、`.vue` の再現例を調べるためのコンパイラ出力と解析情報をまとめて表示します。
公式 Vue SFC コンパイラと Vize の出力、仮想 TypeScript、VIR、ローカルの複数ファイルにまたがるグラフを確認できます。

次の URL から開きます。

```bash
https://vizejs.dev/play/?tab=inspector
```

ブラウザでは、次の出力と操作を利用できます。

- `@vue/compiler-sfc` による比較元の出力
- Vize WASM による Vize の出力
- 選択したファイルの、Canon による仮想 TypeScript と Croquis の VIR
- CLI と共有する、ネイティブの `vize_curator` によるグラフと差分のメタデータ
- ペイロード内のファイルに対するクロスファイル診断
- DOM または SSR の対象選択
- カスタムレンダラーとテンプレート構文モードの切り替え
- 両コンパイラの完全な出力タブ
- Vue だけ、Vize だけに含まれる行を表示する比較タブ
- パーマリンクと、必要な内容を入力済みの PR 作成リンク

## CLI ペイロード

ローカルプロジェクトに再現例がある場合は、`vize inspector` を使います。
単一ファイルを指定すると、デフォルトでプレイグラウンドの URL を出力します。

```bash
vize inspector src/App.vue
```

ディレクトリや glob を指定すると、複数ファイルのペイロードを作ります。
プレイグラウンドで開き、表示するファイルを切り替えられます。

```bash
vize inspector src/components
vize inspector "src/**/*.vue" --target ssr
```

ファイル数が多い場合は、長い URL の代わりに JSON を出力できます。

```bash
vize inspector "src/**/*.vue" --format json --output inspector-payload.json
```

AI エージェントへの受け渡しや、端末での作業の引き継ぎには、エージェント用レポートを出力します。
ペイロード、プレイグラウンドの URL、集計値、クロスファイルグラフのメタデータを含みます。

```bash
vize inspector "src/**/*.vue" --format agent --output inspector-agent.json
```

開発用のチェックアウトでは、CLI から直接コンパイラを比較することもできます。
現在のバイナリ内の Rust コンパイラを使い、プロジェクトまたは Vize ワークスペースの `node_modules` から `@vue/compiler-sfc` を読み込みます。

```bash
vize inspector "src/**/*.vue" --format compare --output inspector-compare.json
```

ペイロードとエージェント用レポートは、ローカル専用の Rust クレート `vize_curator` が生成します。
プレイグラウンドの WASM バインディングも、グラフと行の差分には同じクレートを使います。
そのため、CLI の複数ファイルのレポートとブラウザ表示で同じ情報を扱えます。
ブラウザの比較では、公式 Vue コンパイラもブラウザ内で実行します。

主なオプションは次のとおりです。

| オプション | 説明 |
| --- | --- |
| `--target dom` | VDOM コンパイラの出力を比較する |
| `--target ssr` | SSR コンパイラの出力を比較する |
| `--format agent` | グラフのメタデータを含むエージェント用 JSON を出力する |
| `--format compare` | 開発環境で Vue と CLI の出力を比較する |
| `--custom-renderer` | プレイグラウンドのカスタムレンダラーモードを有効にする |
| `--template-syntax` | `standard`、`strict`、`quirks` のいずれかを選ぶ |
| `--max-files <n>` | ペイロードに含めるファイル数を制限する |
| `--playground-url` | リンクに使うプレイグラウンドの URL を指定する |

## PR ワークフロー

コンパイラの互換性を修正する PR には、本文にインスペクタのパーマリンクを載せます。
併せて最小限のフィクスチャまたは完全なスナップショットを追加し、CI で出力の変化を確認できるようにします。
内容を入力済みの PR リンクは、作成の出発点として使ってください。
ブランチを push した後、GitHub で比較対象の変更を求められた場合は、その head を差し替えます。

レビューに必要な証拠は次のとおりです。

- インスペクタのパーマリンク
- 選択した対象とオプション
- 最小化した `.vue` フィクスチャ、または完全なスナップショット
- 複数のコンパイラ機能にまたがる修正では、関連する仮想 TypeScript、VIR、グラフ
- Vize の出力を Vue と一致させる理由、または意図的に異なる出力にする理由
- 修正した機能を確認するローカルの検証コマンド
