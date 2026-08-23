---
title: CLI
---

<!-- Generated translation; source: guide/cli.md -->

# CLI リファレンス

> **⚠️ 進行中の作業:**Vize は活発に開発中であり、CLI サーフェスはまだ進化中です。

ほとんどのアプリケーション ワークフローでは、`vize` npm パッケージをインストールし、`package.json` を通じて実行する必要があります。
スクリプト。このページでは、LSP、IDE管理、
`check-server`、プロファイリング、およびその他の直接 CLI ワークフロー。 npm パッケージは共有構成を公開します
ヘルパーと、NAPI サポートの `build`、`fmt`、`lint`、`check`、`clean`、`ready`、および `upgrade` コマンド。

分析パイプラインのより高度な説明については、[静的分析](./static-analysis.md)を参照してください。

## アプリケーション パッケージ スクリプト

アプリの場合は、npm からインストールし、安定したコマンドをプロジェクト スクリプトに接続します。

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

1 回限りのローカル デバッグには `vp exec vize ...` を使用しますが、文書化するには名前付きスクリプトを優先します。
ワークフローとCI。

## Rust バイナリのインストール

v1 アルファの場合は、事前に構築された GitHub リリース バイナリまたは Nix エントリ ポイントを使用します。 Rust CLI は
crates.io インストール チャネルはまだサポートされていません。

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

プラットフォーム固有のバイナリを次からダウンロードすることもできます。
[GitHub リリース](https://github.com/ubugeeei-prod/vize/releases)。

このリポジトリ内のローカル開発の場合は、ワークスペース ビルドをインストールします。

```bash
cargo install --path crates/vize --force --locked
```

## npm パッケージ スクリプトと Rust CLI の比較

| 必要                                                                                    | 推奨エントリーポイント           |
| --------------------------------------------------------------------------------------- | -------------------------------- |
| ビルド、フォーマット、lint、チェック、準備完了、アップグレード用のパッケージ スクリプト | npm パッケージの `vp run vize:*` |
| `.vue`、`.ts`、`.tsx`、および `.d.ts` にわたるプロジェクトに基づく型チェック            | 錆び                             |
| LSP、IDE セットアップ、`check-server`、プロファイリング アーティファクト                | Rust `vize` バイナリ             |
| 共有 Vite プラグイン、npm package コマンド、および Rust CLI 設定                        | `vize.config.*`                  |

## コマンド

```bash
vize [COMMAND]
```

コマンドなしで呼び出された場合、`vize` はデフォルトで `build` になります。

| コマンド       | 説明                                                              |
| -------------- | ----------------------------------------------------------------- |
| `build`        | Vue SFC ファイルをコンパイルする                                  |
| `fmt`          | Vue SFC ファイルをフォーマットする                                |
| `lint`         | Lint Vue SFC ファイル                                             |
| `check`        | Vue SFC、TS、TSX、および `.d.ts` 入力のタイプ チェック            |
| `inspector`    | プレイグラウンド コンパイラー インスペクター ペイロードを作成する |
| `clean`        | Vize で生成されたキャッシュ アーティファクトを削除する            |
| `ready`        | `fmt`、`lint`、`check`、および `build` を実行します。             |
| `upgrade`      | インストールされている CLI を更新する                             |
| `check-server` | Unix JSON-RPC タイプチェック サーバーを起動します。               |
| `musea`        | Musea のサブコマンドとスキャフォールディング                      |
| `lsp`          | 言語サーバーを起動します                                          |
| `ide`          | エディタ統合をインストールまたは管理する                          |

すべての `--profile` ターミナル レポートは、ローカル専用の `vize_curator` クレートによってレンダリングされます。の
インストルメンテーション フックは `vize_carton` に残りますが、管理者は CLI レポート シェイプと並行して所有します。
検査官とエージェント向けのアーティファクト。

## 建てる

```bash
vize build src/**/*.vue
vize build --ssr
vize build --profile src
```

主なオプション:

| オプション            | 説明                                                             |
| --------------------- | ---------------------------------------------------------------- |
| `-o, --output`        | 共通入力ルートの下のソース相対出力。衝突を拒否します             |
| `-f, --format`        | 出力形式: `js`、`json`、`stats`                                  |
| `--ssr`               | SSR コンパイルを有効にする                                       |
| `--custom-renderer`   | 小文字の非 HTML タグをカスタム レンダラー要素として扱う          |
| `--custom-elements`   | カスタム要素としてコンパイルするタグパターン。複数回指定可能     |
| `--script-ext`        | `preserve` または `downcompile`                                  |
| `--declaration`       | ビルドされた SFC の `.d.ts` ファイルを出力 (エイリアス: `--dts`) |
| `--declaration-dir`   | 宣言出力ディレクトリ (デフォルト: ビルド出力ディレクトリ)        |
| `-j, --threads`       | スレッド数のオーバーライド                                       |
| `--profile`           | 印刷タイミング プロファイル                                      |
| `--continue-on-error` | コンパイルを続けて最後に失敗を報告する                           |

## フォーマット

```bash
vize fmt --check src
vize fmt --write src
```

主なオプション:

| オプション                         | 説明                                                 |
| ---------------------------------- | ---------------------------------------------------- |
| `--check`                          | 変更されるレポート ファイル                          |
| `-w, --write`                      | フォーマットされた出力を書き込む                     |
| `--single-quote`                   | 文字列引用スタイルを切り替える                       |
| `--print-width`                    | 最大線幅                                             |
| `--tab-width`                      | インデント幅                                         |
| `--use-tabs`                       | タブとスペースを切り替える                           |
| `--no-semi`                        | セミコロンを省略します                               |
| `--sort-attributes`                | テンプレート属性の並べ替え                           |
| `--single-attribute-per-line`      | 1 行に 1 つの属性を入力します                        |
| `--max-attributes-per-line`        | 指定された属性数の後に折り返す                       |
| `--normalize-directive-shorthands` | `v-bind:` / `v-on:` / `v-slot:` 省略表記を正規化する |
| `--profile`                        | 印刷タイミング プロファイル                          |

## 糸くず

```bash
vize lint src
vize lint --preset opinionated src
vize lint --help-level short src
```

主なオプション:

| オプション            | 説明                                                                                     |
| --------------------- | ---------------------------------------------------------------------------------------- |
| `--fix`               | テキスト編集を提供するルールから安全な自動修正を適用し、残りの診断を報告します。         |
| `-f, --format`        | 出力形式: `text`、`ansi`、`plain`、`json`、`stylish`、`markdown`、`html`、または `agent` |
| `--max-warnings`      | 警告が制限を超えると失敗します。                                                         |
| `-q, --quiet`         | 概要のみを表示                                                                           |
| `--help-level`        | `full`、`short`、または `none`                                                           |
| `--preset`            | `happy-path`、`opinionated`、`essential`、`incremental`、または `nuxt`                   |
| `--cross-file`        | オプトインのファイル間チェックを有効にする                                               |
| `--cross-file-tree`   | ファイル間リンティングが有効な場合に提供/注入ツリーを出力します。                        |
| `--strict-reactivity` | ネイティブ チェッカーによる反応性損失リンティングを有効にする                            |
| `--profile`           | 印刷タイミング プロファイル                                                              |
| `--slow-threshold`    | プロファイル出力の低速ファイルしきい値                                                   |

プリセットは段階的な導入を目的としています。

| プリセット    |                                                                       | の場合に使用します。 |
| ------------- | --------------------------------------------------------------------- | -------------------- |
| `essential`   | CI で正確性を重視した診断が必要な場合                                 |
| `happy-path`  | デフォルトの推奨バンドルが必要な場合                                  |
| `opinionated` | より強力な規則、スクリプト ルール、および型を認識する候補が必要です。 |
| `incremental` | 明示的に構成されたルールのみが必要な場合                              |
| `nuxt`        | Nuxt コンポーネントの前提条件を備えた独自のルールが必要です           |

例:

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

## チェック

```bash
vize check
vize check src
vize check --tsconfig tsconfig.app.json
vize check --profile src
```

`vize check` は、`vize_canon` と [`corsa-bind`](https://github.com/ubugeeei/corsa-bind) を通じて公開された Corsa プロジェクト セッションによってサポートされています。 Vize は、Vue SFC 用の仮想 TypeScript を生成し、ネイティブ パスでプロジェクト診断を実行し、結果を元のソースの場所にマッピングします。

明示的なパスが指定されていない場合、`vize check` は `tsconfig.json` `files` / `include` / を使用します。
利用可能な場合は、`exclude`。明示的な入力はファイル、ディレクトリ、またはグロブであり、`.vue` を含めることができます。
`.ts`、`.tsx`、および `.d.ts`。

主なオプション:

| オプション          | 説明                                                            |
| ------------------- | --------------------------------------------------------------- | ------------------------ |
| `-s, --socket`      | 実行中の `check-server`                                         | に接続します。           |
| `--tsconfig`        | `tsconfig.json`                                                 | をオーバーライドします。 |
| `-f, --format`      | 出力形式: `text` または `json`                                  |
| `--show-virtual-ts` | 生成された仮想 TypeScript を印刷する                            |
| `-q, --quiet`       | 概要のみを表示                                                  |
| `--profile`         | プロファイル アーティファクトを `node_modules/.vize` に書き込む |
| `--corsa-path`      | Corsa 実行可能ファイルのパスをオーバーライドする                |
| `--servers`         | 予約済み Corsa サーバー数。 `1` のみがサポートされています。    |
| `--declaration`     | `.d.ts` 出力を出力する                                          |
| `--declaration-dir` | 発行された宣言の出力ディレクトリ                                |

Vize の開発中またはテスト中にカスタム Corsa 実行可能ファイルを固定したい場合は、`--corsa-path` を使用します。
ローカル `corsa-bind` チェックアウト。共有構成キーは `typeChecker.corsaPath` です。 `typeChecker.tsgoPath`
は互換性エイリアスとしてのみ保持されます。

便利なパターン:

```bash
vize check --tsconfig tsconfig.app.json src
vize check --show-virtual-ts src/components/App.vue
vize check --profile src
vize check --declaration --declaration-dir dist/types
```

プロジェクト全体のテンプレート値と Vue アンビエント タイプは、TypeScript プロジェクトを通じて表示される必要があります
構成。 `auto-imports.d.ts`、`components.d.ts` などの生成されたファイル、または独自のファイルを含めます。
`tsconfig.json` で Vue 宣言を行い、必要に応じて `--tsconfig` でそのプロジェクトを選択します。

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

## 検査官

```bash
vize inspector src/App.vue
vize inspector "src/**/*.vue" --target ssr
vize inspector src --format json --output inspector-payload.json
vize inspector src --format agent --output inspector-agent.json
```

`vize inspector` は、プレイグラウンドによって消費されるペイロードに 1 つ以上の `.vue` ファイルをパッケージ化します。
コンパイラインスペクタ。次に、ブラウザは Vue 出力、Vize 出力、仮想 TS、VIR、および
ファイル間のグラフを作成し、パーマリンクと事前入力されたプル リクエスト リンクを生成します。

別のローカル ツールまたは AI エージェントが同じ再現を必要とする場合、
ブラウザ。レポートには、正確なペイロード、プレイグラウンド URL、概要メトリクス、インポート グラフが含まれます。
ペイロード、グラフ、および行の差分メタデータは、ローカル専用の `vize_curator` クレートによって構築されるため、CLI および
遊び場の検査は調整を続けます。

主なオプション:

| オプション          | 説明                                                 |
| ------------------- | ---------------------------------------------------- |
| `-f, --format`      | 出力形式: `url`、`json`、または `agent`              |
| `--target`          | コンパイラ ターゲット: `dom` または `ssr`            |
| `--playground-url`  | 生成されたリンクの Playground ベース URL             |
| `--max-files`       | バッチ ペイロードに含まれるファイルを制限する        |
| `--custom-renderer` | カスタム レンダラーの比較を有効にする                |
| `--template-syntax` | `standard`、`strict`、または `quirks` を選択します   |
| `-o, --output`      | URL または JSON ペイロードをファイルに書き込みます。 |

コントリビューターのワークフローについては、[Compiler Inspector](./compiler-inspector.md) を参照してください。

## クリーン

```bash
vize clean
vize clean --dry-run
vize clean --scope node-modules
vize clean --scope project
vize clean --force
vize clean path/to/project
```

`vize clean` は、選択したプロジェクト ルートの既知の Vize 所有のローカル アーティファクトを削除してから、
空の `.vize` および `node_modules/.vize` の親。管理対象アーティファクト リストにはプロファイル出力が含まれます。
Musea レポート/スナップショット/トークン、Patina セッション、構成スキーマ、LSP ログ、ソケットの残り物、OXC
ダンプ、Oxlint 回避策ファイル、具体化された Corsa プロジェクト ファイル。 `.vize` の下の不明なエントリ
デフォルトで保存されます。選択したアーティファクト ルートを削除する必要がある場合にのみ、`--force` を使用します。
卸売り。 `--dry-run` は、削除されるアーティファクト パスを出力します。 `--scope node-modules`を使用してください
または、1 つのアーティファクト ルートのみをクリーンアップする必要がある場合は、`--scope project`。

## 準備ができて

```bash
vize ready src
vize ready --output dist src
```

`vize ready` は、`fmt --write`、`lint`、`check`、`build` を順番に実行します。コマンドは次の時点で停止します。
最初の失敗ステップ。

主なオプション:

| オプション     | 説明                                |
| -------------- | ----------------------------------- |
| `-o, --output` | ビルドステップの出力ディレクトリ    |
| `--ssr`        | ビルドの SSR コンパイルを有効にする |
| `--script-ext` | `preserve` または `downcompile`     |

## アップグレード

```bash
vize upgrade
vize upgrade --dry-run
```

デフォルトでは、`vize upgrade` は Vite+ を通じて npm パッケージを更新します。

```bash
vp install -D vize@latest
```

`--source cargo` は、明示的なローカル Cargo インストールの場合にのみ使用してください。

## 美術館

```bash
vize musea --help
vize musea serve --port 6006
vize musea new
```

`musea` サブコマンドは現在、スキャフォールディングと実験的なエントリ ポイントに重点を置いています。
日々のギャラリー開発において、現在推奨されるワークフローは次のとおりです。
`@vizejs/vite-plugin-musea`。

npm パッケージは、Musea で Vite を実行する便利な `vize musea` コマンドも公開します。
プロジェクトにインストールされているプラグイン:

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

`vize lsp` は言語サーバーを直接起動します。
`vize ide` は、VS Code および Zed 用のエディター固有のインストールおよび管理コマンドを追加します
統合。

## グローバル オプション

```bash
vize --help
vize --version
vize <command> --help
```
