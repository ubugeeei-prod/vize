---
title: 診断の種類
---

<!-- Reviewed translation; source: guide/analysis-diagnostics.md -->

<span id="分析診断"></span>

# 診断の種類

Vize の診断は、単一ファイルの lint、ファイル横断解析、型チェックなどに分かれます。
個々のルールの動作、既定の診断レベル、プリセット、悪い例と良い例は、ルールのリファレンスで確認できます。

## ルールのリファレンス

- [ルール概要](../rules/index.md)
- [Vue ルール](../rules/vue.md)
- [アクセシビリティルール](../rules/accessibility.md)
- [型とスクリプトのルール](../rules/type-and-script.md)
- [HTML ルール](../rules/html.md)
- [SSR ルール](../rules/ssr.md)
- [Vapor ルール](../rules/vapor.md)
- [ファイル横断ルール](../rules/cross-file.md)
- [Musea と CSS のルール](../rules/musea-and-css.md)

<span id="診断ファミリー"></span>

## 診断の分類

Patina は、単一ファイルを対象に lint を実行します。ルール名は `vue/require-v-for-key` などです。
共有設定、CLI、JavaScript API、Oxlint 連携から設定できます。

ファイル横断解析の診断コードは `vize:croquis/cf/*` です。`vize lint --cross-file` は、
プロジェクト内の依存関係を解析し、provide / inject の対応、ID の重複、コンポーネントをまたぐ
リアクティビティの問題を検査します。

型情報を使う診断には TypeScript の型チェッカーが必要です。`tsconfig.json` の
`compilerOptions.types`、`paths`、プロジェクト参照など、TypeScript と同じ設定を使います。
これらの型や名前を Vize 用の `globals` に重ねて指定する必要はありません。

Musea と CSS の診断は、art ブロックやスタイルを解析した場合に実行されます。
通常の Vue テンプレートルールとは対象が異なるため、専用のリファレンスにまとめています。
