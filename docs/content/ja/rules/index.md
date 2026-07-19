---
title: ルール
---

<!-- Generated translation; source: rules/index.md -->

# ルール

Vize の診断は、1 つの大きなマトリックスとしてではなく、ルールとして文書化されます。各ルール ページには、
Bad/Good の例に近い検出動作なので、参照を ESLint ルールのように読み取ることができます
マニュアル。

## ページ

- [すべての緑青ルール](./all.md): すべての緑青ルールの実装に関する 1 ページのメタデータ テーブル、
  GitHub ソース リンクを含む。
- [Vue ルール](./vue.md): SFC テンプレート構造、Vue ディレクティブ、コンポーネント規約、および
  単一ファイルの Vue の正当性チェック。
- [タイプとスクリプトのルール](./type-and-script.md): TypeScript チェッカーによる診断と Vapor
  スクリプトの制限。
- [HTML ルール](./html.md): HTML の有効性とセマンティック マークアップのチェック。
- [アクセシビリティ ルール](./accessibility.md): ARIA、キーボード インタラクション、ラベル、ランドマーク、および
  アクセス可能なメディアのチェック。
- [SSRルール](./ssr.md): サーバーレンダリングとハイドレーションの危険。
- [Vapor ルール](./vapor.md): Vapor のみのテンプレート制約。
- [エコシステム ルール](./ecosystem.md): Nuxt、Vue Router、Ponia、vue-i18n のプリセットに基づくチェック
  Vue テスト ユーティリティと Void Vue。
- [Musea と CSS ルール](./musea-and-css.md): Musea のアートブロック チェックとスタイル診断。
- [クロスファイル ルール](./cross-file.md): によって発行されるプロジェクト グラフ診断
  `vize lint --cross-file`。

## プリセット

`essential` には、ほぼ常に有効にする必要がある正確性ルールが含まれています。 `happy-path` が追加
日々の Vue 開発のための実践的な衛生チェック。 `ecosystem` は広範なデフォルトから開始します
Vue Router、Vue I18n、Pinia、Vue Test Utils、Nuxt、および Void Vue チェックをバンドルして追加します。 `nuxt`
Nuxt 指向の SSR 期待値と Vapor 期待値が含まれます。 `opinionated` が最も広範です
内蔵プリセット。

`incremental` は空から始まります。ホストがルールを継承せずに特定のルールをオプトインしたい場合に使用します。
大きなプリセット。

## タイプ認識構成

セマンティック情報を必要とするルールは、`tsconfig.json` を通じて TypeScript プロジェクトを読み取ります。好む
共有環境名を保持する代わりに、`compilerOptions.types` またはプロジェクト参照に配置します。
Vize 構成内の別の `globals` リスト。
