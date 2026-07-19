---
title: ユーザーのワークフロー
---

<!-- Generated translation; source: guide/workflows.md -->

# ユーザーワークフロー

このガイドでは、一般的な Vize ワークフローのコンパクトなパスを提供します: インストール、構成の接続、
CI で同じゲートをフォーマット、lint、型チェック、コンパイル、実行します。

## インストール

Vue の依存関係を所有するプロジェクトに npm パッケージをインストールします。

```bash
vp install -D vize
```

モノリポジトリの場合、パッケージが 1 つのロックファイルを共有する場合は、ワークスペースのルートにインストールします。にインストールします
パッケージに独自のロックファイルと依存関係グラフがある場合のみ。

## パッケージスクリプトを追加する

ローカル実行と CI 実行で同じエントリ ポイントを共有できるように、1 回限りのコマンドよりも名前付きスクリプトを優先します。

```json
{
  "scripts": {
    "vize:fmt": "vize fmt --check src",
    "vize:fmt:fix": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path --max-warnings 0 src",
    "vize:check": "vize check src",
    "vize:build": "vize build src",
    "vize:ready": "vize ready src"
  }
}
```

`vize ready` は広範なローカル ゲートです。大規模なリポジトリでは、個々のコマンドも保持します。
開発者は、フォーマット、lint、型チェック、コンパイラのエラーを分離できます。

## 一度構成する

デフォルトでは不十分な場合は、プロジェクト ルートに `vize.config.ts` を作成します。

```ts
import { defineConfig } from "vize";

export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  linter: {
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    tsconfig: "tsconfig.json",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

フラットなモノリポジトリエントリ、PKL、JSON、コンパイラオプション、Vue の型解決の
詳細については、[設定](./configuration.md)を参照してください。

## フォーマット

CI ではチェック モードを使用し、ローカルでは書き込みモードを使用します。

```bash
vp run vize:fmt
vp run vize:fmt:fix
```

1 回限りの移行作業の場合、`vize fmt --write` はファイル、ディレクトリ、またはグロブをターゲットにできます。

## 糸くず

正確さと低ノイズの Vue 診断を行うには、`happy-path` から始めます。

```bash
vize lint --preset happy-path --max-warnings 0 src
```

CI 出力をコンパクトに保つ必要がある場合は `--help-level short` を使用し、別のツールを使用する場合は `--format json` を使用します。
診断を消費します。完全なルールについては、[CLI](./cli.md) および [ルール](../rules/index.md) を参照してください。
表面。

## タイプチェック

プロジェクト ルートから `vize check` を実行すると、アクティブな `tsconfig`、Vue バージョン、フレームワーク パッケージ、
アンビエント型は同じ依存関係グラフから来ています。

```bash
vize check src
```

パッケージ固有のモノリポジトリ チェックの場合は、パッケージ ディレクトリから実行するか、`typeChecker.tsconfig` を設定します。
スコープ設定された構成エントリ内。

## コンパイル

Vite プラグイン パス外のコンパイラ出力が必要な場合は、`vize build` を使用します。

```bash
vize build src --output dist/vize
```

Vite アプリケーションの場合は、`@vizejs/vite-plugin` を優先し、Vite 独自のビルド オーケストレーションを使用します。参照
[Vite プラグイン](./vite-plugin.md)。

## CI

CI で同じパッケージ スクリプトを使用します。

```yaml
- run: vp install --frozen-lockfile
- run: vp run vize:fmt
- run: vp run vize:lint
- run: vp run vize:check
```

プロジェクトが Vize コンパイラー出力を直接使用する場合にのみ、`vize:build` をゲート内に保持します。のために
Vite アプリケーションでは、通常のアプリケーションのビルドでプラグインが実行されます。

## デバッグの失敗

障害が不明瞭な場合:

- `--format json` を再実行して、安定した診断フィールドを検査します。
- 遅いフェーズを見つけるには、`check`、`lint`、または `build` で `--profile` を使用します。
- コンパイラの不一致に対して、`vize inspector` を使用してインスペクタ ペイロードを作成します。
- 修正をリクエストする場合は、最小の `.vue` ファイルまたはプロジェクト スライスを含めます。

[テストとフィードバック](./testing.md) および [トラブルシューティング](./troubleshooting.md) ページの内容は次のとおりです。
レポート、現実世界の設備、および一般的な環境問題。
