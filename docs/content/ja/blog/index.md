---
title: ブログ
description: Vize プロジェクトのリリースノートと不定期ノート。
---

<!-- Generated translation; source: blog/index.md -->

# ブログ

Vize ドキュメントには 2 つの書き込みレーンが追加されました。

<div class="blog-grid">
  <a class="blog-card" href="./releases/">
    <span class="blog-card-kicker">トラック</span>
    <strong>Rリリースノート</strong>
    <p>出荷された変更、リリース ハイライト、移行ノート、ロールアウト ガイダンス。</p>
  </a>

<a class="blog-card" href="./notes/">
    <span class="blog-card-kicker">トラック</span>
    <strong>メモ</strong>
    <p>開発ブログ、設計記事、アーキテクチャ ノート、舞台裏の更新などの不定期投稿。</p>
  </a>
</div>

## 公開方法

- リリース投稿は `docs/content/blog/releases/` にあります。
- `docs/content/blog/notes/`に不定期投稿が生息しています。
- 投稿を並べ替え可能な状態に保つために、ファイル名に `YYYY-MM-DD-slug.md` を使用します。
- `docs/templates/blog-release.md` または `docs/templates/blog-note.md` から開始します。
- 新しい投稿は、ドキュメント ツリーの一致するセクションの下に表示されます。

## 開始点

- [リリースノート](./releases/)
- [メモ](./notes/)

## 最新の投稿

<div class="blog-post-list">
  <a class="blog-post-list-item" href="./notes/2026-06-07-real-world-testing/">
    <strong>現実世界のテスト</strong>
    <span>Vize は現実世界のテスト段階に入ります — 実際のプロジェクトは現在テストスイートであり、v1.0.0.</span> への明確なロードマップがあります。
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-comparing-vize-with-official-vue-oxc-golar-verter-flint-and-tsslint/">
    <strong>ツーリングの比較</strong>
    <span>A 公式 Vue ツール、Oxc、Golar、Verter、Flint、TSSLint にわたる Vize と近隣プロジェクトの実用的な比較。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-performance-tuning-notes-for-a-vue-toolchain/">
    <strong>パフォーマンスチューニング</strong>
    <span>解析、割り当て、並列処理、フィードバック ループがすべて重要となる Vue ツールチェーンの構築から得られる実践的なパフォーマンスのレッスン。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-testing-agentic-coding-and-trust/">
    <strong>テストとエージェント</strong>
    <span>エージェントが開発ループの一部である場合、スナップショットを多用するテスト、現実世界のフィクスチャ、決定論的チェックがより重要になる理由。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-vapor-mode-and-the-next-vue-compiler-surface/">
    <strong>蒸気モード</strong>
    <span>Vize にとって Vapor モードが重要な理由、および直接の詳細なコンパイラー パスが実行時のパフォーマンスよりも大きく変化する理由。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-vue-as-a-language-and-the-strongest-frontend-environment/">
    <strong>言語としてのVue</strong>
    <span>このノートは、Vue は UI 用の言語であるという考えに基づいて、フロントエンド開発に分散したツールではなく一貫した環境が必要な理由を説明します。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-why-musea-and-design-systems-matter-in-the-ai-era/">
    <strong>美術館 & AI</strong>
    <span>AI は UI を迅速に生成できますが、Musea とデザイン システムにより、意図、制約、アクセシビリティ、レビュー ワークフローが永続的になります。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-real-world-feedback-and-the-road-to-production-ready/">
    <strong>本番準備完了</strong>
    <span>実験的なプロジェクトから運用準備が整ったツールチェーンへの道には、なぜ徹底的な現実世界の検証とコミュニティからのフィードバックが必要なのか。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-personal-tooling-and-development-speed/">
    <strong>パーソナルスピード</strong>
    <span>なぜ Vize が独立していて個人的なものであることが、探索、スピード、野心的なツールチェーン設計にとって利点となるのか。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-the-advantages-and-beauty-of-toolchains-and-vertical-integration/">
    <strong>垂直ツールチェーン</strong>
    <span>より多くのスタックを所有すると、開発者ツールの速度、一貫性、さらには美的品質が向上する理由。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-why-ai-needs-deterministic-fast-static-analysis/">
    <strong>AI</strong> の静的解析
    <span>A AI はより多くのコードを記述します。より高速で信頼性の高い静的フィードバックが必要です。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-where-vize-fits-in-the-vue-tooling-landscape/">
    <strong>Vue ツール マップ</strong>
<span>A 現在の Vue ツール環境における Vize の位置と、隣接するプロジェクトとの違いを示すマップ。</span>
  </a>
  <a class="blog-post-list-item" href="./releases/2026-03-26-oxlint-plugin-vize-alpha/">
    <strong><code>oxlint-plugin-vize</code> アルファ</strong>
    <span>A 新しい Oxlint JS プラグイン ブリッジにより、Vize Patina 診断が Vue SFC の単一の Oxlint 実行に組み込まれます。</span>
  </a>
  <a class="blog-post-list-item" href="./releases/2026-03-26-docs-blog-support/">
    <strong>ドキュメント ブログ</strong>
    <span>Vize ドキュメントでは、リリース ノートと不定期ノートの両方をホストできるようになりました。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-why-vize-needs-notes/">
    <strong>ノートレーン</strong>
    <span>一部のプロジェクト更新には、単なる変更ログ エントリではなく、コンテキストを考慮する余地が必要です。</span>
  </a>
</div>
