---
title: ブログ
description: Vize プロジェクトのリリースノートと、不定期に公開する設計や開発のノート。
---

<!-- Reviewed translation; source: blog/index.md -->

# ブログ

Vize のドキュメントでは、二つの種類の記事を公開しています。

<div class="blog-grid">
  <a class="blog-card" href="./releases/">
    <span class="blog-card-kicker">記事の種類</span>
    <strong>リリースノート</strong>
    <p>公開した変更、リリースの見どころ、移行時の注意点、導入の案内。</p>
  </a>
  <a class="blog-card" href="./notes/">
    <span class="blog-card-kicker">記事の種類</span>
    <strong>ノート</strong>
    <p>開発の記録、設計の解説、アーキテクチャの考察、プロジェクトの舞台裏などを伝える不定期の記事。</p>
  </a>
</div>

## 公開方法

- リリースの記事は `docs/content/blog/releases/` に置きます。
- 不定期のノートは `docs/content/blog/notes/` に置きます。
- ファイル名には `YYYY-MM-DD-slug.md` を使い、日付順に並べられるようにします。
- `docs/templates/blog-release.md` または `docs/templates/blog-note.md` を出発点にします。
- 新しい記事は、ドキュメントの対応するセクションに表示されます。

## 開始点

- [リリースノート](./releases/)
- [ノート](./notes/)

## 最新の投稿

<div class="blog-post-list">
  <a class="blog-post-list-item" href="./notes/2026-06-07-real-world-testing/">
    <strong>実際のプロジェクトでの検証</strong>
    <span>実際のプロジェクトをテストの中心に据え、v1.0.0 へのロードマップを示した、Vize の実プロジェクト検証段階への移行。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-comparing-vize-with-official-vue-oxc-golar-verter-flint-and-tsslint/">
    <strong>ツールの比較</strong>
    <span>公式 Vue ツール、Oxc、Golar、Verter、Flint、TSSLint と Vize を、実用面から比較します。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-performance-tuning-notes-for-a-vue-toolchain/">
    <strong>性能を高めるための工夫</strong>
    <span>解析、メモリ割り当て、並列処理、フィードバックの速さが重要な Vue ツールチェーンで、性能改善から得た知見。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-testing-agentic-coding-and-trust/">
    <strong>テストとエージェント</strong>
    <span>エージェントが開発に加わるほど、スナップショット、実プロジェクトのフィクスチャ、決定的な検証が重要になる理由。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-vapor-mode-and-the-next-vue-compiler-surface/">
    <strong>Vapor Mode</strong>
    <span>Vapor Mode が Vize にとって重要な理由と、DOM を直接扱うきめ細かなコンパイル方式が、実行時の性能以外にもたらす変化。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-vue-as-a-language-and-the-strongest-frontend-environment/">
    <strong>UI の言語としての Vue</strong>
    <span>Vue を UI の言語と捉え、フロントエンド開発に、ばらばらのツールではなく一貫した環境が必要な理由を考えます。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-why-musea-and-design-systems-matter-in-the-ai-era/">
    <strong>Musea と AI</strong>
    <span>AI が UI を素早く生成する時代に、意図、制約、アクセシビリティ、レビューの仕組みを保つための Musea とデザインシステムの役割。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-real-world-feedback-and-the-road-to-production-ready/">
    <strong>本番運用への道</strong>
    <span>実験的なプロジェクトを本番で使えるツールチェーンに育てるために、実プロジェクトの徹底した検証とコミュニティの声が必要な理由。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-personal-tooling-and-development-speed/">
    <strong>個人開発とスピード</strong>
    <span>Vize が独立した個人のプロジェクトであることが、探求、速さ、意欲的なツールチェーンの設計にもたらす利点。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-the-advantages-and-beauty-of-toolchains-and-vertical-integration/">
    <strong>ツールチェーンの垂直統合</strong>
    <span>スタックを広く自分で扱うことが、開発ツールの速さ、一貫性、美しさを高める理由。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-why-ai-needs-deterministic-fast-static-analysis/">
    <strong>AI のための静的解析</strong>
    <span>AI が書くコードが増えるほど、速く信頼できる静的なフィードバックが必要になります。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-where-vize-fits-in-the-vue-tooling-landscape/">
    <strong>Vue ツールの中での Vize の位置</strong>
    <span>Vue のツール環境における Vize の位置付けと、周辺のプロジェクトとの違いを示す地図。</span>
  </a>
  <a class="blog-post-list-item" href="./releases/2026-03-26-oxlint-plugin-vize-alpha/">
    <strong><code>oxlint-plugin-vize</code> アルファ版</strong>
    <span>Oxlint の JS プラグインによって、Vue SFC に対する Vize Patina の診断を、一度の Oxlint 実行で扱えるようになります。</span>
  </a>
  <a class="blog-post-list-item" href="./releases/2026-03-26-docs-blog-support/">
    <strong>ドキュメント内のブログ</strong>
    <span>Vize のドキュメントに、リリースノートと不定期のノートを掲載できるようになりました。</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-why-vize-needs-notes/">
    <strong>ノートを書く理由</strong>
    <span>変更履歴だけでは伝えきれない、プロジェクトの背景や判断を説明するための場所。</span>
  </a>
</div>
