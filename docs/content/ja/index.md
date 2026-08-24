---
layout: entry
title: Vize
description: Rust の高性能 Vue.js ツールチェーン。 Vue コンポーネントをコンパイル、lint、フォーマット、型チェック、探索します。
hero:
  name: Vize
  text: Rust の高性能 Vue.js ツールチェーン
  tagline: /viːz/（ヴィーズ）— コードを透視する賢明なツール。 Vue コンポーネントのコンパイル、lint、フォーマット、型チェック、探索はすべて Rust によって行われます。 ⚠️ まだ製品化の準備ができていません。
  image:
    src: logo.svg
    alt: Vize のロゴ
  actions:
    - theme: brand
      text: 始めましょう
      link: ja/getting-started.md
    - theme: alt
      text: GitHub
      link: https://github.com/ubugeeei-prod/vize
    - theme: alt
      text: 遊び場
      link: https://vizejs.dev/play
features:
  - title: Viteプラグイン
    details: Vue アプリケーションに推奨される統合から始めます。共有 Vize 構成を使用した Vite 内のネイティブ SFC コンパイルです。
    link: ja/guide/vite-plugin.md
  - title: 静的解析パイプライン
    details: パーサー、セマンティック分析、lint ルール、仮想 TypeScript、クロスファイル チェック、およびエディター診断は、同じ Rust ネイティブ分析レイヤーを共有します。
    link: ja/guide/static-analysis.md
  - title: ルールのドキュメント
    details: 具体的な Vue、HTML、SSR、Vapor、Musea、タイプ認識、ファイル間診断を悪い例と良い例とともに参照します。
    link: ja/rules/index.md
  - title: 共有構成
    details: コンパイラ オプション、Vite スキャン、lint プリセット、型チェック、フォーマット、LSP 機能、および Musea を `vize.config.*` から構成します。
    link: ja/guide/configuration.md
  - title: ネイティブ型チェック
    details: "`vize:check` パッケージ スクリプトは、`vize_canon` および `corsa-bind` によってサポートされる Corsa プロジェクト セッションを通じて実行され、Vue 対応の診断をネイティブ パスに維持します。"
    link: ja/guide/static-analysis.md
  - title: パッケージスクリプトとCLIリファレンス
    details: LSP、プロファイリング、バイナリの直接使用について文書化された Rust CLI とともに、アプリのワークフローのプロジェクト スクリプトから npm パッケージを使用します。
    link: ja/guide/cli.md
  - title: コンパイラインスペクタ
    details: Vue 出力、Vize 出力、仮想 TS、VIR、ファイル間グラフを検査し、パーマリンクされた再現またはエージェント レポートを共有します。
    link: ja/guide/compiler-inspector.md
  - title: Oxlint プラグイン
    details: Oxlint 内で Vize の Vue 診断を実行し、それらを 1 つのパスで OXC の JS および TS ルールと結合します。
    link: ja/guide/oxlint.md
  - title: 実験的なバンドラー統合
    details: rollup、webpack、esbuild、および専用の Rspack パスが存在しますが、依然として Vite が推奨され、最も安定した統合です。
    link: ja/guide/unplugin.md
  - title: 8.3倍高速
    details: 15,000 個の SFC ファイル (36.9 MB) を 500 ミリ秒以内にマルチスレッドでコンパイルします。アリーナ割り当て、レーヨン並列処理、ゼロ GC。
    link: ja/architecture/performance.md
  - title: コンポーネントギャラリー
    details: Musea — @vizejs/vite-plugin-musea によって提供されるギャラリー ワークフローを使用したアート ファイル、ドキュメント、パレット生成、a11y、および VRT ツール。
    link: ja/guide/musea.md
  - title: WASM バインディング
    details: WebAssembly を使用してブラウザで Vue コンパイラを直接実行します。遊び場、ドキュメント、教育ツールを強化します。
    link: ja/guide/wasm.md
  - title: AIの統合
    details: AI アシスタントが Musea を通じて Vue コンポーネントを理解し、操作できるようにする MCP サーバー。
    link: ja/integrations/mcp.md
  - title: ベーパーモード
    details: Vue 3.6 Vapor モードのファーストクラスのサポート — 仮想 DOM を使用しないきめ細かいリアクティブ コンパイル。
    link: ja/architecture/overview.md
  - title: 哲学
    details: アートからインスピレーションを得たアーキテクチャ、酸化エコシステム (OXC、oxlint、corsa-bind)、および統一されたツールチェーン ビジョン。
    link: ja/philosophy.md
  - title: ブログ
    details: 出荷された変更に関するリリース ノートに加え、設計の更新、開発ブログ、プロジェクトの考え方に関する不定期のノート。
    link: ja/blog/index.md
---

<!-- Generated translation; source: index.md -->

## 現在の方向

Vize における最近の最大の変化の 1 つは、ネイティブ型チェックです。によって使用される `vize check` コマンド
npm パッケージ スクリプトとエディター向けの型チェック パイプラインは `vize_canon` プラスに移行します
[`corsa-bind`](https://github.com/ubugeeei/corsa-bind)、Vize が Vue 仮想ファイルを保持できるようにします。
TypeScript プロジェクトの診断をネイティブ パスで行う時間が長くなります。

それは素の速度以上に重要です。これにより、Vize はテンプレート分析、診断、ナビゲーション、将来のエディター機能の間で緊密なループを実現すると同時に、JavaScript がホストするコンパイラー プロセスを通じて跳ね返される必要がある作業量を削減します。忠実度の話はまだ追いついていませんが、これがツールチェーンが明らかに向かっている方向です。

同じ方向がリンティングと Musea にも当てはまります。静的解析はパーサーとクロッキーから始まります
セマンティック モデル、Patina lint ルール、Canon 仮想 TypeScript、コンパイラー決定、エディターをフィードします。
診断、およびコンポーネント ギャラリーのメタデータ。実際のワークフローは以下に文書化されています。
[静的分析](./guide/static-analysis.md)、構成の詳細は次のとおりです。
[構成](./guide/configuration.md)。具体的なルールと診断カタログは次のとおりです。
[ルール](./rules/index.md)。

## 著者

![ウブゲエイ](https://github.com/ubugeeei.png)

- \*[ubugeeei](https://github.com/ubugeeei)\*\*は東京を拠点とするソフトウェア エンジニアで、Vue、Rust、デザイン、言語ツールを担当しています。

彼は [Vue.js コア チーム](https://vuejs.org/about/team.html)、[Vue.js 日本ユーザー グループ](https://github.com/vuejs-jp) コア スタッフ、[Vite+](https://github.com/voidzero-dev/vite-plus) コア コントリビューター、[mates-dev](https://github.com/mates-dev) のチーフ エンジニアの一員です。

[chibivue](https://github.com/chibivue-land/chibivue)、[Vize](https://github.com/ubugeeei-prod/vize)、[Ox Content](https://github.com/ubugeeei/ox-content)の作者でもあります。

- GitHub: [github.com/ubugeeei](https://github.com/ubugeeei)
- X (Twitter): [@ubugeeei](https://x.com/ubugeeei)
- ブログ: [wtrclred.io](https://wtrclred.io)
- chibivue.land: [chibivue.land](https://chibivue.land)

## スポンサー

Vize は、MIT のもとでライセンス供与された無料のオープンソース プロジェクトです。完全なツールチェーン (コンパイラー、リンター、フォーマッタ、型チェッカー、LSP、コンポーネント ギャラリー、WASM バインディング) の開発と保守は、継続的な集中力と献身が必要な重要な作業です。

Vize によって時間を節約し、開発エクスペリエンスを向上できる場合、または高性能 Vue.js ツールチェーンのビジョンを信じている場合は、プロジェクトのスポンサーになることを検討してください。

- CI/CD ランナー インフラストラクチャは [Blacksmith](https://www.blacksmith.sh/) によってスポンサーされています。
- [GitHub スポンサー](https://github.com/sponsors/ubugeeei)

あなたのサポートは、継続的な開発とインフラストラクチャのコストに資金を提供し、Vize が誰にとっても無料であり続けることを保証します。規模に関係なく、あらゆる貢献が大きな変化をもたらします。
