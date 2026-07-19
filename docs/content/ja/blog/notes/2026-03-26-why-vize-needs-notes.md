---
title: ノートレーン
description: 一部のプロジェクトの更新には、単なる変更ログのエントリではなく、コンテキストを考慮する余地が必要です。
---

<!-- Generated translation; source: blog/notes/2026-03-26-why-vize-needs-notes.md -->

# ノートレーン

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

リファレンス ドキュメントは、「これをどのように使用するか?」という質問に答えるのに優れています。彼らは次のような質問に答えるのがはるかに下手です。

- なぜこの機能が追加されたのですか?
- このアーキテクチャはどのようなトレードオフによって導かれたのでしょうか?
- 有望だがまだ安定していない実験はどれですか?
- プロジェクトは次に何を学ぼうとしていますか?

そのため、ドキュメントには別の**メモ**レーンが含まれるようになりました。

## ここに属するもの

メモは意図的に不規則です。投稿には次のようなものがあります。

- 1週間のコンパイラ作業後の開発ブログ
- 新しいクレートのアーキテクチャに関する記述
- Musea、LSP、または Vite 統合に関する設計メモ
- 便利ですが、バージョン タグに関連付けられていない短いプロジェクトの更新

## すべてをリリースノートに記載しない理由

リリース ノートは出荷時の変更に合わせて最適化されています。それらは明確で実行可能なものである必要があります。

メモには、機能の背後にあるコンテキスト、ロードマップの形状、読者が長期的にプロジェクトを理解するのに役立つ考え方など、より広範なストーリーテリングのための余地が生まれます。

## 書き込み方向

疑問がある場合:

- 投稿で出荷されたものを発表する場合は**リリース ノート**を使用してください
- 投稿で思考、進捗状況、実験について説明している場合は**メモ**を使用してください
