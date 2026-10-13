---
title: 開発ノートを設ける理由
description: 変更履歴だけでは伝えきれない背景や設計の考え方を、開発ノートとして残す理由。
---

<!-- Reviewed translation; source: blog/notes/2026-03-26-why-vize-needs-notes.md -->

<a id="ノートレーン"></a>

# 開発ノートを設ける理由

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

リファレンスは「どう使うか」を調べるのに向いています。一方で、次のような背景までは十分に伝えられません。

- なぜ、この機能を追加したのか
- どのようなトレードオフを考えて、この設計を選んだのか
- 有望でも、まだ安定していない実験は何か
- プロジェクトとして、次に何を学びたいのか

そうした背景を伝えるため、ドキュメントに**開発ノート**を設けました。

<a id="ここに属するもの"></a>

## 開発ノートに書くこと

開発ノートは不定期に公開します。たとえば、次のような記事です。

- 1 週間のコンパイラ開発を振り返る記録
- 新しいクレートのアーキテクチャについての解説
- Musea、LSP、Vite 統合の設計メモ
- 特定のバージョンに結びつかなくても、共有する価値がある近況

<a id="すべてをリリースノートに記載しない理由"></a>

## リリースノートと分ける理由

リリースノートは、公開済みの変更を伝えるためのものです。何が変わり、利用者がどう対応すればよいかを明確にします。

開発ノートでは、機能を作った背景、ロードマップの方向性、設計を考える過程も扱います。継続して読むことで、プロジェクトへの理解が深まる記事を目指します。

<a id="書き込み方向"></a>

## どちらに書くか

迷ったときは、次のように使い分けます。

- 公開済みの変更を知らせるなら、**リリースノート**
- 考え方、開発の進捗、実験を伝えるなら、**開発ノート**
