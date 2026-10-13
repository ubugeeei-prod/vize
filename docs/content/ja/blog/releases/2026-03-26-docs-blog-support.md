---
title: ドキュメントにブログを追加
description: Vize のドキュメントに、リリースノートと不定期のノートを掲載できるようになりました。
---

<!-- Reviewed translation; source: blog/releases/2026-03-26-docs-blog-support.md; scope: complete document -->

# ドキュメントにブログを追加

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

Vize のドキュメントの `docs/content/blog/` に、2 種類の記事を置けるようになりました。

- `releases/`: 公開済みの変更と、リリースに関するお知らせ
- `notes/`: 開発ログ、アーキテクチャの解説、プロジェクトの近況など、不定期の記事

## 変更点

- ドキュメントのトップレベルに **ブログ** のセクションを追加しました。
- 記事の種類を **リリースノート** と **ノート** に分けました。
- 新しい記事を書きやすく、体裁もそろえやすいように、ひな形を追加しました。

## なぜ必要か

Vize は、パッケージの README だけでは説明しきれない規模になりました。リファレンスに書くべき更新もありますが、背景まで含めて伝えたい更新もあります。何を公開したのか、なぜ重要なのか、どこがまだ実験的なのか、プロジェクトがどこへ向かっているのか、といった内容です。

今回のブログは、別のサイトや公開の仕組みを増やさずに、そのための場所を用意するものです。

## 記事を書く場所

- リリースの記事: `docs/content/blog/releases/`
- 不定期の記事: `docs/content/blog/notes/`
- ひな形: `docs/templates/blog-release.md` と `docs/templates/blog-note.md`
