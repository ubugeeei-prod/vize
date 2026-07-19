---
title: 博物館
---

<!-- Generated translation; source: guide/musea.md -->

# 美術館

> **⚠️ 進行中の作業:**Musea はまだ進化中です。ファイル形式、API、UI の動作は変更される可能性があります。

Musea は、Vize のアート ファイルおよびコンポーネント ギャラリーのツールチェーンです。

- `vize_musea` は、`*.art.vue` の解析、ドキュメントの生成、小道具パレットの構築のための Rust コアです。
  バリアントの自動生成と VRT データの準備。
- `@vizejs/vite-plugin-musea` は、現在推奨されるギャラリーおよび開発サーバーのワークフローです。
- `musea-vrt` は、視覚的な回帰スナップショット、監査、承認、クリーンアップ、および
  生成されたアートファイル。

## 概要

![Musea コンポーネント ギャラリー — ホーム](/musea-home.png)

Musea は、`*.art.vue` ファイルを使用して、Vue ネイティブ構文でコンポーネント バリアントを記述します。

## インストール

[Vite+ インストール ガイド](https://viteplus.dev/guide/install) から `vp` を一度インストールし、パッケージを追加します。

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

## 推奨される使用法: Vite プラグイン

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
      previewSetup: "musea.preview.ts",
    }),
  ],
});
```

通常の Vite dev サーバーを実行し、設定された Musea ルートを開きます。

```bash
vp dev
```

```txt
http://localhost:5173/__musea__
```

`vize` npm パッケージをインストールすると、`vp exec vize musea` は Vite の便利なラッパーになります。

```bash
vp exec vize musea
vp exec vize musea --build
```

## 共有構成

`musea()` オプションは共有設定をオーバーライドします。安定したプロジェクトのデフォルトを `vize.config.ts` に入れて保持します。
`vite.config.ts` のプレビュー専用設定。

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  musea: {
    include: ["src/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
    basePath: "/__musea__",
    storybookCompat: false,
    inlineArt: false,
  },
});
```

共有構成は現在、`include`、`exclude`、`basePath`、`storybookCompat`、および
`inlineArt`。 `previewCss`、`previewSetup`、`tokensPath`、`theme`、および `storybookOutDir` を渡す
`musea()` に直接送信してください。

## アート ファイル

```art-vue
<script setup lang="ts">
import { ref } from "vue";

defineArt("./MyButton.vue", {
  title: "MyButton",
  category: "Components",
  status: "ready",
  tags: ["button", "ui", "input"],
});

const pressed = ref(false);
</script>

<art>
  <variant name="Default" default>
    <MyButton type="button" :pressed="pressed">Click me</MyButton>
  </variant>

  <variant name="Outlined">
    <MyButton type="button" outlined :pressed="pressed">Click me</MyButton>
  </variant>
</art>
```

`defineArt(source, options)` はコンパイラ マクロです。 Musea がロードするコンポーネントを宣言します。
加えて、`<art>` に存在していたメタデータも含まれます。次のような相対コンポーネント パス文字列を推奨します。
`defineArt("./MyButton.vue", { title: "MyButton" })`; Musea はそのコンポーネントを生成されたファイルにインポートします
ランタイム コードと言語サーバーは、prop とスロット推論に同じソースを使用します。
ソース文字列は、パス補完、未解決ファイルの診断、ドキュメントのリンク、および
定義に進みます。

`<art title="..." component="...">` は互換性のために引き続き機能し、明示的な `<art>` 属性も機能します
両方が存在する場合、`defineArt` メタデータをオーバーライドします。

### バリアントローカル状態

ルート `<script setup>` 状態は、デフォルトでバリアントごとに分離されます。各バリアントは独自のセットアップを受け取ります
インスタンスなので、あるバリアントの参照値と計算値が別のバリアントに漏洩することはありません。

```art-vue
<script setup lang="ts">
import { computed, ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>

<art>
  <variant name="Base" default>
    <Counter :count="count" />
  </variant>
  <variant name="Doubled">
    <Counter :count="doubled" />
  </variant>
</art>
```

アート ファイルに意図的に 1 つの共有設定が必要な場合にのみ、`<script setup isolate="false">` を使用してください。
すべてのバリアントにわたるインスタンス:

```art-vue
<script setup lang="ts" isolate="false">
import { ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const sharedCount = ref(0);
</script>
```

### 解剖学

| 要素/マクロ                      | 目的                                           |
| -------------------------------- | ---------------------------------------------- |
| `defineArt(source, options)`     | ターゲット コンポーネントとアートのメタデータ  |
| `defineArt(...).title`           | 表示名                                         |
| `defineArt(...).category`        | サイドバーのグループ化                         |
| `defineArt(...).status`          | オプションのステータスバッジ                   |
| `defineArt(...).tags`            | タグの検索とフィルタリング                     |
| `<script setup>`                 | デフォルトのバリアントローカルセットアップ状態 |
| `<script setup isolate="false">` | すべてのバリアント間でセットアップ状態を共有   |
| `<art>`                          | ルート バリアント ブロック                     |
| `<art title component ...>`      | 互換性メタデータ属性                           |
| `<variant>`                      | 名前付きコンポーネントのバリエーション         |
| `default`                        | デフォルトのバリアントをマークします           |
| `args`、`viewport`、`skip-vrt`   | オプションのバリアント構成                     |

バリアントがコンポーネントのコントラクトの一部である場合は、アート ファイルをコンポーネントの近くに置きます。

```txt
src/components/Button.vue
src/components/Button.art.vue
```

デザイン システムが多数の横断的なサンプルを所有している場合は、別の `stories` または `art` ディレクトリを使用します。
または、Nuxt コンポーネントの自動検出がコンポーネント ディレクトリをスキャンするとき:

```txt
src/components/Button.vue
stories/forms/Button.art.vue
stories/navigation/Menu.art.vue
```

## インラインアート

`inlineArt` が有効な場合、`<art>` ブロックを含む通常の `.vue` ファイルが
ギャラリー。これは、サンプルが同じファイル内に存在する必要がある小さなコンポーネントに役立ちます。

```ts
musea({
  inlineArt: true,
});
```

インライン アート内で、`<Self>` を使用してホスト コンポーネントをレンダリングします。

## ギャラリーの機能

![Musea コンポーネントの詳細 - バリアント](/musea-component.png)

Musea は次のことを実現できます。

- コンポーネントとバリアントのメタデータ
- プロップパレットの生成
- トークンビューを設計する
- アクセシビリティチェック
- 視覚的な回帰テストヘルパー
- リクエストに応じてストーリーブック互換の出力

## 小道具パレット

![美術館小道具パネル](/musea-props.png)

パレット パイプラインは、コンポーネントのメタデータとアート定義からインタラクティブなコントロールを推測できます。

## デザイントークン

![美術館デザイントークン](/musea-tokens.png)

`@vizejs/vite-plugin-musea` は、スタイル ディクショナリと互換性のあるトークン ファイルを取り込み、次の形式で公開できます。
ギャラリーUI。

```ts
musea({
  tokensPath: "src/tokens.json",
});
```

## 構成をプレビューする

プロジェクト CSS を挿入し、セットアップ コードをプレビューできます。

```ts
musea({
  previewCss: ["src/styles/main.css", "src/styles/musea-preview.css"],
  previewSetup: "musea.preview.ts",
});
```

これは、プレビュー iframe に `vue-i18n` や `vue-router` などのプラグインをインストールする場合に便利です。

```ts
// musea.preview.ts
import type { App } from "vue";
import { createI18n } from "vue-i18n";

export default function setup(app: App) {
  app.use(
    createI18n({
      legacy: false,
      locale: "en",
      messages: {
        en: {},
      },
    }),
  );
}
```

## 視覚的な回帰テスト

パッケージは `musea-vrt` バイナリを公開します。

```bash
vp exec musea-vrt --base-url http://localhost:5173
vp exec musea-vrt --update
vp exec musea-vrt --ci --json
vp exec musea-vrt --a11y
vp exec musea-vrt approve
vp exec musea-vrt approve "Button/*"
vp exec musea-vrt clean
```

一般的な CI フローでは、1 つのプロセスで Vite サーバーを起動し、それに対してスナップショット コマンドを実行します。

```bash
vp dev --host 0.0.0.0
vp exec musea-vrt --base-url http://localhost:5173 --ci --json
```

ワークフロー: スナップショット ディレクトリの下にベースラインをコミットし、スナップショット ディレクトリに対して `musea-vrt --ci --json` を実行します。
開発サーバーを実行し、`vrt-report.json`/`vrt-report.html` と `snapshots/current` を検査し、
失敗時は `snapshots/diff`。 `--update` (または選択したバリアントの場合は `approve`) を使用して再実行します。
意図的な変更を加えず、アート ファイルを削除した後に `clean` を実行して、古いベースラインによってギャップが隠れないようにしてください。
`--ci` は、視覚的な差分およびプレビュー/キャプチャ エラー (ルートの欠落、ブラウザーの欠落) に対してゼロ以外で終了します
失敗、セレクタのタイムアウト）;新しいベースラインは `new` として報告されるため、最初に `--update` をローカルで実行します。

サンプル アプリは、Playwright ネイティブの VRT パス (`examples/vite-musea`、経由で実行) も接続します。
`vp run test:vrt` / `vp run test:vrt:update`)。スナップショットは `e2e/vrt/__snapshots__` に存在します、失敗します
`e2e/vrt/test-results` のアーティファクト、および `playwright-report` の HTML レポート。 GitHub アクション
失敗時にそれらをアップロードすることで、レビュー担当者がベースライン、現在、および差分イメージを検査できるようになります。

## アート ファイルを生成する

ジェネレーターを使用して、既存のコンポーネントから最初の `.art.vue` ドラフトを作成します。

```bash
vp exec musea-vrt generate src/components/Button.vue
```

生成されたファイルが開始点となります。前に、バリエーション、タイトル、タグ、小道具の範囲を確認してください。
それをコミットしている。

## ストーリーブックの出力

Musea アート ファイルを Storybook セットアップにフィードする場合は、Storybook 互換の CSF 生成を有効にします。

```ts
musea({
  storybookCompat: true,
  storybookOutDir: ".storybook/stories",
});
```

## CLI ステータス

`vize musea` は Rust CLI に存在しますが、現在推奨されている Musea ワークフローは依然として Vite です
プラグインのパス。専用のギャラリーのワークフローが安定するまでは、Rust サブコマンドを実験的なものとして扱います。

Rust サブコマンドは、スターター アート プロジェクトの足場を築くことができます。

```bash
vize musea new
```

## 関連パッケージ

- `@vizejs/vite-plugin-musea`
- `@vizejs/musea-mcp-server`
- `vize_musea`
