---
title: ルール
---

# ルール

[全ルール](./all.md)に、各ルールの目的、既定の重大度、適用範囲、設定、悪い例・良い例と理由をまとめています。
ページを移動せずに比較でき、一覧から必要なルールへ直接移動できます。英語と日本語で同じコード例を使っています。

Vite+ では `@vizejs/vite-plugin/vite-plus` の `defineConfig` を使い、`vue.linter` に設定します。
`vp run lint` で Vize と Oxlint の検査を実行します。既存の同名 script がある場合は
`vp run vize:lint` を使ってください。組み込みの `vp lint` は Vite+ 自身の検査です。

## ページ

- [全ルールの検索・一覧](./all.md)
- [ルールのオプション](./options.md)：型と既定値
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

### 既定で有効なルール

| プリセット | ルール数 | 有効になる内容 |
| --- | ---: | --- |
| `essential` | 85 | テンプレートとディレクティブの妥当性、Vue 3 で削除された構文と API（`vue/no-deprecated-*`、`script/no-deprecated-*`）、スクリプトのマクロ・`nextTick`・Options API の正しさ、`await` の後に登録されるライフサイクルフック・ウォッチャー・`expose`、`@vue/*` ではなく `vue` からの import、未使用のコンポーネント、`vue/no-v-html`、`vue/no-unsafe-url`、`html/id-duplication` |
| `happy-path`（既定） | 147 | `essential` に加え、テンプレートの書式（`v-bind` の省略記法、コンポーネントのイベント名のハイフン区切りを含む）、アクセシビリティ、HTML 適合性、SSR の危険、未使用のプロパティ、emit の宣言、props と emits の型 |
| `ecosystem` | 155 | `happy-path` に加え、エコシステムのライブラリー向けの検査 |
| `nuxt` | 213 | `opinionated` を Nuxt の自動インポートに合わせたもの |
| `opinionated` | 213 | `happy-path` に加え、より厳しい規約、props の型宣言、型ベースの `defineProps`/`defineEmits`、`<script setup>` のマクロ順序、camelCase のイベント名、`useTemplateRef`、`defineSlots`、props の既定値と emit の整理、`defineOptions`、`css/*` のスタイル規則 |
| `incremental` | 0 | なし |

ルール数にファイル横断のルールは含みません。各ルールの所属は[全ルールの一覧](./all.md)の「プリセット」列で確認できます。
`compiler.compatibility.vueVersion` が `2` または `2.7` のときは、Vue 3 で削除された構文と API の検査を無効にします。

`vue/no-use-v-else-with-v-for` は、同じ要素に条件とループを置く、Vue が受け付ける構文への警告です。条件をラッパーに分け、役割を明確にします。`script/no-deep-destructure-in-props` は `opinionated` と `nuxt` で有効な深さの規約です。コンパイラーの `propsDestructure` 設定は参照せず、リアクティブな分割代入を無効にした場合や `withDefaults` を使った場合は、入れ子の代入もコンパイルできます。どちらも警告であり、ルール名を指定して無効にできます。これらのルールは、`essential` と `incremental` では既定で無効です。

## 型情報を使う設定

型を使うルールには TypeScript プロジェクトと `typeAware` が必要です。グローバル名の型は
`tsconfig.json` の `compilerOptions.types` や project references で管理してください。
