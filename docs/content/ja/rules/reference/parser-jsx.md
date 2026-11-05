---
title: "parser/jsx"
description: "リンターが報告する JSX・TSX の構文解析診断と、その修正方法を説明します。"
---

<!-- Reviewed translation; source: rules/reference/parser-jsx.md -->

# `parser/jsx`

JSX・TSX モジュールの構文解析や中間表現への変換で問題が報告されると、リンターがこの診断を使います。
プリセットのルール一覧とは別に発生する処理上の診断です。自動修正はありません。

## 診断が出る条件

JSX・TSX の入力は、マークアップのルールが描画のルートを検査する前に、モジュールとして解析されます。
各診断は、パーサーまたは中間表現への変換処理が出したメッセージとソース範囲を保持します。
その処理がエラーを返すとエラー、警告を返すと警告になります。
変換処理が同じ ID と範囲で既存の構文診断を報告した場合、リンターは重複を取り除きます。

不正な JSX だけでなく、JavaScript・TypeScript の構文エラーを示す場合もあります。
メッセージ全体を読み、構文エラーなのか、変換処理が受け付けない構文なのかを確認してください。

## 悪い例

親の `div` を閉じる前に、JSX の `span` を閉じていません。

```tsx annotate="remove:1"
export default () => <div><span>Notice</div>;
```

## 良い例

入れ子の順序に従って、各 JSX 要素を閉じます。

```tsx annotate="add:1"
export default () => (
  <div>
    <span>Notice</span>
  </div>
);
```

## 修正方法

報告された範囲の構文を修正して、lint を再実行します。要素名の対応、式を囲む波括弧、
引用符、周囲のモジュール構文を確認してください。JSX は `.jsx`、TypeScript の JSX は
`.tsx` に保存し、選択される言語とソースを一致させます。

正しい構文が変換処理で拒否される場合は、小さな再現例に絞り、メッセージ全体とともに
不具合を報告してください。lint のプリセットを変えても、不正な構文は解析できるようになりません。

## 関連ページ

- [JSX と TSX](../../guide/jsx.md)
- [ネイティブの lint 実行設定](../../guide/lint-execution.md)
- [診断変換のソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine/jsx.rs)
- [JSX lint の振り分け処理](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine.rs)
