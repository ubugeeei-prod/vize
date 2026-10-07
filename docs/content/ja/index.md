---
layout: entry
title: Vize
description: Rust の高性能 Vue.js ツールチェーン。 Vue コンポーネントをコンパイル、lint、フォーマット、型チェック、探索します。
hero:
  name: Vize
  text: Rust の高性能 Vue.js ツールチェーン
  tagline: /viːz/（ヴィーズ）— コードを透視する賢明なツール。 Vue コンポーネントのコンパイル、lint、フォーマット、型チェック、探索はすべて Rust によって行われます。 ⚠️ まだ製品化の準備ができていません。
  image:
    src: /logo.svg
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
  - title: Vite+ から始める
    details: 既存アプリに Vue コンパイラとネイティブ検査タスクを追加します。設定は vite.config.ts にまとめます。
    link: ja/getting-started.md
  - title: 既存ツールから移行する
    details: 変更点と対応オプションを確認し、具体的な差分をコピーして移行します。
    link: ja/guide/migration.md
  - title: 検査を設定する
    details: コンパイラ、lint、フォーマット、型チェックの設定場所を確認します。
    link: ja/guide/configuration.md
  - title: lint ルールを調べる
    details: 診断を探し、Vue の悪い例と良い例を比較します。
    link: ja/rules/all.md
  - title: コンポーネントを探す
    details: UI の使用例と import を確認し、実際の動作を試します。
    link: ja/guide/ui/index.md
  - title: 自分のコンポーネントをプレビューする
    details: Musea の art ファイルを作り、アプリと一緒にギャラリーを開きます。
    link: ja/guide/musea.md
---

## アプリに Vize を追加する

[Getting Started](./getting-started.md) で導入し、[移行ガイド](./guide/migration.md)で
Vue コンパイラを置き換えます。検査結果を比較してから CI を変更してください。
Vite+ の統合設定は `vite.config.ts`、単独 CLI や LSP の設定は `vize.config.ts` に置きます。

Vize は開発中です。[対応状況](./stability.md)で制限を確認し、導入前に自分のプロジェクトで
診断とビルド出力を比較してください。

## 著者

![ウブゲエイ](https://github.com/ubugeeei.png)

**[ubugeeei](https://github.com/ubugeeei)** は東京を拠点とするソフトウェア エンジニアで、Vue、Rust、デザイン、言語ツールを担当しています。

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
