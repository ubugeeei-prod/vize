---
title: 日常の開発ワークフロー
---

<!-- Reviewed translation; source: guide/workflows.md -->

<span id="ユーザーワークフロー"></span>

# 日常の開発ワークフロー

Vize のインストールから設定、フォーマット、lint、型チェック、コンパイルまでを順に説明します。
ローカルで使うコマンドを CI でも実行し、同じ基準でコードを検査します。

## インストール

Vue の依存関係を管理しているプロジェクトに npm パッケージをインストールします。

```bash
vp install -D vize
```

モノリポジトリでロックファイルを共有している場合は、ワークスペースのルートにインストールします。
各パッケージが独立したロックファイルと依存関係を持つ場合は、それぞれのパッケージにインストールします。

## パッケージスクリプトを追加する

package.json にスクリプトを登録すると、ローカルと CI で同じコマンドを使えます。

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

`vize ready` は複数の検査をまとめて実行します。大規模なリポジトリでは個別のコマンドも用意すると、
フォーマット、lint、型チェック、コンパイルのどこで失敗したかを切り分けられます。

<span id="一度構成する"></span>

## 共通の設定を用意する

既定値を変更する場合は、既存の `vite.config.*` のトップレベルの `vize` にネイティブ設定を追加します。検索規則、次回リリースの対応状況、必要な場合に使える専用形式は[設定ガイド](./configuration.md)を参照してください。

```ts
// vite.config.ts
export default {
  vize: {
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
  },
};
```

フラットなモノリポジトリエントリ、PKL、JSON、コンパイラオプション、Vue の型解決の
詳細については、[設定](./configuration.md)を参照してください。

## フォーマット

CI では検査モード、ローカルでは書き込みモードを使います。

```bash
vp run vize:fmt
vp run vize:fmt:fix
```

1 回限りの移行作業の場合、`vize fmt --write` はファイル、ディレクトリ、またはグロブをターゲットにできます。

<span id="糸くず"></span>

## lint

最初は、正確性を重視し、不要な警告を抑えた `happy-path` プリセットを使います。

```bash
vize lint --preset happy-path --max-warnings 0 src
```

CI の出力を短くするには `--help-level short`、診断を別のツールで処理するには `--format json` を使います。
詳しくは [CLI コマンド](./cli.md)と[ルール一覧](../rules/index.md)を参照してください。

<span id="タイプチェック"></span>

## 型チェック

`vize check` はプロジェクトのルートから実行します。これにより、`tsconfig`、Vue、フレームワーク、
グローバルな型宣言を同じプロジェクトの依存関係から解決します。

```bash
vize check src
```

モノリポジトリ内の特定のパッケージを検査する場合は、そのディレクトリから実行するか、
対象の設定エントリに `typeChecker.tsconfig` を指定します。

## コンパイル

Vite プラグインを使わずにコンパイル結果を取得する場合は、`vize build` を使います。

```bash
vize build src --output dist/vize
```

Vite アプリでは [Vite プラグイン](./vite-plugin.md)を導入し、Vite のビルド処理を使います。

## CI

CI で同じパッケージ スクリプトを使用します。

```yaml
- run: vp install --frozen-lockfile
- run: vp run vize:fmt
- run: vp run vize:lint
- run: vp run vize:check
```

Vize のコンパイル結果を直接使うプロジェクトでは、CI でも `vize:build` を実行します。
Vite アプリでは、通常のアプリビルドでプラグインを検証できます。

<span id="デバッグの失敗"></span>

## 問題の切り分け

問題の原因が分からない場合は、次の情報を確認します。

- `--format json` で再実行し、診断の各フィールドを確認します。
- 遅いフェーズを見つけるには、`check`、`lint`、または `build` で `--profile` を使用します。
- コンパイル結果に問題がある場合は、`vize inspector` で調査用のペイロードを作成します。
- 不具合を報告する場合は、再現に必要な最小の `.vue` ファイルやプロジェクトの一部を添えます。

[テストとフィードバック](./testing.md)には、不具合報告と実プロジェクトを使った検証をまとめています。
よくある環境の問題は[トラブルシューティング](./troubleshooting.md)を参照してください。
