---
title: クレート
---

<!-- Reviewed translation; source: architecture/crates.md; scope: complete document -->

# クレートリファレンス

コンパイラや開発ツールの機能を担当する Rust クレートを探すためのリファレンスです。
たとえば、SFC のコンパイルは `vize_atelier_sfc`、フォーマッタの変更は `vize_glyph` が入口になります。
[パッケージとの対応表](#パッケージのマッピング) で、CLI コマンドや JavaScript パッケージから担当クレートを調べられます。

表は `crates/` にある既存製品のクレートを扱います。実験段階のレベルクレートは `davinci/` にあります。
公開 Rust API に依存する前に [Rust クレートのサポート区分](../stability.md) を確認してください。
実装を変更する場合は [ソースガイド](./source-guide.md) に進んでください。

## 基盤

| クレート          | 役割                                                                                                      |
| ----------------- | --------------------------------------------------------------------------------------------------------- |
| `vize_carton`     | 共通のアロケーター、文字列、ハッシュコレクション、フラグ、プロファイラー、i18n、DOM・タグのユーティリティ |
| `vize_relief`     | 共通の Vue テンプレート AST、コンパイラのエラー、コンパイラのオプション                                   |
| `vize_armature`   | Vue テンプレートの字句解析器とパーサー                                                                    |
| `vize_croquis`    | 意味解析、スコープの追跡、バインディングのメタデータ、リアクティビティ、仮想 TS の補助処理                |
| `vize_croquis_cf` | 明示的に有効化するファイル間の意味解析と、プロジェクト全体の診断                                          |

## コンパイル

| クレート             | 役割                                                             |
| -------------------- | ---------------------------------------------------------------- |
| `vize_atelier_core`  | 共通の変換処理とコード生成の基盤                                 |
| `vize_atelier_dom`   | VDOM 向けのテンプレートのコンパイル                              |
| `vize_atelier_vapor` | Vapor モードのテンプレートのコンパイル                           |
| `vize_atelier_ssr`   | サーバーレンダリング向けのテンプレートのコンパイル               |
| `vize_atelier_sfc`   | `.vue` の解析と、スクリプト・テンプレート・スタイルの処理の連携  |
| `vize_atelier_jsx`   | 共通の JSX・TSX の解析、下位表現への変換、コンパイラへの組み込み |

## 開発者ツール

| クレート       | 役割                                                                             |
| -------------- | -------------------------------------------------------------------------------- |
| `vize_patina`  | Vue SFC のリンターと診断の整形                                                   |
| `vize_glyph`   | Vue SFC のフォーマッタ                                                           |
| `vize_canon`   | Vue に対応した型チェックと仮想 TypeScript の生成                                 |
| `vize_maestro` | Language Server Protocol の実装                                                  |
| `vize_musea`   | Musea art ファイルの解析、ドキュメント・パレットの生成、自動生成、VRT の中核処理 |
| `vize_curator` | ローカルのインスペクター用データ、グラフ・差分のメタデータ、プロファイルレポート |
| `vize_fresco`  | TUI の実験で使うターミナル UI の基本部品                                         |

## 配布・連携

| クレート       | 役割                                                               |
| -------------- | ------------------------------------------------------------------ |
| `vize_vitrine` | JavaScript から利用する共通の NAPI・WASM バインディング            |
| `vize`         | Rust ネイティブ CLI と、ドキュメント向けのクレートの再エクスポート |

## 注意事項

- `vize_musea` は Musea art ツールの Rust 側の中核です。ギャラリー UI と開発サーバーの処理は `@vizejs/vite-plugin-musea` が提供します。
- `vize_curator` は公開パッケージではありません。インスペクター用データ、エージェント向けレポート、ファイル間グラフのメタデータ、CLI のプロファイルレポートの描画など、ローカル開発用の成果物を担当します。共通クレート内の頻繁に実行される処理を計測するため、低レベルのプロファイラーは `vize_carton` に置きます。
- `vize_vitrine` は Rust と JavaScript の橋渡しをします。`@vizejs/native` や `@vizejs/wasm` は、そのバインディングを公開するパッケージです。
- `vize` は、ワークスペース内の完全な Rust CLI クレートです。v1 alpha のバイナリは GitHub Releases または Nix で提供し、パッケージスクリプトから呼び出す入口には npm の `vize` パッケージを使います。

## パッケージのマッピング

| パッケージ・コマンド        | 主な Rust クレート                                                                       |
| --------------------------- | ---------------------------------------------------------------------------------------- |
| `vize build`                | `vize`, `vize_atelier_sfc`, `vize_atelier_dom`, `vize_atelier_vapor`, `vize_atelier_ssr` |
| `vize fmt`                  | `vize`, `vize_glyph`                                                                     |
| `vize lint`                 | `vize`, `vize_patina`                                                                    |
| `vize check`                | `vize`, `vize_canon`                                                                     |
| `vize inspector`            | `vize`, `vize_curator`                                                                   |
| `vize lsp`                  | `vize`, `vize_maestro`                                                                   |
| `@vizejs/vite-plugin`       | `vize_vitrine`, `vize_atelier_sfc`                                                       |
| `@vizejs/native`            | `vize_vitrine`                                                                           |
| `@vizejs/wasm`              | `vize_vitrine`                                                                           |
| `@vizejs/vite-plugin-musea` | `vize_musea`, `vize_vitrine`                                                             |
| `@vizejs/musea-mcp-server`  | `vize_musea`, `vize_vitrine`                                                             |
| `oxlint-plugin-vize`        | `vize_patina`, `vize_vitrine`                                                            |
