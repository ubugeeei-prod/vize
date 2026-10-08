---
title: ルール
---

# ルール

[全ルール](./all.md)に、各ルールの目的、既定の重大度、適用範囲、設定、悪い例・良い例と理由をまとめています。
ページを移動せずに比較でき、一覧から必要なルールへ直接移動できます。英語と日本語で同じコード例を使っています。

Vite+ では `@vizejs/vite-plugin/vite-plus` の `defineConfig` を使い、`lint.vize` に設定します。
`vp run lint` で Vize と Oxlint の検査を実行します。既存の同名 script がある場合は
`vp run vize:lint` を使ってください。組み込みの `vp lint` は Vite+ 自身の検査です。

- [全ルールの検索・一覧](./all.md)
- [ルール オプション](./options.md)：型と既定値
- [ESLint からの移行対応表](./migration.md)
- [ファイル間の検査と Router の例](./cross-file.md)
- [Vue の検査](./vue.md)
- [型と script の検査](./type-and-script.md)
- [HTML](./html.md)・[アクセシビリティ](./accessibility.md)
- [SSR](./ssr.md)・[Vapor](./vapor.md)・[エコシステム](./ecosystem.md)
- [Musea と CSS](./musea-and-css.md)

## プリセット

`essential` は基本的な正しさ、`happy-path` は日常的な改善、`opinionated` は厳しい規約を検査します。
`ecosystem` は対応ライブラリー向け、`nuxt` は Nuxt 向けの規則を含みます。`incremental` は空の状態から
必要なルールだけを有効にするためのプリセットです。実際の所属は[全ルールの一覧](./all.md)を参照してください。

型を使うルールには TypeScript プロジェクトと `typeAware` が必要です。グローバル名の型は
`tsconfig.json` の `compilerOptions.types` や project references で管理してください。
