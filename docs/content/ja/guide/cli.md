---
title: CLI
---

<!-- Reviewed translation; source: guide/cli.md; scope: complete command reference except the shared configuration entry pending source integration -->

# CLI リファレンス

CLI を使うと、Vue ファイルのコンパイル・整形・lint・型チェックをターミナルや CI で実行できます。
アプリでは `vize` npm パッケージをインストールし、まず [package.json のスクリプト](#アプリケーション-パッケージ-スクリプト) を設定してください。
LSP、IDE 管理、`check-server`、プロファイリングには [Rust バイナリ](#rust-バイナリのインストール) を使います。

npm パッケージは共有設定のヘルパーと、NAPI を利用する `build`、`fmt`、`lint`、`check`、`clean`、`ready`、`upgrade` を提供します。
[エントリーポイントの比較](#npm-パッケージ-スクリプトと-rust-cli-の比較) で用途に合うものを選び、具体的な作業順序は [ユーザーワークフロー](./workflows.md) を参照してください。

## アプリケーション パッケージ スクリプト

アプリでは npm からインストールし、日常のコマンドをプロジェクトのスクリプトに登録します。

```bash
vp install -D vize
```

```json
{
  "scripts": {
    "vize:build": "vize build src",
    "vize:fmt": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path src",
    "vize:check": "vize check src",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:lint
vp run vize:check
vp run vize:ready
```

単発のデバッグには `vp exec vize ...` を使い、文書化するワークフローや CI には名前付きスクリプトを使ってください。

## Rust バイナリのインストール

v1 alpha では、GitHub Releases のビルド済みバイナリか Nix のエントリーポイントを使います。crates.io 経由の Rust CLI インストールはまだサポートしていません。

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

OS とアーキテクチャに合うバイナリは [GitHub Releases](https://github.com/ubugeeei-prod/vize/releases) からもダウンロードできます。
このリポジトリでローカル開発をする場合は、ワークスペースのビルドをインストールしてください。

```bash
cargo install --path crates/vize --force --locked
```

## npm パッケージ スクリプトと Rust CLI の比較

| 用途                                                                               | 推奨エントリーポイント           |
| ---------------------------------------------------------------------------------- | -------------------------------- |
| ビルド・整形・lint・型チェック・準備・アップグレードを実行するパッケージスクリプト | npm パッケージの `vp run vize:*` |
| `.vue`、`.ts`、`.tsx`、`.d.ts` を含むプロジェクト全体の型チェック                  | Rust `vize check`                |
| LSP、IDE セットアップ、`check-server`、プロファイル出力                            | Rust `vize` バイナリ             |
| 共有 Vite プラグイン、npm package コマンド、および Rust CLI 設定                   | `vize.config.*`                  |

## コマンド

```bash
vize [COMMAND]
```

コマンドを指定しない場合、`vize` は `build` を実行します。

| コマンド         | 説明                                                                                       |
| ---------------- | ------------------------------------------------------------------------------------------ |
| `build`          | Vue SFC をコンパイルする                                                                   |
| `fmt`            | Vue SFC を整形する                                                                         |
| `lint`           | Vue SFC に lint を実行する                                                                 |
| `check`          | Vue SFC、TS、TSX、`.d.ts` 入力を型チェックする                                             |
| `doctor`         | アプリケーション全体の健全性を診断する                                                     |
| `inspector`      | プレイグラウンドのコンパイラーインスペクター用データを作成する                             |
| `clean`          | Vize が生成したキャッシュを削除する                                                        |
| `lib`            | バージョンを指定して UI・composable のソースを取り込む（[Lib Pull ガイド](./lib-pull.md)） |
| `ready`          | `fmt`、`lint`、`check`、`build` を実行する                                                 |
| `upgrade`        | インストール済み CLI を更新する                                                            |
| `check-server`   | Unix JSON-RPC の型チェックサーバーを起動する                                               |
| `content-mapper` | [TypeScript ContentMapper](./content-mapper.md) サーバーを起動する                         |
| `musea`          | Musea のサブコマンドと雛形作成                                                             |
| `lsp`            | 言語サーバーを起動する                                                                     |
| `ide`            | エディター連携をインストール・管理する                                                     |

すべての `--profile` のターミナル出力は、ローカル用の `vize_curator` クレートが生成します。計測フックは `vize_carton` にあり、`vize_curator` は CLI レポート、インスペクター、エージェント向けデータの形式を管理します。

<a id="建てる"></a>

## ビルド

```bash
vize build src/**/*.vue
vize build --ssr
vize build --profile src
```

| オプション                    | 説明                                                                   |
| ----------------------------- | ---------------------------------------------------------------------- |
| `-o, --output`                | 共通の入力ルートからの相対パスで出力する。同名出力の衝突はエラーにする |
| `-f, --format`                | 出力形式: `js`、`json`、`stats`                                        |
| `--ssr`                       | SSR 用にコンパイルする                                                 |
| `--custom-renderer`           | 小文字の非 HTML タグをカスタムレンダラーの要素として扱う               |
| `--custom-elements <PATTERN>` | カスタム要素として扱うタグのパターン。複数回指定可能                   |
| `--script-ext`                | `preserve` または `downcompile`                                        |
| `--declaration`               | ビルドした SFC の `.d.ts` を出力する（別名: `--dts`）                  |
| `--declaration-dir`           | 宣言ファイルの出力先（デフォルト: ビルドの出力先）                     |
| `-j, --threads`               | スレッド数を指定する                                                   |
| `--profile`                   | 処理時間のプロファイルを表示する                                       |
| `--continue-on-error`         | エラー後もコンパイルを続け、最後に失敗を報告する                       |

## フォーマット

```bash
vize fmt --check src
vize fmt --write src
```

デフォルトのファイル探索では Vue、JavaScript/TypeScript、JSON/JSONC を整形します。YAML と Markdown の整形は未実装です。`.yaml`、`.yml`、`.md`、`.markdown` は探索対象から除外され、明示的に選択するとファイルを変更せずにエラーを返します。

| オプション                         | 説明                                                   |
| ---------------------------------- | ------------------------------------------------------ |
| `--check`                          | 整形すると変更されるファイルを報告する                 |
| `-w, --write`                      | 整形結果をファイルに書き込む                           |
| `--single-quote`                   | 文字列の引用符を切り替える                             |
| `--print-width`                    | 1 行の最大幅                                           |
| `--tab-width`                      | インデント幅                                           |
| `--use-tabs`                       | タブとスペースを切り替える                             |
| `--no-semi`                        | セミコロンを省略する                                   |
| `--sort-attributes`                | テンプレートの属性を並べ替える                         |
| `--single-attribute-per-line`      | 属性を 1 行に 1 つずつ配置する                         |
| `--max-attributes-per-line`        | 指定した属性数で改行する                               |
| `--normalize-directive-shorthands` | `v-bind:` / `v-on:` / `v-slot:` の省略記法を正規化する |
| `--profile`                        | 処理時間のプロファイルを表示する                       |

<a id="糸くず"></a>

## lint

```bash
vize lint src
vize lint --preset opinionated src
vize lint --help-level short src
```

| オプション            | 説明                                                                                                                                |
| --------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `--fix`               | テキスト編集を提供するルールの安全な自動修正を適用し、残った診断を報告する                                                          |
| `-f, --format`        | 出力形式: `text`、`rich`（`--locale ja` または `zh` で言語を指定）、`ansi`、`plain`、`json`、`stylish`、`markdown`、`html`、`agent` |
| `--max-warnings`      | 警告数が指定した上限を超えると失敗する                                                                                              |
| `-q, --quiet`         | 概要だけを表示する                                                                                                                  |
| `--help-level`        | `full`、`short`、`none`                                                                                                             |
| `--preset`            | `happy-path`、`opinionated`、`essential`、`incremental`、`nuxt`                                                                     |
| `--cross-file`        | ファイル間のチェックを有効にする（明示的な有効化が必要）                                                                            |
| `--cross-file-tree`   | ファイル間 lint の実行時に provide/inject ツリーを表示する                                                                          |
| `--strict-reactivity` | ネイティブチェッカーを使ってリアクティビティの喪失を検出する                                                                        |
| `--profile`           | 処理時間のプロファイルを表示する                                                                                                    |
| `--slow-threshold`    | プロファイルで遅いファイルとして報告する時間のしきい値                                                                              |

プリセットを選ぶと、段階的にルールを導入できます。

| プリセット    | 選ぶ場面                                                           |
| ------------- | ------------------------------------------------------------------ |
| `essential`   | CI で正確性を重視した診断を行いたい                                |
| `happy-path`  | 推奨するデフォルトのルールセットを使いたい                         |
| `opinionated` | より厳しい規約、スクリプトのルール、型情報を使う候補も有効にしたい |
| `incremental` | 明示的に設定したルールだけを使いたい                               |
| `nuxt`        | Nuxt コンポーネントの前提に合う、規約を重視したルールを使いたい    |

```bash
vize lint --preset essential --max-warnings 0 src
vize lint --preset opinionated --help-level short src
vize lint --cross-file --cross-file-tree src
vize lint --strict-reactivity src
vize lint --format ansi src
vize lint --format plain src
vize lint --format agent src
vize lint --format markdown src
```

<a id="チェック"></a>
<span id="check"></span>

## 型チェック

```bash
vize check
vize check src
vize check --tsconfig tsconfig.app.json
vize check --profile src
```

`vize check` は、`vize_canon` と [`corsa-bind`](https://github.com/ubugeeei/corsa-bind) が提供する Corsa のプロジェクトセッションを使います。Vize は Vue SFC の仮想 TypeScript を生成し、ネイティブの経路でプロジェクトを診断して、結果を元のソース位置へ対応付けます。

パスを指定しない場合、利用可能な `tsconfig.json` の `files` / `include` / `exclude` を使います。明示的な入力にはファイル・ディレクトリ・glob を指定でき、`.vue`、`.ts`、`.tsx`、`.d.ts` を扱えます。

| オプション          | 説明                                                         |
| ------------------- | ------------------------------------------------------------ |
| `-s, --socket`      | 起動中の `check-server` に接続する                           |
| `--tsconfig`        | 使用する `tsconfig.json` を指定する                          |
| `-f, --format`      | 出力形式: `text` または `json`                               |
| `--show-virtual-ts` | 生成した仮想 TypeScript を表示する                           |
| `-q, --quiet`       | 概要だけを表示する                                           |
| `--profile`         | `node_modules/.vize` にプロファイルを書き込む                |
| `--corsa-path`      | Corsa 実行ファイルのパスを指定する                           |
| `--servers`         | 将来用に予約した Corsa サーバー数。現在は `1` のみをサポート |
| `--declaration`     | `.d.ts` を出力する                                           |
| `--declaration-dir` | 宣言ファイルの出力先                                         |

ローカルでビルドした Corsa や `corsa-bind` を開発・検証に使う場合、`--corsa-path` で実行ファイルを固定してください。共有設定キーは `typeChecker.corsaPath` です。`typeChecker.tsgoPath` は互換性のための別名としてのみ残っています。

```bash
vize check --tsconfig tsconfig.app.json src
vize check --show-virtual-ts src/components/App.vue
vize check --profile src
vize check --declaration --declaration-dir dist/types
```

プロジェクト全体で使うテンプレートの値や Vue の型拡張は、TypeScript のプロジェクト設定から見える必要があります。`auto-imports.d.ts`、`components.d.ts`、独自の `declare module "vue"` を `tsconfig.json` の `include` に含め、必要に応じて `--tsconfig` でそのプロジェクトを選んでください。

```json
{
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue", "src/**/*.d.ts"]
}
```

```ts
// src/types/vue-app.d.ts
declare module "vue" {
  interface ComponentCustomProperties {
    $t: (key: string) => string;
  }
}
```

## Doctor

```bash
vize doctor
vize doctor src
vize doctor src packages/shared --format json
```

`vize doctor` は、Vue SFC とスクリプトモジュールを横断する、決定的なアプリケーショングラフを構築します。現在の解析対象は依存性注入、要素 ID の一意性、サーバーとクライアントの境界、リアクティビティの流れ、非同期の変更によるリスク、setup コンテキストの所有権、循環 import、コンポーネント間の props 契約です。
Vue の診断位置は、元の `.vue` のバイトオフセットに対応付けます。`<script>` と `<script setup>` を併用するコンポーネントのテンプレート内の props 使用箇所と、関連する `defineProps` 宣言も対象です。
ソース管理の ignore ファイルに従って探索し、指定したワークスペース外の入力は拒否します。コンポーネントやスクリプトを解析できない場合は処理を失敗として扱い、ファイルには書き込みません。

| オプション     | 説明                                                                           |
| -------------- | ------------------------------------------------------------------------------ |
| `--root`       | ワークスペースの境界とレポート内のパスの基準（デフォルト: 現在のディレクトリ） |
| `-f, --format` | 出力形式: `text`（デフォルト）またはバージョン付きの `json`                    |
| `--exit-zero`  | 通常は失敗になる、確認済みのエラーがあっても成功を返す                         |

終了コード `0` は検証の通過、`1` は確実または高い確信度のエラー、`2` は探索・構文解析・解析・シリアライズ・出力の失敗を示します。警告や確信度の低い指摘は健全性スコアに影響しますが、デフォルトでは失敗にしません。
フィルター、変更ファイル、スコア、自動化の詳しい契約は [Doctor の診断（英語）](../../guide/doctor.md) を参照してください。決定的な `--format json` レポートには形式とスコア計算のバージョンが明記され、CI や AI ツールから扱う場合に適しています。

```bash
vize doctor src --format json > doctor-report.json
```

<a id="検査官"></a>

## インスペクター

```bash
vize inspector src/App.vue
vize inspector "src/**/*.vue" --target ssr
vize inspector src --format json --output inspector-payload.json
vize inspector src --format agent --output inspector-agent.json
```

`vize inspector` は、1 つ以上の `.vue` ファイルを、プレイグラウンドのコンパイラーインスペクターが読み込めるデータにまとめます。ブラウザーでは Vue の出力、Vize の出力、Virtual TS、VIR、ファイル間のグラフを比較でき、パーマリンクと内容を入力済みの PR 作成リンクも生成できます。

ローカルツールや AI エージェントに同じ再現データを渡す場合は、`--format agent` を使ってください。ブラウザーを開かずに、同じデータ、プレイグラウンドの URL、概要の数値、import グラフを取得できます。データ、グラフ、行単位の差分情報はローカル用の `vize_curator` が生成し、CLI とプレイグラウンドの比較内容を揃えます。

| オプション          | 説明                                         |
| ------------------- | -------------------------------------------- |
| `-f, --format`      | 出力形式: `url`、`json`、`agent`             |
| `--target`          | コンパイル対象: `dom` または `ssr`           |
| `--playground-url`  | 生成するリンクのプレイグラウンドのベース URL |
| `--max-files`       | まとめて扱うファイル数の上限                 |
| `--custom-renderer` | カスタムレンダラーの比較を有効にする         |
| `--template-syntax` | `standard`、`strict`、`quirks` を選ぶ        |
| `-o, --output`      | URL または JSON データをファイルに書き込む   |

コントリビューター向けの作業手順は [Compiler Inspector](./compiler-inspector.md) を参照してください。

## クリーン

```bash
vize clean
vize clean --dry-run
vize clean --scope node-modules
vize clean --scope project
vize clean --force
vize clean path/to/project
```

`vize clean` は、選んだプロジェクトの既知の Vize 生成ファイルを削除し、空になった `.vize` と `node_modules/.vize` も削除します。対象はプロファイル、Musea のレポート・スナップショット・トークン、Patina セッション、設定スキーマ、LSP ログ、残ったソケット、OXC ダンプ、Oxlint の回避策ファイル、生成した Corsa のプロジェクトファイルです。
`.vize` 内の未知のファイルはデフォルトで残します。`--force` は対象の生成ファイルルートを丸ごと削除する場合にだけ使ってください。`--dry-run` は削除予定のパスを表示し、`--scope node-modules` または `--scope project` は削除対象を片方のルートに限定します。

<a id="準備ができて"></a>

## 準備状況の確認

```bash
vize ready src
vize ready --output dist src
```

`vize ready` は `fmt --write`、`lint`、`check`、`build` を順番に実行し、最初に失敗した段階で停止します。
対話的な端末の stderr には段階の番号と完了までの時間を表示し、CI やリダイレクトした stderr には通常のログを出力します。進捗表示は端末の色と Unicode の設定に従います。

| オプション     | 説明                                |
| -------------- | ----------------------------------- |
| `-o, --output` | ビルドの出力先                      |
| `--ssr`        | ビルドで SSR コンパイルを有効にする |
| `--script-ext` | `preserve` または `downcompile`     |

## アップグレード

```bash
vize upgrade
vize upgrade --dry-run
```

デフォルトでは、`vize upgrade` は Vite+ を使って npm パッケージを更新します。

```bash
vp install -D vize@latest
```

`--source cargo` は、ローカルで明示的に Cargo を使ってインストールした場合にだけ指定してください。

<a id="美術館"></a>

## Musea

```bash
vize musea --help
vize musea serve --port 6006
vize musea new
```

`musea` サブコマンドは、現在は雛形作成と実験的なエントリーポイントを中心に提供しています。日常のギャラリー開発には `@vizejs/vite-plugin-musea` を使ってください。
npm パッケージの `vize musea` は、プロジェクトにインストールした Musea プラグインを使って Vite を起動します。`--build` を付けると本番用にビルドします。

```bash
vp exec vize musea
vp exec vize musea --build
```

## LSP と IDE

```bash
vize lsp
vize lsp --port 9527
vize ide vscode
vize ide zed
```

`vize lsp` は言語サーバーを直接起動します。`vize ide` は VS Code と Zed の連携をインストール・管理するコマンドを提供します。

## グローバル オプション

```bash
vize --help
vize --version
vize <command> --help
```
