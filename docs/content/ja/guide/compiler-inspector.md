---
title: コンパイラーインスペクター
---

<!-- Reviewed translation; source: guide/compiler-inspector.md -->

<span id="コンパイラインスペクタ"></span>

# コンパイラーインスペクター

Compiler Inspector は、同じ `.vue` ファイルから Vue と Vize が生成するコードを比較するツールです。
「このテンプレートで出力が変わる」「SSR の結果がおかしい」といった問題を、小さな再現例に絞り込むときに使います。
生成された仮想 TypeScript、VIR（Vize の中間表現）、複数ファイルの依存関係も確認できます。

[プレイグラウンドの Compiler Inspector](https://vizejs.dev/play/?tab=inspector)を開いてください。
再現例を共有するときは、[テストとフィードバック](./testing.md)の手順に沿って情報をまとめてください。

```bash
https://vizejs.dev/play/?tab=inspector
```

## 出力を比較する

1. 問題が起きる `.vue` ファイルの内容を入力します。再現する状態を保ちながら、関係のない処理やスタイルを取り除いてください。
2. 調べたい出力を選びます。通常のクライアント用コードは DOM、サーバー用コードは SSR を選び、再現時と同じ構文モードやレンダラーの設定にそろえます。
3. 比較タブで差がある行を探し、それぞれの出力タブで前後のコードも読みます。生成コードに差があるだけでは、不具合とは限りません。実際の表示や処理にどう影響するかも確認してください。
4. 再現例ができたら、パーマリンクをコピーします。報告に添える内容は[テストとフィードバック](./testing.md)を参照してください。

ブラウザーでは、次の情報を確認できます。

- 公式の `@vue/compiler-sfc` と Vize WASM がそれぞれ生成したコード
- 選択したファイルの仮想 TypeScript（`vize check` がチェックするコード）と VIR
- ファイル間のグラフと行ごとの差分情報。CLI と同じ方法で作るため、ターミナルのレポートと一致します。
- 読み込んだファイル間の関係から得られる診断
- DOM・SSR・Vapor の切り替え、カスタムレンダラーとテンプレート構文モードの設定
- 両コンパイラーの出力全体と、片方にだけ含まれる行を示す比較表示
- 同じ再現例を開くパーマリンクと、再現情報を入力済みの PR 作成リンク

<span id="cli-ペイロード"></span>

## ローカルの再現例を読み込む

ローカルプロジェクトに再現例がある場合は、`vize inspector` を使います。
単一ファイルを指定すると、デフォルトでプレイグラウンドの URL を出力します。

```bash
vize inspector src/App.vue
```

親子コンポーネントなど、複数ファイルが必要な問題では、ディレクトリや glob を指定します。
関連するソースを一つのデータにまとめ、プレイグラウンドでファイルを切り替えて調べられます。
glob は引用符で囲み、シェルが先に展開しないようにします。

```bash
vize inspector src/components
vize inspector "src/**/*.vue" --target ssr
```

ファイル数が多く URL が長くなる場合は、JSON ファイルに保存できます。

```bash
vize inspector "src/**/*.vue" --format json --output inspector-payload.json
```

AI エージェントに調査を引き継ぐ場合は、エージェント用レポートを使えます。
ソースをまとめたデータ、プレイグラウンドの URL、集計値、ファイル間のグラフ情報を含みます。

```bash
vize inspector "src/**/*.vue" --format agent --output inspector-agent.json
```

開発用のチェックアウトでは、CLI から直接コンパイラーを比較することもできます。
実行中の CLI に含まれる Rust コンパイラーと、プロジェクトまたは Vize ワークスペースの `node_modules` にある `@vue/compiler-sfc` を使います。

```bash
vize inspector "src/**/*.vue" --format compare --output inspector-compare.json
```

この `compare` 形式には、Vue 3 の `@vue/compiler-sfc` または `vue/compiler-sfc` が必要です。
Vue 2・Nuxt 2 のプロジェクトには対応していません。公式コンパイラーを実行せずに再現データを取り出す場合は、`json` または `agent` 形式を使ってください。

CLI とプレイグラウンドは、再現データ、グラフ、行ごとの差分を同じ方法で作るため、ターミナルのレポートは
ブラウザーの表示と一致します。公式の Vue コンパイラーは常にブラウザー内で実行します。

主なオプションは次のとおりです。

| オプション | 説明 |
| --- | --- |
| `--target dom` | クライアント用の VDOM コードを比較する |
| `--target ssr` | SSR 用のコードを比較する |
| `--target vapor` | Vapor 用のコードを比較する |
| `--format agent` | グラフ情報を含むエージェント用 JSON を出力する |
| `--format compare` | 開発環境で Vue と CLI の出力を比較する |
| `--custom-renderer` | プレイグラウンドのカスタムレンダラーモードを有効にする |
| `--template-syntax` | `standard`、`strict`、`quirks` のいずれかを選ぶ |
| `--max-files <n>` | 読み込むファイル数の上限を指定する |
| `--playground-url` | リンクに使うプレイグラウンドの URL を指定する |

<span id="pr-ワークフロー"></span>

## 修正をレビューできる形で共有する

コンパイラーの互換性を修正する PR には、本文にインスペクターのパーマリンクを載せます。
併せて最小限のフィクスチャまたは完全なスナップショットを追加し、CI で出力の変化を確認できるようにします。
インスペクターの PR 作成リンクでは、再現情報が本文に入ります。
修正ブランチを push したら、GitHub の `base` に変更を取り込むブランチ、`compare` に自分の修正ブランチを選んでください。

レビューに必要な証拠は次のとおりです。

- インスペクターのパーマリンク
- 選択したコンパイル先とオプション
- 最小化した `.vue` フィクスチャ、または完全なスナップショット
- 複数の処理にまたがる修正では、関連する仮想 TypeScript、VIR、グラフ
- Vize の出力を Vue と一致させる理由、または意図的に異なる出力にする理由
- 修正した機能を確認するローカルの検証コマンド
