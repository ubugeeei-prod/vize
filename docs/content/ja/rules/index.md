---
title: ルール
description: プリセットで Vize の lint ルールを有効にし、ルールごとに調整します。
---

# ルール

Vize のリンターは、Vue の単一ファイルコンポーネントのテンプレート、script、style と、ファイル間の関係を
検査します。プリセットを選び、必要に応じて個々のルールを強めたり、弱めたり、無効にしたりします。

## lint を有効にする

Vite+ では `@vizejs/vite-plugin/vite-plus` の `defineConfig` を使い、`vue.linter` に設定して
`vp run lint` を実行します。このコマンドで Vize のルールと Oxlint の両方が動きます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      preset: "happy-path",
      rules: {
        "vue/no-v-html": "off",
        "a11y/alt-text": "error",
      },
    },
  },
});
```

各ルールには `off`・`warn`・`error` を指定します。設定項目のあるルールは `vue.linter.ruleOptions` から
読み取ります。[ルールのオプション](./options.md)を参照してください。

プロジェクトに `lint` という名前の script がすでにある場合は、`vp run vize:lint` を使ってください。
組み込みの `vp lint` は Vite+ 自身の検査で、Vize のルールは実行しません。

## プリセット

プリセットは、最初に有効にするルールの組み合わせです。既定は `happy-path` です。

| プリセット | ルール数 | 有効になる内容 |
| --- | ---: | --- |
| `essential` | 85 | テンプレートとディレクティブの妥当性、Vue 3 で削除された構文と API（`vue/no-deprecated-*`、`script/no-deprecated-*`）、スクリプトのマクロ・`nextTick`・Options API の正しさ、`await` の後に登録されるライフサイクルフック・ウォッチャー・`expose`、`@vue/*` ではなく `vue` からの import、未使用のコンポーネント、`vue/no-v-html`、`vue/no-unsafe-url`、`html/id-duplication` |
| `happy-path`（既定） | 147 | `essential` に加え、テンプレートの書式（`v-bind` の省略記法、コンポーネントのイベント名のハイフン区切りを含む）、アクセシビリティ、HTML 適合性、SSR の危険、未使用のプロパティ、emit の宣言、props と emits の型 |
| `ecosystem` | 155 | `happy-path` に加え、エコシステムのライブラリー向けの検査 |
| `nuxt` | 213 | `opinionated` を Nuxt の自動インポートに合わせたもの |
| `opinionated` | 213 | `happy-path` に加え、より厳しい規約、props の型宣言、型ベースの `defineProps`/`defineEmits`、`<script setup>` のマクロ順序、camelCase のイベント名、`useTemplateRef`、`defineSlots`、props の既定値と emit の整理、`defineOptions`、`css/*` のスタイル規則 |
| `incremental` | 0 | なし。指定したルールだけを有効にするときの出発点です。 |

ルール数にファイル間ルールは含みません。`compiler.compatibility.vueVersion` が `2` または `2.7` のときは、
Vue 3 で削除された構文と API の検査を無効にします。

## ルールを探す

| ページ | 内容 |
| --- | --- |
| [全ルール](./all.md) | すべてのルールをカテゴリ別に一覧にし、有効にする最小のプリセットを示します |
| [Vue](./vue.md) | SFC のテンプレート、ディレクティブ、コンポーネント、各ブロック、Vue 3 への移行 |
| [型と script](./type-and-script.md) | `<script setup>`、コンパイラーマクロ、リアクティビティ、型情報を使う検査 |
| [HTML](./html.md) | HTML の妥当性と意味に沿ったマークアップ |
| [アクセシビリティ](./accessibility.md) | ARIA、キーボード操作、ラベル、ランドマーク、メディアの代替手段 |
| [SSR](./ssr.md) | サーバーレンダリングとハイドレーションの危険 |
| [Vapor](./vapor.md) | Vapor モードでコンパイルするコンポーネントの制約 |
| [エコシステムと Nuxt](./ecosystem.md) | Nuxt、Vue Router、Pinia、Vue I18n、Vue Test Utils、Void Vue |
| [Musea と CSS](./musea-and-css.md) | Musea の `.art.vue` ストーリーとプレーン CSS |
| [petite-vue](./petite-vue.md) | petite-vue を使う HTML ページ |
| [ファイル間](./cross-file.md) | コンポーネント、provide/inject、ルーターをまたぐ検査。`crossFile: true` で有効にします |
| [ルールのオプション](./options.md) | 型付きの `ruleOptions` と既定値 |
| [ESLint からの移行](./migration.md) | `eslint-plugin-vue` のルール名と Vize の対応、および差異 |

各ルールのページは、検出する内容、既定の重大度、プリセットから始まります。続いて、ルールが診断する悪い例と
診断しない良い例をそれぞれ一行の理由とともに示し、そのルールを有効にする設定を載せています。例は、リンターの
テストが実際に実行しているものと同じです。

## 型情報を使うルール

`type/` のルールは、`tsconfig.json` の TypeScript プロジェクトから型情報を読み取ります。プリセットまたは
`rules` で有効にしたうえで、`vue.linter` に `typeAware: true` を指定してください。グローバル名の型は、Vize の
設定に別の `globals` を置くのではなく、`compilerOptions.types` や project references で宣言します。
