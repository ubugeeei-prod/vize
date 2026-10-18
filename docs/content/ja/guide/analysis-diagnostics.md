---
title: 診断
description: Vize の診断の読み方と、対応するルールの調べ方、修正・設定・抑制の選び方を説明します。
---

<!-- Reviewed translation; source: guide/analysis-diagnostics.md; scope: complete document -->

<span id="分析診断"></span>

# 診断

Vize が報告する問題は、すべて「診断」として出力されます。診断には、ルール ID、重大度、位置、そして多くの場合は修正のヒントが含まれます。このページでは、診断の読み方、表示の変え方、診断を受け取ったあとの対応を説明します。

特定のルールを調べたい場合は、[すべてのルール](../rules/all.md)を直接開いてください。

## 診断を読む

問題を 2 つ含むコンポーネントを例にします。

```vue
<script setup lang="ts">
import { ref } from "vue";

const items = ref([{ id: 1, name: "Ada" }]);
</script>

<template>
  <ul>
    <li v-for="item in items">{{ item.name }}</li>
  </ul>
  <img src="/logo.png" />
</template>
```

`vize lint src/App.vue --help-level short --locale ja` を実行すると、次のように表示されます。

```text
  × [vize:vue/require-v-for-key] `v-for` で繰り返す <li> には `v-bind:key` が必要です
    ╭─[src/App.vue:9:9]
  8 │   <ul>
  9 │     <li v-for="item in items">{{ item.name }}</li>
    ·         ─────────────────────
 10 │   </ul>
    ╰────
  help: 各項目を一意に識別できる値を :key に指定してください
        Reference: docs/content/rules/vue.md


  ⚠ [vize:a11y/alt-text] <img>要素にはalt属性が必要です
    ╭─[src/App.vue:11:3]
 10 │   </ul>
 11 │   <img src="/logo.png" />
    ·   ───────────────────────
 12 │ </template>
    ╰────
  help: alt属性を追加してください: <img alt="説明"> または装飾的な画像には <img alt="">
        Reference: docs/content/rules/accessibility.md


1 error, 1 warning in 1 file
Linted 1 files in 1.8971ms
```

| 項目      | 例での表示                   | 意味                                                                                                                  |
| --------- | ---------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| 重大度    | `×` と `⚠`                   | `×` はエラー、`⚠` は警告です。エラーがあると、コマンドは終了コード 1 で終了します。                                   |
| ルール ID | `vize:vue/require-v-for-key` | 問題を報告したルールです。設定に書くときは、`vize:` より後ろの部分を使います。                                        |
| 位置      | `src/App.vue:9:9`            | ファイル、行、列です。その下で下線が引かれている部分が、対象の範囲です。                                              |
| ヘルプ    | `help: …`                    | ルールの目的と直し方です。`--help-level short` を付けなければ、修正後のコード例も表示されます。                       |
| Reference | `docs/content/rules/vue.md`  | ルールが属するカテゴリのページです。同じルールの悪い例と良い例は、[ルールリファレンス](../rules/index.md)にあります。 |

警告だけの場合、終了コードは 0 のままです。警告が 1 件でも残っていたら失敗させたい場合は、`--max-warnings 0` を指定します。

## 表示を変える

| オプション     | 値                                                                           | 用途                                                      |
| -------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------- |
| `--format`     | `text`（既定）、`plain`、`stylish`、`json`、`markdown`、`html`、`agent` など | ターミナル、CI のログ、レポート、ツール連携に合わせます。 |
| `--help-level` | `full`（既定）、`short`、`none`                                              | ヘルプを短くしたり、非表示にしたりします。                |
| `--locale`     | `en`（既定）、`ja`、`zh`                                                     | メッセージとヘルプの言語を切り替えます。                  |
| `--quiet`      |                                                                              | 集計結果だけを表示します。                                |

`--format plain` は診断を 1 行ずつ出力するので、CI のログで読みやすくなります。

```text
Patina lint report: 1 error, 1 warning in 1 file

src/App.vue
  src/App.vue:9:9 error vue/require-v-for-key `v-for` で繰り返す <li> には `v-bind:key` が必要です
    Reference: docs/content/rules/vue.md
    Help:
      各項目を一意に識別できる値を :key に指定してください
  src/App.vue:11:3 warning a11y/alt-text <img>要素にはalt属性が必要です
    Reference: docs/content/rules/accessibility.md
    Help:
      alt属性を追加してください: <img alt="説明"> または装飾的な画像には <img alt="">
```

`--format json` では、同じ情報を `ruleId`、`severity`（エラーは 2、警告は 1）、`line`、`column`、`endLine`、`endColumn`、`message`、`help` というフィールドで受け取れます。

## 対応を決める

1. **コードを直す**: [ルールリファレンス](../rules/index.md)で該当するルールを開き、悪い例と良い例を見比べます。`vize lint --fix` を実行すると、安全に自動修正できるものが適用されます。
2. **プロジェクトのルール設定を変える**: 重大度を `"error"`、`"warn"`、`"off"` のいずれかに設定します。

   ```ts
   import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

   export default defineConfig({
     lint: {
       vize: {
         rules: {
           "a11y/alt-text": "error",
           "vue/require-v-for-key": "off",
         },
       },
     },
   });
   ```

   重大度以外のオプションを受け取るルールは、[ルールのオプション](../rules/options.md)にまとめています。

3. **その箇所だけ抑制する**: 対象の直前の行にコメントを書きます。ほかのディレクティブは[コメントによる設定](./comment-annotations.md)で説明しています。

   ```vue
   <template>
     <ul>
       <!-- @vize:expected -->
       <li v-for="item in items">{{ item.name }}</li>
     </ul>
   </template>
   ```

<span id="診断ファミリー"></span>

## 診断の種類

| 種類                | ID の例                 | 診断を報告する機能                                                         |
| ------------------- | ----------------------- | -------------------------------------------------------------------- |
| 単一ファイルの lint | `vue/require-v-for-key` | `vize lint`、エディター、Oxlint プラグイン                           |
| ファイル横断        | `vize:croquis/cf/*`     | `vize lint --cross-file`（プロジェクト全体の依存関係を解析したあと） |
| 型情報を使う診断    | `type/*`                | `vize lint --type-aware`（TypeScript の型チェッカーを使用）          |
| Musea と CSS        | `musea/*`               | Musea の art ブロックやスタイルを解析する lint                       |

単一ファイルのルールは、Vize のリンターである Patina が実行します。Vite+ の設定、共有の `vize.config.*`、CLI、JavaScript API、Oxlint プラグインから設定できます。

ファイル横断の診断は、provide と inject の対応、ID の重複、コンポーネントをまたぐリアクティビティの問題を検査します。出力されるレポートは[ファイル間の複雑さ](./cross-file-complexity.md)で説明しています。

型情報を使う診断には、`tsconfig.json` の `compilerOptions.types`、`paths`、プロジェクト参照など、TypeScript と同じ設定が使われます。これらの名前を Vize 用の `globals` に重ねて指定する必要はありません。

Musea と CSS の診断は、通常の Vue テンプレートのルールとは対象が異なるため、専用のリファレンスにまとめています。

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
