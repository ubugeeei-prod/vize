---
title: CLI
---

<!-- Reviewed translation; source: guide/cli.md; scope: complete command reference; native configuration guidance -->

# CLI リファレンス

CLI を使うと、Vue ファイルのコンパイル・整形・lint・型チェックをターミナルや CI で実行できます。
アプリでは `vize` npm パッケージをインストールし、まず [package.json のスクリプト](#アプリケーションのパッケージのスクリプト) を設定してください。
LSP、IDE 管理、`check-server`、プロファイリングには [Rust バイナリ](#rust-バイナリのインストール) を使います。

npm パッケージは共有設定のヘルパーと、NAPI を利用する `build`、`fmt`、`lint`、`check`、`clean`、`ready`、`upgrade` を提供します。
[エントリーポイントの比較](#npm-パッケージのスクリプトと-rust-cli-の比較) で用途に合うものを選び、具体的な作業順序は [ユーザーワークフロー](./workflows.md) を参照してください。

<span id="アプリケーションのパッケージのスクリプト"></span>

## package.json にスクリプトを登録する

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
    "typecheck": "tsc --runExternalCode --noEmit -p tsconfig.json",
    "typecheck:tsrs": "tsrs --runExternalCode --noEmit -p tsconfig.json",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:lint
tsc --runExternalCode --noEmit -p tsconfig.json
tsrs --runExternalCode --noEmit -p tsconfig.json
vp run vize:ready
```

型チェックは content mapper を使います。tsrs は同じ `tsconfig.json` の drop-in です。`vize check` は同梱チェッカーのままです。単発のデバッグには `vp exec vize ...` を使い、文書化するワークフローや CI には名前付きスクリプトを使ってください。

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

## npm パッケージのスクリプトと Rust CLI の比較

| 用途                                                                               | 推奨エントリーポイント                           |
| ---------------------------------------------------------------------------------- | ------------------------------------------------ |
| ビルド・整形・lint・型チェック・準備・アップグレードを実行するパッケージスクリプト | npm パッケージの `vp run vize:*`                 |
| `.vue`、`.ts`、`.tsx`、`.d.ts` を含むプロジェクト全体の型チェック                  | TypeScript 7.1 の `tsc --runExternalCode`、または tsrs。[型チェック](./type-checking.md)を参照 |
| LSP、IDE セットアップ、`check-server`、プロファイル出力                            | Rust `vize` バイナリ                             |
| Vite プラグイン、npm パッケージのコマンド、Rust CLI で共有する設定                   | [Vite の設定](./configuration.md#standalone-cli) |

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
| `explain`        | 診断コードの説明ページを表示する                                                           |
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
| `--vapor`                     | Vapor モードでコンパイルする                                           |
| `--template-syntax`           | 不正な構文の扱い: `standard`（デフォルト）、`strict`、`quirks`         |
| `--custom-renderer`           | 小文字の非 HTML タグをカスタムレンダラーの要素として扱う               |
| `--custom-elements <PATTERN>` | カスタム要素として扱うタグのパターン。複数回指定可能                   |
| `--script-ext`                | `preserve` または `downcompile`                                        |
| `--declaration`               | ビルドした SFC の `.d.ts` を出力する（別名: `--dts`）                  |
| `--declaration-dir`           | 宣言ファイルの出力先（デフォルト: ビルドの出力先）                     |
| `-j, --threads`               | スレッド数を指定する                                                   |
| `--profile`                   | 処理時間のプロファイルを表示する                                       |
| `--continue-on-error`         | エラー後もコンパイルを続け、最後に失敗を報告する                       |

`<style lang="scss">`・`sass`・`less`・`stylus` のブロックは、Vite と同じく、プロジェクトにインストールされたプリプロセッサ（`sass` または `sass-embedded`、`less`、`stylus`）でコンパイルしてからスコープを付けます。プリプロセッサのエラーは `.vue` ファイル内の行を指します。Node やパッケージが見つからない場合は、ブロックをそのまま出力して警告を表示します。

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
| `--jsx-single-quote`               | JSX 属性の引用符を切り替える                           |
| `--quote-props`                    | オブジェクトキーの引用符ポリシーを選ぶ                 |
| `--trailing-comma`                 | 末尾のカンマを `all`、`es5`、`none` から選ぶ           |
| `--arrow-parens`                   | 1 引数のアロー関数の括弧を `always`、`avoid` から選ぶ  |
| `--bracket-spacing`                | オブジェクトリテラルの波括弧内の空白を切り替える       |
| `--bracket-same-line`              | 複数行タグの `>` を最後の属性行に置く                  |
| `--end-of-line`                    | 改行を `lf`、`crlf`、`cr`、自動検出の `auto` から選ぶ  |
| `--print-width`                    | 1 行の最大幅                                           |
| `--tab-width`                      | インデント幅                                           |
| `--use-tabs`                       | タブとスペースを切り替える                             |
| `--no-semi`                        | セミコロンを省略する                                   |
| `--sort-attributes`                | テンプレートの属性を並べ替える                         |
| `--single-attribute-per-line`      | 属性を 1 行に 1 つずつ配置する                         |
| `--max-attributes-per-line`        | 指定した属性数で改行する                               |
| `--normalize-directive-shorthands` | `v-bind:` / `v-on:` / `v-slot:` の省略記法を正規化する |
| `--preset`                         | `vize`、`prettier`、`compact`、`documentation` のフォーマッタ既定値を適用する |
| `--profile`                        | 処理時間のプロファイルを表示する                       |

<a id="糸くず"></a>

## lint

```bash
vize lint src
vize lint --preset opinionated src
vize lint --help-level short src
```

| オプション                           | 説明                                                                                                                                |
| ------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------- |
| `--fix`                              | テキスト編集を提供するルールの安全な自動修正を適用し、残った診断を報告する                                                          |
| `-f, --format`                       | 出力形式: `text`、`rich`（`--locale ja` または `zh` で言語を指定）、`ansi`、`plain`、`json`、`stylish`、`markdown`、`html`、`agent` |
| `--max-warnings`                     | 警告数が指定した上限を超えると失敗する                                                                                              |
| `--report-unused-disable-directives` | 何も抑制していないテンプレートの `eslint-disable` コメントを `vue/comment-directive` のエラーとして報告する                         |
| `-q, --quiet`                        | 概要だけを表示する                                                                                                                  |
| `--help-level`                       | `full`、`short`、`none`                                                                                                             |
| `--locale`                           | メッセージの言語: `en`、`ja`、`zh`                                                                                                  |
| `--preset`                           | `happy-path`、`ecosystem`、`opinionated`、`essential`、`incremental`、`nuxt`                                                        |
| `--cross-file`                       | ファイル間のチェックを有効にする（明示的な有効化が必要）                                                                            |
| `--cross-file-tree`                  | ファイル間 lint の実行時に provide/inject ツリーを表示する                                                                          |
| `--cross-file-complexity`            | ファイル間 lint の実行時に複雑度スコアとホットスポットを表示する                                                                    |
| `--type-aware`                       | lint 設定で有効にした型情報を使うルールを実行する                                                                                   |
| `--strict-reactivity`                | ネイティブチェッカーを使ってリアクティビティの喪失を検出する                                                                        |
| `--profile`                          | 処理時間のプロファイルを表示する                                                                                                    |
| `--slow-threshold`                   | プロファイルで遅いファイルとして報告する時間のしきい値                                                                              |

プリセットを選ぶと、段階的にルールを導入できます。

| プリセット    | 選ぶ場面                                                           |
| ------------- | ------------------------------------------------------------------ |
| `essential`   | CI で正確性を重視した診断を行いたい                                |
| `happy-path`  | 推奨するデフォルトのルールセットを使いたい                         |
| `ecosystem`   | `happy-path` に Router・Pinia・Nuxt・i18n の[エコシステムルール](../rules/ecosystem.md)を加えたい |
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

推奨の型チェックは、TypeScript 7.1 以降と Vize の content mapper です。
[tsrs](https://github.com/oxc-project/tsrs) でも同じプロジェクトを実行できます。コマンドは
[型チェック](./type-checking.md)を参照してください。

`vize check` は同梱のチェッカーです。[TypeScript のネイティブコンパイラー](https://github.com/microsoft/typescript-go)（TypeScript 7 として出荷される Go 実装、Project Corsa）で Vue SFC を型チェックします。Vize は SFC ごとに仮想 TypeScript を生成し、[`corsa-bind`](https://github.com/ubugeeei/corsa-bind) を通してネイティブにプロジェクトを診断し、結果を元の `.vue` の位置へ対応付けます。フラグや設定キーの「Corsa」は、このネイティブ TypeScript コンパイラーを指します。

```bash
vize check
vize check src
vize check --tsconfig tsconfig.app.json
vize check --profile src
```

パスを指定しない場合、利用可能な `tsconfig.json` の `files` / `include` / `exclude` を使います。明示的な入力にはファイル・ディレクトリ・glob を指定でき、`.vue`、`.ts`、`.tsx`、`.d.ts` を扱えます。

| オプション          | 説明                                                         |
| ------------------- | ------------------------------------------------------------ |
| `-s, --socket`      | 起動中の `check-server` に接続する                           |
| `--tsconfig`        | 使用する `tsconfig.json` を指定する                          |
| `-f, --format`      | 出力形式: `text` または `json`                               |
| `--show-virtual-ts` | 生成した仮想 TypeScript を表示する                           |
| `--save-virtual-ts-for` | 指定ファイルの仮想 TypeScript をその隣に保存する。複数回指定可能 |
| `--max-warnings`    | 警告数が上限を超えると失敗する                               |
| `--no-check-props`  | props の型チェックを省く                                     |
| `--no-check-emits`  | emit の型チェックを省く                                      |
| `--no-check-template-bindings` | テンプレートのバインディングのチェックを省く      |
| `-q, --quiet`       | 概要だけを表示する                                           |
| `--profile`         | `node_modules/.vize` にプロファイルを書き込む                |
| `--corsa-path`      | ネイティブ TypeScript（Corsa）実行ファイルのパスを指定する   |
| `--tsrs-path`       | 実験的な tsrs 実行ファイルを使用する                         |
| `--servers`         | 並列に起動する Corsa CLI サーバー数（`1`〜`32`）。省略時は自動調整 |
| `--checkers`        | Corsa プロセスごとのチェッカー数（デフォルト `1`）           |
| `--declaration`     | `.d.ts` を出力する                                           |
| `--declaration-dir` | 宣言ファイルの出力先                                         |

ローカルでビルドした Corsa や `corsa-bind` を開発・検証に使う場合、`--corsa-path` で実行ファイルを固定してください。共有設定キーは `typeChecker.corsaPath` です。`typeChecker.tsgoPath` は互換性のための別名としてのみ残っています。`--tsrs-path` と `typeChecker.tsrsPath` は、この同梱チェッカーの実行時に tsrs を選びます。tsrs 自身で型チェックするときは、[型チェック](./type-checking.md)の content mapper コマンドを使ってください。

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

`vize doctor` は Vue SFC とスクリプトモジュールを横断するアプリケーショングラフを作り、単一ファイルの lint では見えないアプリ全体の問題を報告します。

- 依存性注入（`provide` / `inject`）とコンポーネント間の props 契約
- 要素 ID の重複と、サーバーとクライアントの境界
- リアクティビティの流れ、非同期の変更によるリスク、setup コンテキストの所有権
- 循環 import

指摘は元の `.vue` ソースの位置を指します。ignore ファイルに従ってワークスペース内だけを探索し、解析できないファイルがあればエラーで停止します。ファイルには書き込みません。

| オプション     | 説明                                                                           |
| -------------- | ------------------------------------------------------------------------------ |
| `--root`       | ワークスペースの境界とレポート内のパスの基準（デフォルト: 現在のディレクトリ） |
| `-f, --format` | 出力形式: `text`（デフォルト）、バージョン付きの `json`、コードホスト向けの `sarif` |
| `--exit-zero`  | 通常は失敗になる、確認済みのエラーがあっても成功を返す                         |
| `--tui`        | 対話的なターミナル画面でレポートを確認する                                     |
| `--public-sfc` | 公開コンポーネント向けの SFC の規約も検査する（デフォルトは無効）              |

終了コード `0` は検証の通過、`1` は確実または高い確信度のエラー、`2` は探索・構文解析・解析・シリアライズ・出力の失敗を示します。警告や確信度の低い指摘は健全性スコアに影響しますが、デフォルトでは失敗にしません。
フィルター、変更ファイル、スコア、自動化に関する詳しい仕様は [Doctor による診断](./doctor.md) を参照してください。`--format json` のレポートは同じ入力から同じ結果が得られ、形式とスコア計算のバージョンも明記されるため、CI や AI ツールで扱う場合に適しています。

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

`vize inspector` は、1 つ以上の `.vue` ファイルを、プレイグラウンドのコンパイラーインスペクターが読み込めるデータにまとめます。ブラウザーでは Vue の出力、Vize の出力、Virtual TS、VIR、ファイル間のグラフを比較でき、パーマリンクや、再現情報を入力済みの PR 作成リンクも生成できます。

ローカルツールや AI エージェントに同じ再現データを渡す場合は、`--format agent` を使ってください。ブラウザーを開かずに、同じデータ、プレイグラウンドの URL、概要の数値、import グラフを取得できます。

| オプション          | 説明                                         |
| ------------------- | -------------------------------------------- |
| `-f, --format`      | 出力形式: `url`、`json`、`agent`、`compare`  |
| `--target`          | コンパイル対象: `dom`、`ssr`、`vapor`        |
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

`vize clean` は、Vize が `.vize` と `node_modules/.vize` に書き込むキャッシュとレポートを削除します。対象はプロファイル、Musea のレポートとスナップショット、lint のセッション、設定スキーマ、LSP ログ、`vize check` が生成する TypeScript プロジェクトです。

| オプション                          | 説明                                                   |
| ----------------------------------- | ------------------------------------------------------ |
| `--dry-run`                         | 削除予定のパスを表示する                               |
| `--scope node-modules` / `project` | `node_modules/.vize` または `.vize` の片方だけを削除する |
| `--force`                           | 未知のファイルも含めてフォルダーごと削除する           |
| `-q, --quiet`                       | 状態の出力を抑える                                     |

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
| `--tsconfig`   | 型チェックで使う `tsconfig.json`    |
| `--no-check-props`、`--no-check-emits`、`--no-check-template-bindings` | 指定した型チェックを省く |

## アップグレード

```bash
vize upgrade
vize upgrade --dry-run
```

デフォルトでは、`vize upgrade` は Vite+ を使い、プロジェクトのパッケージマネージャーで npm パッケージを更新します。

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

プロジェクトを対象にするコマンド（`build`、`fmt`、`lint`、`check`、`ready`）は、特定の Vite 設定を読む
`-c, --config <path>` と、プロジェクトの設定を読み込まない `--no-config` も受け付けます。
