---
title: 実際のプロジェクトでの検証
description: Vize は、実際のプロジェクトで検証する段階に入ります。これからは実際のアプリケーションがテストスイートです。v1.0.0 までの道筋も示します。
---

<!-- Reviewed translation; source: blog/notes/2026-06-07-real-world-testing.md; scope: complete document -->

# 実際のプロジェクトでの検証

<div class="blog-post-meta">
  <span class="blog-meta-chip">
    <span>
      <span class="blog-meta-label">公開</span>
      <span class="blog-meta-value">2026-06-07</span>
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

<video class="blog-post-video" src="/blog/vize-real-world-testing.mp4" controls muted playsinline loop preload="metadata" aria-label="Real World Testing PV"></video>

Vize は新しい段階に入ります。

これまでの開発では、機能の実装と基盤づくりを進め、専用のテストスイートや検証用に作ったサンプルで動作を確かめてきました。

次の段階は、これまでとは異なります。

Vize を試すための **実際のプロジェクト** を、積極的に募集しています。

## 目的

この段階の目的は、互換性の不足、仕様の抜け、性能のボトルネック、そして本番のコードベースでしか現れないエッジケースを見つけることです。

Vue のアプリケーション、ライブラリ、フレームワーク、ツールをメンテナンスしている方は、Vize で動かしてみた結果をぜひ教えてください。

不具合報告、再現手順、ベンチマークの結果、互換性についての発見のひとつひとつが、最初の安定版リリースに近づく力になります。

## まだ実験段階です。正しさを最優先にします

Vize は、今も実験的なものとして扱ってください。破壊的な変更が入る可能性があり、修正が必要な箇所も残っています。場合によっては、Vue と動作が異なることもあります。

この段階で重視するのは、機能の追加ではなく正しさです。これからは、実際のアプリケーションがテストスイートになります。修正が必要なものを見つけたら、ぜひ報告してください。ひとつひとつの報告が、コンパイラ、言語仕様、そしてエコシステム全体の改善につながります。

## 協力のお願い

不具合報告や PR をお待ちしています。検証の対象にできる、ある程度の規模の Vue プロジェクトも積極的に募集しています。コードベースが大きく、実際に使われているものであるほど、得られる情報の価値は高くなります。規模の大きい Vue のアプリケーション、ライブラリ、フレームワーク、ツールをメンテナンスしている方、またはご存じの方は、不具合を報告するか、ご連絡ください。Vize をそのプロジェクトで実行します。不具合報告、再現手順、ベンチマークの結果は、どれも歓迎します。

プレイグラウンドでの出力の確認、既存のテストケースの読み方、`vize check --profile` によるプロファイル、E2E / VRT の検証対象としてプロジェクトを提供する方法は、[テストとフィードバック](../../guide/testing.md)のガイドにまとめています。

## v1.0.0 までの道筋

現在の段階は **実際のプロジェクトでの検証** です。

この段階を終えたら、次の順に進みます。

- v1.0.0-alpha
- v1.0.0-beta
- v1.0.0-rc
- v1.0.0

alpha、beta、リリース候補の各段階では、安定化、エコシステムとの互換性、性能の改善、長期的なメンテナンスの保証に取り組みます。

目指しているのは、1.0 を急いで出すことではありません。1.0 と呼ぶにふさわしいものにすることです。

Vize のこれからに関わってみたい方にとって、今が参加するのに最適な時期です。
