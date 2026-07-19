---
title: ドキュメント ブログ
description: Vize ドキュメントでは、リリース ノートと不定期ノートの両方をホストできるようになりました。
---

<!-- Generated translation; source: blog/releases/2026-03-26-docs-blog-support.md -->

# ドキュメント ブログ

<div class="blog-post-meta">
  <span class="blog-meta-chip">
    <span>
      <span class="blog-meta-label">公開</span>
      <span class="blog-meta-value">2026-03-26</span>
    </span>
  </span>
  <a class="blog-author-card" href="https://github.com/ubugeeei">
    <img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
    <span class="blog-author-text">
      <span class="blog-meta-label">著者</span>
      <span class="blog-meta-value">ubugeeei</span>
    </span>
  </a>
</div>

Vize ドキュメントでは、`docs/content/blog/` 内で 2 種類の投稿を直接ホストできるようになりました。

- `releases/` (出荷時の変更およびリリース通知)
- `notes/` 開発ブログ、アーキテクチャの記事、プロジェクトの更新などの不定期な書き込み用

## 何が変わったのか

- トップレベルの**ブログ**セクションをドキュメントに追加しました。
- 作成フローを**リリース ノート**と**ノート**に分割します。
- スターター テンプレートを追加したため、今後の投稿を簡単に作成し、一貫性を保つことができます。

## なぜこれが重要なのか

Vize はすでにパッケージの README 以上のものに成長しています。一部の更新はリファレンス ドキュメントに含まれますが、その他の更新には、何がリリースされたのか、なぜそれが重要なのか、まだ実験段階にあるのか、プロジェクトがどこに向かっているのかなど、物語のコンテキストを記載する場所が必要です。

この新しいブログ構造は、別のサイトや 2 番目の公開ワークフローを導入することなく、そのスペースを作成します。

## どこに書くか

- リリース投稿: `docs/content/blog/releases/`
- 不定期投稿：`docs/content/blog/notes/`
- テンプレート: `docs/templates/blog-release.md` および `docs/templates/blog-note.md`
