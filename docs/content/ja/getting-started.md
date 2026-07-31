---
title: はじめる
---

<!-- Generated translation; source: getting-started.md -->

# はじめる

> **⚠️ 開発中:** Vize は活発に開発されており、まだ本番環境向けではありません。
> API やパッケージの境界は予告なく変更される可能性があります。

Vize (_/viːz/_) は Rust ネイティブな Vue.js ツールチェーンです。コンパイル、lint、
フォーマット、型チェック、エディター診断、コンポーネントの検証を 1 つのワークスペースに
まとめつつ、それぞれの機能を用途別のパッケージとコマンドから利用できます。

| やりたいこと                                                    | 推奨される入口                  |
| --------------------------------------------------------------- | ------------------------------- |
| Vite で Vue SFC をコンパイルする                                | `@vizejs/vite-plugin`           |
| Nuxt で Vue SFC をコンパイルする                                | `@vizejs/nuxt`                  |
| プロジェクトスクリプトから lint・フォーマット・型チェックを行う | `vize`                          |
| Vize の診断を Oxlint と組み合わせる                             | `oxlint-plugin-vize`            |
| コンポーネントを検証・閲覧する                                  | `@vizejs/vite-plugin-musea`     |
| エディター機能を試す                                            | VS Code、Zed、または `vize lsp` |

## 既存プロジェクトをセットアップする

プロジェクトのルートで対話形式の初期化コマンドを実行します。

```bash
vpx vize init
```

`vpx` は [Vite+](https://viteplus.dev/guide/install) に含まれています。シェルでこのコマンドを
利用できない場合は、先に Vite+ をインストールしてください。

`vize init` は、ファイルを書き換える前に Vite、Vite+、Nuxt、パッケージマネージャー、
TypeScript、有効な lint コマンド、既存の Vize 設定を検出します。設定する機能は個別に選べます。

- Vite プラグインまたは Nuxt モジュール
- 実際の lint コマンドが読み込む設定ファイル内の Oxlint プラグイン
- `vize fmt` と `vize check` のプロジェクトスクリプト
- 共有の `vize.config.*` 設定
- VS Code 拡張機能の推奨設定

ファイルや依存関係を変更せず、予定されている処理をすべて確認できます。

```bash
vpx vize init --dry-run
```

CI などの非対話環境では、必要な機能を明示します。

```bash
vpx vize init --yes --lint --bundler --fmt --typecheck --editor
```

検出規則、全オプション、冪等性の保証、編集を意図的に拒否する条件については、
[Project Setup（英語）](../guide/init.md)を参照してください。

## 手動で導入する

既存の設定を維持したい場合や、Vize の機能を 1 つずつ導入したい場合は手動設定が適しています。

- [Vite プラグイン](./guide/vite-plugin.md) — Vite でのネイティブ Vue SFC コンパイル
- [Nuxt 統合](./integrations/nuxt.md) — Nuxt の Vite パイプラインを通すサポート済みの方法
- [パッケージスクリプトと CLI](./guide/cli.md) — `vize build`、`fmt`、`lint`、`check`、
  `ready`、および完全な Rust CLI

Vite が推奨されるバンドラー統合です。unplugin と Rspack のパッケージはまだ実験的です。
現在の対応範囲は[その他のバンドラー](./guide/unplugin.md)を参照してください。

## 目的別ガイドへ進む

このページは入口の案内に役割を絞っています。設定と統合の詳細については、次のガイドを
信頼できる情報源として参照してください。

- [設定](./guide/configuration.md) — `vize.config.*`、コンパイラ、型チェック、Musea の設定
- [静的解析](./guide/static-analysis.md) — lint と型チェックのモデル
- [ルール一覧](./rules/index.md) — 診断と具体例
- [Oxlint プラグイン](./guide/oxlint.md) — プリセット、設定、各コマンドが実際に読み込むファイル
- [VS Code とその他のエディター](./integrations/vscode.md) — オプトインのエディタープロファイルと LSP 設定
- [JSX & TSX](./guide/jsx.md) — `.vue` SFC 以外で記述する Vue コンポーネント
- [Musea](./guide/musea.md) — コンポーネント例、ドキュメント、トークン、a11y、VRT

Vize のエディター統合が実験段階にある間、日常的な Vue 開発では公式の
[`vuejs/language-tools`](https://github.com/vuejs/language-tools)を引き続き使用してください。
