---
layout: entry
title: Vize
description: Rust 製の高速な Vue.js ツールチェーン。Vue コンポーネントのコンパイル、lint、フォーマット、型チェック、プレビューをまとめて扱えます。
hero:
  name: Vize
  text: Rust 製の高速な Vue.js ツールチェーン
  tagline: /viːz/（ヴィーズ）— コードを見通す、賢いツール。Rust を基盤に、Vue コンポーネントのコンパイル、lint、フォーマット、型チェック、プレビューをまとめて扱えます。⚠️ まだ本番利用できる段階ではありません。
  image:
    src: /logo.svg
    alt: Vize のロゴ
  actions:
    - theme: brand
      text: はじめる
      link: ja/getting-started.md
    - theme: alt
      text: GitHub
      link: https://github.com/ubugeeei-prod/vize
    - theme: alt
      text: プレイグラウンド
      link: https://vizejs.dev/play
features:
  - title: Vite+ から始める
    details: 既存のアプリに Vue コンパイラと検査タスクを追加します。ツールの設定は vite.config.ts で共有し、TypeScript の設定には tsconfig.json を使います。
    link: ja/getting-started.md
  - title: 既存ツールから移行する
    details: 何が変わるかを確認し、対応するオプションを移して、変更前後の差分をそのままコピーできます。
    link: ja/guide/migration.md
  - title: 検査を設定する
    details: コンパイラ、lint、フォーマット、型チェックの設定をどこに書くかを確認します。
    link: ja/guide/configuration.md
  - title: lint ルールを調べる
    details: 診断の意味を調べ、Vue の悪い例と良い例を見比べます。
    link: ja/rules/all.md
  - title: コンポーネントを探す
    details: UI の使用例を見て import をコピーし、実際の動作を試します。
    link: ja/guide/ui/index.md
  - title: 自分のコンポーネントをプレビューする
    details: Musea の art ファイルを書き、アプリと並べてギャラリーを開きます。
    link: ja/guide/musea.md
---

## アプリに Vize を追加する

まず[はじめる](./getting-started.md)の手順で導入し、[移行ガイド](./guide/migration.md)に沿って
Vue コンパイラを置き換えます。CI を切り替える前に、検査結果を見比べてください。
Vite+、CLI、言語サーバーで使う共通設定は `vite.config.ts` に、TypeScript の設定は `tsconfig.json` にまとめます。既存の `vize.config.ts` も引き続き使えます。設定の検索順序と優先順位は[設定ガイド](./guide/configuration.md)を参照してください。

Vize は開発中です。既知の制限は[サポート状況](./stability.md)にまとめています。導入する前に、
自分のプロジェクトで診断とビルド出力を見比べてください。

## 作者

![ubugeeei](https://github.com/ubugeeei.png)

**[ubugeeei](https://github.com/ubugeeei)** は東京を拠点とするソフトウェアエンジニアです。Vue、Rust、デザイン、言語ツールに取り組んでいます。

[Vue.js コアチーム](https://vuejs.org/about/team.html)のメンバーで、[Vue.js 日本ユーザーグループ](https://github.com/vuejs-jp)のコアスタッフ、[Vite+](https://github.com/voidzero-dev/vite-plus) のコアコントリビューター、[mates-dev](https://github.com/mates-dev) のチーフエンジニアを務めています。

[chibivue](https://github.com/chibivue-land/chibivue)、[Vize](https://github.com/ubugeeei-prod/vize)、[Ox Content](https://github.com/ubugeeei/ox-content) の作者でもあります。

- GitHub: [github.com/ubugeeei](https://github.com/ubugeeei)
- X (Twitter): [@ubugeeei](https://x.com/ubugeeei)
- ブログ: [wtrclred.io](https://wtrclred.io)
- chibivue.land: [chibivue.land](https://chibivue.land)

## スポンサー

Vize は MIT ライセンスで公開している、無料のオープンソースプロジェクトです。コンパイラ、リンター、フォーマッタ、型チェッカー、LSP、コンポーネントギャラリー、WASM バインディングまでを含むツールチェーン全体を開発し、保守し続けるには、多くの時間と継続的な取り組みが必要です。

Vize が開発の時間を節約したり、開発体験を良くしたりしているなら、あるいは高速な Vue.js ツールチェーンという目標に共感していただけるなら、スポンサーとしての支援をご検討ください。

- CI/CD のランナー環境は [Blacksmith](https://www.blacksmith.sh/) の提供を受けています。
- [GitHub Sponsors](https://github.com/sponsors/ubugeeei)

いただいた支援は、継続的な開発とインフラの費用に充てられ、Vize を誰もが無料で使える状態に保つ助けになります。金額にかかわらず、すべての支援が大きな力になります。
