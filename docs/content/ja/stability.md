---
title: 安定性
description: Vize のサポート区分、互換性の方針、実験的な機能を確認する。
---

<!-- Reviewed translation; source: stability.md -->

# 安定性

Vize は v1 アルファ版を目指して開発中です。早期に導入できる機能と、変更が続く内部実装・実験的な統合を区別します。
ツールチェーン全体が本番利用に十分な状態になった、という意味ではありません。
導入判断には[本番利用の準備状況](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/production-readiness.md)を、
非推奨期間やバージョンごとのサポートには[サポート方針](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/support-policy.md)を参照してください。

<span id="バージョニング契約"></span>

## バージョンと互換性

安定版 v1 より前のリリースには破壊的変更が含まれる可能性があります。公開 API、CLI のフラグ、設定項目、診断コード、
生成結果に影響する変更は、リリースノートで説明します。アルファ版の互換性の方針は次のとおりです。

| 対象 | 方針 |
| --- | --- |
| 公開パッケージ名 | 利用できる状態を維持するか、移行方法を案内する |
| 文書化した CLI コマンド・フラグ | 告知のない動作変更を避ける |
| 文書化した設定項目 | リリースノートで説明する変更を除き、名前と値の形式を維持する |
| 文書化した診断コード | 診断の抑制や修正レポートが引き続き使える識別子を維持する |
| Rust クレートの公開 API | 下記の区分と非推奨期間に従う |
| Rust クレートの非公開実装 | 安定版 v1 より前は、移行手順なしに変更される場合がある |
| 生成コード・仮想 TypeScript ファイル | 正確性、互換性、性能、診断のために変更される場合がある |

## ランタイムサポート

公開 npm パッケージが必要とする Node.js の最低バージョンは、原則として Node 22 です。
`oxlint-plugin-vize` は `^22 || >= 24` を指定しており、Node 22 と Node 24 以降に対応します。Node 23 は検証対象に含みません。

リリース時は、各パッケージが対応を宣言している macOS・Linux・Windows の x64 / arm64 向けにネイティブパッケージをビルドします。
CI では、最低対応バージョンとプロジェクトで使う Node.js バージョンを検証します。

新規インストールの全体検証（`.github/workflows/native-smoke.yml`）は、毎週および必要に応じて実行します。
すべての PR で実行するわけではありません。公開パッケージを GitHub のホストランナーでインストールし、
linux-x64-gnu、linux-arm64-gnu、darwin-arm64、win32-x64-msvc を検証します。
darwin-x64 と win32-arm64-msvc は、各アーキテクチャのホストランナーで検証します。対象の Node.js は 22 と 24 です。
npm への公開前には、リリース用 tarball のインストール検証も必要です。
`vize --version`、`vize check`、`@vizejs/native` の `require` / `import`、インストールした `@vizejs/vite-plugin` による `vite build` を確認します。

Linux musl の 2 つのターゲットには、新規インストールを実行するホストランナーの検証がありません。
現時点では、対象別のビルド成果物と `@vizejs/native-*` の optional dependency 解決を検証しています。
対応するネイティブ tarball を Alpine コンテナーで検証できるようになるまでの制約です。

| 対象 | ホストランナーの制約 | 代わりに行う検証 |
| --- | --- | --- |
| linux-x64-musl | GitHub に Alpine / musl のネイティブランナーがない | musl tarball をビルドし、`node:alpine` で手動検証する |
| linux-arm64-musl | arm64 ホストランナーは Ubuntu GNU で、Alpine / musl ではない | arm64 musl tarball をビルドし、Alpine arm64 で手動検証する |

これらの不足は [#493](https://github.com/ubugeeei-prod/vize/issues/493) と関連する課題で追跡します。

Rust の最低対応バージョン（MSRV）は、`Cargo.toml` の `[workspace.package].rust-version` に記載しています。
開発用の `rust-toolchain.toml` は、同じバージョンか、それより新しいバージョンを指定します。
安定版 v1 より前は MSRV を引き上げる場合があり、その際はリリースノートで案内します。
パッケージを配布する場合は、ツールチェーンの指定から推測せず、クレートの `Cargo.toml` にある `rust-version` を確認してください。

## パッケージのサポート層

| 区分 | パッケージ | 方針 |
| --- | --- | --- |
| アルファ版対応 | `vize`、`@vizejs/native`、`@vizejs/vite-plugin`、`@vizejs/plugin-sdk` | 早期の本番導入を対象とする。破壊的変更はリリースノートで案内する |
| 互換性プレビュー | `@vizejs/unplugin`、`@vizejs/rspack-plugin`、`@vizejs/nuxt`、`@vizejs/nuxt-lint-config`、`@vizejs/musea-nuxt` | 一般的な構成での動作を想定するが、フレームワークとの互換性は変化する可能性がある |
| 実験的 | `oxlint-plugin-vize`、`@vizejs/vite-plugin-musea`、`@vizejs/musea-mcp-server`、`@vizejs/wasm` | 公開済みだが、API・コマンド・生成結果・ワークフローはアルファ版の間に変わる可能性がある |
| 実験的 | `@vizejs/composable`、`@vizejs/ui`、`@vizejs/marquette` | 公開ライブラリだが、API の構成と生成結果の検証基準は変わる可能性がある |
| 実験的（ワークスペース内のみ） | `@vizejs/data`、`@vizejs/devtools`、`@vizejs/router`、`@vizejs/state` | リポジトリ内に実装があるが npm には未公開。レジストリからはインストールできない |
| 開発初期 | `@vizejs/fresco`、`@vizejs/fresco-native`、エディター拡張 | 開発と評価に使えるが、v1 アルファ版の本番対応対象には含まれない |

## Rust クレートのサポート階層

以下の表は、crates.io の利用者向けの互換性の方針です。Cargo メタデータで公開が許可されたクレートをすべて含みます。
初回公開の準備などでリリースが延期されているクレートも対象です。非公開のモジュールや実装の詳細には、同じ互換性を保証しません。

<!-- rust-crate-support:start -->

| クレート | 区分 | 対象 | 公開 API | 削除・非推奨 |
| --- | --- | --- | --- | --- |
| `vize_carton` | アルファ版対応 | Vize のコンパイラ・ライブラリ開発者 | `vize_carton::{Allocator, Box, FxHashMap}` | `#[deprecated]` を付けて 1 minor release の間保持 |
| `vize_relief` | アルファ版対応 | AST・コンパイラ統合の開発者 | `vize_relief::{RootNode, CompilerOptions}` | `#[deprecated]` を付けて 1 minor release の間保持 |
| `vize_armature` | アルファ版対応 | Vue テンプレートを解析するツール | `vize_armature::{parse, Parser, Tokenizer}` | `#[deprecated]` を付けて 1 minor release の間保持 |
| `vize_croquis` | 互換性プレビュー | 意味解析・型情報を使うツールの開発者 | `vize_croquis::{Croquis, Drawer}` | `#[deprecated]` を付けて 1 minor release の間保持 |
| `vize_croquis_cf` | 実験的 | プロジェクト全体の解析を評価する開発者 | `vize_croquis_cf::CrossFileAnalyzer` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_l0_derive` | 実験的 | Davinci のダンプ形式の開発者 | `vize_l0_derive::Dump` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_doctor` | 実験的 | アプリの状態を解析するツールの開発者 | `vize_doctor::{DoctorFinding, FindingEvidence}` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_l3` | 実験的 | Davinci のバックエンド実行計画の開発者 | `vize_l3::{dump::Page, op, verify}` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_l4` | 実験的 | ネイティブコード生成を評価する開発者 | `vize_l4::{targets::dom::emit_file, module::assemble}` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_atelier_core` | アルファ版対応 | Vue コンパイラのバックエンド開発者 | `vize_atelier_core::{transform, generate}` | `#[deprecated]` を付けて 1 minor release の間保持 |
| `vize_atelier_dom` | アルファ版対応 | VDOM コンパイラ・バンドラー統合 | `vize_atelier_dom::compile_template` | `#[deprecated]` を付けて 1 minor release の間保持 |
| `vize_atelier_vapor` | 実験的 | Vapor コンパイラ統合の評価 | `vize_atelier_vapor::compile_vapor` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_atelier_ssr` | 互換性プレビュー | SSR・フレームワーク統合の開発者 | `vize_atelier_ssr::compile_ssr` | `#[deprecated]` を付けて 1 minor release の間保持 |
| `vize_atelier_sfc` | アルファ版対応 | SFC ツール・バンドラーの開発者 | `vize_atelier_sfc::{parse_sfc, compile_sfc}` | `#[deprecated]` を付けて 1 minor release の間保持 |
| `vize_atelier_jsx` | 互換性プレビュー | JSX/TSX コンパイラ・ツールの開発者 | `vize_atelier_jsx::{compile_jsx, lower_source}` | `#[deprecated]` を付けて 1 minor release の間保持 |
| `vize_marquette` | 実験的 | フレームワーク・対象別アダプターの開発者 | `vize_marquette::{ApplicationContract, Target}` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_musea` | 実験的 | Musea ギャラリー・ドキュメントツール | `vize_musea::{parse_art, transform_to_csf}` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_fresco` | 開発初期 | TUI の評価 | `vize_fresco::{RenderTree, LayoutEngine}` | 最低期間の保証なし |
| `vize_canon` | 互換性プレビュー | 型チェッカー・エディター統合 | `vize_canon::{type_check_sfc, TypeChecker}` | `#[deprecated]` を付けて 1 minor release の間保持 |
| `vize_patina` | 互換性プレビュー | リンター・Oxlint 統合 | `vize_patina::{lint, Linter}` | `#[deprecated]` を付けて 1 minor release の間保持 |
| `vize_l1` | 実験的 | 情報を保持する Vue テンプレートツールの開発者 | `vize_l1::{parse, SurfaceTree}` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_l1_to_l2` | 実験的 | Vue の変換・コンパイラバックエンド開発者 | `vize_l1_to_l2::{lower, emit_dom}` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_l2` | 実験的 | 方言に依存しないコンパイラ IR の開発者 | `vize_l2::{dump::Page, op, verify}` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_l0` | 実験的 | Davinci の各層・パスの開発者 | `vize_l0::{id::NodeId, side_table::SideTable}` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |
| `vize_l2_to_l3` | 実験的 | L2 から L3 への変換・バックエンド開発者 | `vize_l2_to_l3::{lower, PartitionFacts}` | 最低期間の保証なし。可能な範囲で破壊的変更を告知 |

<!-- rust-crate-support:end -->

各クレートの区分は `package.metadata.vize.stability` にも記載します。CI は Cargo メタデータ、この表、
リリース対象のクレート一覧を照合し、追加・削除・区分変更によって互換性の方針が暗黙に変わることを防ぎます。

<span id="semver-ゲートの解釈"></span>

### SemVer 検証の意味

`cargo-semver-checks` は、レジストリに比較対象のリリースがあるクレートに対して実行します。
初回公開を待つクレートは、比較対象を取得できるようになった段階で検証に加わります。
それまでも、メタデータ・サポート表・リリース一覧の照合は行います。

| 区分 | CI での扱い |
| --- | --- |
| アルファ版対応・互換性プレビュー | API の破壊的変更を修正するか、サポート方針の非推奨期間を守り、破壊的変更の印を付ける |
| 実験的 | 意図しない API の変化を検出する。意図した破壊的変更は、非推奨期間なしに印を付けられる |
| 開発初期 | 同じ検出を行うが、API やクレート全体をリリースで置き換えたり削除したりする場合がある |

CI が認識する破壊的変更の印は、Conventional Commits のタイトルにある `!`、または `BREAKING CHANGE:` のフッターです。
印を付けても、アルファ版対応・互換性プレビューの非推奨期間は省略できません。

<span id="アルファ版にとって十分に安定しているものとは何か"></span>

## アルファ版対応に必要な条件

- インストールと使い方が文書化されている
- パッケージのビルド、インストール、対応 Node.js を CI で検証している
- 公開 API をリリース時に検証している
- 不具合や互換性の報告を担当する人がいる
- 未対応の動作が関連ガイドに明記されている

<span id="まだ約束されていないこと"></span>

## まだ保証していないこと

すべての Vue 構文、パッケージマネージャーの配置、エディター機能、フレームワークとの互換性を保証するわけではありません。
Vize のガイドが意図した違いを明記していない限り、公式の Vue ツールの結果を互換性の基準として扱ってください。
リリースを止めるコンパイラ・型チェック・ランタイム・Vite ビルドの検証項目は、
[Vue 互換性マトリックス](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/vue-parity-matrix.md)にまとめています。

セキュリティ上の問題は `SECURITY.md`、開発への参加と修正の流れは `CONTRIBUTING.md` を参照してください。
