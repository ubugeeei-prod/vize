---
title: 分析診断
---

<!-- Generated translation; source: guide/analysis-diagnostics.md -->

# 分析診断

このページでは、Vize 診断がどのように構成されているかについて説明します。詳細なルール参照は現在、
ルール セクション: 各ルールの動作、デフォルトの重大度、事前設定されたカバレッジ、および不良/良好を維持できます。
例も一緒に。

## ルールのリファレンス

- [ルール概要](../rules/index.md)
- [Vueルール](../rules/vue.md)
- [アクセシビリティルール](../rules/accessibility.md)
- [タイプとスクリプトのルール](../rules/type-and-script.md)
- [HTMLルール](../rules/html.md)
- [SSRルール](../rules/ssr.md)
- [蒸気ルール](../rules/vapor.md)
- [ファイル間ルール](../rules/cross-file.md)
- [Musea と CSS ルール](../rules/musea-and-css.md)

## 診断ファミリー

Patina ルールは単一ファイルの lint ルールです。 `vue/require-v-for-key` などの名前が使用され、次のようになります。
`vize.config.*`、CLI、JavaScript API、および Oxlint ブリッジから構成されます。

クロスファイル診断では、`vize:croquis/cf/*` コードを使用します。それらは次によって放出されます。
Vize がプロジェクト グラフを構築した後、`vize lint --cross-file` でプロバイダーを比較できるようになります。
インジェクター、重複した ID を追跡し、コンポーネントの境界を越えて反応性の危険性を特定します。

型認識診断では TypeScript チェッカーを使用します。同じプロジェクト構成が必要です。
TypeScript は、`compilerOptions.types`、`paths`、プロジェクトを含む `tsconfig.json` を認識します。
参考文献。 Vize では、これらの名前に対して別の `globals` リストを必要としません。

Musea および CSS 診断は、ライブラリに基づいたルールです。これらは、Musea のアート ブロックまたはスタイル コンテンツのときに実行されます。
これらは標準の Vue テンプレート ルールの一部ではないため、解析され、個別に文書化されます。
表面。
