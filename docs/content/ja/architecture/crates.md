---
title: 木箱
---

<!-- Generated translation; source: architecture/crates.md -->

# クレートリファレンス

> **⚠️ 進行中の作業:**Vize は現在開発中です。正規版を参照してください
> [Rust クレートのサポート層](../stability.md#rust-crate-support-tiers) パブリックに依存する前に
> API。

Vize の Rust ワークスペースは、20 個のプライマリ クレートを中心に構成されています。各クレートは 1 つの再利用可能なレーンを所有しているため、
解析、セマンティック分析、コード生成、lint、フォーマット、型チェック、
エディター ツールは同じ構文モデルを共有できます。

## 財団

| 木箱              | 役割                                                                                                          |
| ----------------- | ------------------------------------------------------------------------------------------------------------- |
| `vize_carton`     | 共有アロケーター、文字列、ハッシュ コレクション、フラグ、プロファイラー、i18n、および DOM/タグ ユーティリティ |
| `vize_relief`     | 共有 Vue テンプレート AST、コンパイラ エラー、およびコンパイラ オプション                                     |
| `vize_armature`   | Vue テンプレートのトークナイザーとパーサー                                                                    |
| `vize_croquis`    | セマンティック分析、スコープ追跡、バインディングメタデータ、反応性、仮想 TS ヘルパー                          |
| `vize_croquis_cf` | オプトインのクロスファイルセマンティック分析とプロジェクト全体の診断                                          |

## コンパイル

| 木箱                 | 役割                                                                 |
| -------------------- | -------------------------------------------------------------------- |
| `vize_atelier_core`  | 共有変換レーンとコード生成インフラストラクチャ                       |
| `vize_atelier_dom`   | VDOM 指向のテンプレートのコンパイル                                  |
| `vize_atelier_vapor` | Vapor モード テンプレートのコンパイル                                |
| `vize_atelier_ssr`   | サーバー側のレンダリング テンプレートのコンパイル                    |
| `vize_atelier_sfc`   | `.vue` 解析とスクリプト、テンプレート、スタイル オーケストレーション |
| `vize_atelier_jsx`   | 共有 JSX/TSX の解析、降格、およびコンパイラの統合                    |

## 開発者ツール

| 木箱           | 役割                                                                             |
| -------------- | -------------------------------------------------------------------------------- |
| `vize_patina`  | Vue SFC リンターと診断フォーマット                                               |
| `vize_glyph`   | Vue SFC フォーマッタ                                                             |
| `vize_canon`   | Vue 対応型チェックと仮想 TypeScript 生成                                         |
| `vize_maestro` | 言語サーバー プロトコルの実装                                                    |
| `vize_musea`   | 美術館のアート解析、ドキュメント、パレット生成、autogen、および VRT コア         |
| `vize_curator` | ローカル インスペクター ペイロード、グラフ/差分メタデータ、プロファイル レポート |
| `vize_fresco`  | TUI 指向の実験で使用されるターミナル UI プリミティブ                             |

## ディストリビューション層

| 木箱           | 役割                                                        |
| -------------- | ----------------------------------------------------------- |
| `vize_vitrine` | JS コンシューマー向けの共有 NAPI および WASM バインディング |
| `vize`         | Rust ネイティブ CLI とドキュメントのクレート再エクスポート  |

## 注意事項

- `vize_musea` は、Musea アート ツール用の Rust コアです。ギャラリー UI と開発サーバーのワークフローは次のとおりです。
  `@vizejs/vite-plugin-musea` によって提供されました。
- `vize_curator` は公開されていません。インスペクターペイロードなどのローカル開発アーティファクトを所有しています。
  エージェント レポート、ファイル間のグラフ メタデータ、および CLI プロファイル レポートのレンダリング。低レベル
  共有クレートは独自のホット パスを計測するため、プロファイラーは `vize_carton` に残ります。
- `vize_vitrine` は Rust から JS へのブリッジです。 `@vizejs/native` などのパッケージ
  `@vizejs/wasm` はバインディングを公開します。
- `vize` は、ワークスペース内の完全な Rust CLI クレートです。 v1 アルファの場合、そのパブリック バイナリ チャネルは次のとおりです。
  GitHub Releases または Nix ですが、npm `vize` パッケージがサポートされているパッケージ スクリプト エントリ ポイントです。

## パッケージのマッピング

| パッケージ/コマンド         | メイン Rust クレート                                                                     |
| --------------------------- | ---------------------------------------------------------------------------------------- |
| `vize build`                | `vize`、`vize_atelier_sfc`、`vize_atelier_dom`、`vize_atelier_vapor`、`vize_atelier_ssr` |
| `vize fmt`                  | `vize`、`vize_glyph`                                                                     |
| `vize lint`                 | `vize`、`vize_patina`                                                                    |
| `vize check`                | `vize`、`vize_canon`                                                                     |
| `vize inspector`            | `vize`、`vize_curator`                                                                   |
| `vize lsp`                  | `vize`、`vize_maestro`                                                                   |
| `@vizejs/vite-plugin`       | `vize_vitrine`、`vize_atelier_sfc`                                                       |
| `@vizejs/native`            | `vize_vitrine`                                                                           |
| `@vizejs/wasm`              | `vize_vitrine`                                                                           |
| `@vizejs/vite-plugin-musea` | `vize_musea`、`vize_vitrine`                                                             |
| `@vizejs/musea-mcp-server`  | `vize_musea`、`vize_vitrine`                                                             |
| `oxlint-plugin-vize`        | `vize_patina`、`vize_vitrine`                                                            |
