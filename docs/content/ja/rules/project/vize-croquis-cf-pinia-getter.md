---
title: "vize:croquis/cf/pinia-getter"
---

# `vize:croquis/cf/pinia-getter`

Pinia の getter を `storeToRefs` なしで読んでおり、reactive のままになりません。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

## 悪い

Pinia の getter を `storeToRefs` なしで読んでおり、reactive のままになりません。

## 良い

getter は storeToRefs 経由で読んでください。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
