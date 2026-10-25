---
title: Musea
---

<!-- Reviewed translation; source: guide/musea.md; scope: complete document -->

# Musea

> [!WARNING]
> Musea は開発を続けています。ファイル形式、API、UI の動作は変更される可能性があります。

Musea は、自分のコンポーネントをギャラリーで一覧し、状態の違いを比較・検証するためのツールです。
[すぐに使える UI の使用例](./ui/index.md)を試してから、自分のコンポーネントのバリエーションを `*.art.vue` に書きます。
カテゴリーごとに整理した使用例を、実際の Vue の状態とともに操作できます。

| 目的                                           | 使う機能                                      |
| ---------------------------------------------- | --------------------------------------------- |
| 通常・loading・disabled・エラー状態を比較する  | 名前を付けたバリエーションを持つ art ファイル |
| props とデザイントークンを確認する             | ギャラリーの props パレットとトークン画面     |
| 見た目の変更やアクセシビリティの問題を検出する | CI での `musea-vrt`                           |

- `vize_musea` は Rust のコアです。`*.art.vue` の解析、ドキュメントと Props パレットの生成、バリエーションの自動生成、VRT データの準備を行います。
- `@vizejs/vite-plugin-musea` は、現在推奨するギャラリーと開発サーバーの利用方法です。
- `musea-vrt` は、見た目の回帰テスト、アクセシビリティ監査、スナップショットの承認・整理、art ファイルの生成に使います。

## 概要

![Musea のコンポーネントギャラリーのホーム画面](/musea-home.png)

Musea では、`*.art.vue` ファイルに Vue の構文でコンポーネントのバリアントを書きます。

## インストール

[Vite+ のインストールガイド](https://viteplus.dev/guide/install)に沿って `vp` を用意し、パッケージを追加します。

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

## Vite+ で設定する

[Vite+ 連携](./vite-plus.md)を使うプロジェクトでは、既存の `plugins` に `musea()` を追加します。
ヘルパーが Vize の Vue コンパイラーを設定します。

```ts
// vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    musea({
      include: ["**/*.art.vue"],
      basePath: "/__musea__",
    }),
  ],
});
```

`@vitejs/plugin-vue` を使う既存の Vite+ プロジェクトからは、次の差分で移行できます。
他の Vite オプションとプラグインは維持してください。

```ts annotate="remove:1,2,7;add:3,4,8"
import { defineConfig } from "vite-plus";
import vue from "@vitejs/plugin-vue";
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [vue()],
  plugins: [musea({ include: ["**/*.art.vue"] })],
});
```

dev サーバーを起動して、設定した Musea のルートを開きます。

```bash
vp dev
```

```txt
http://localhost:5173/__musea__
```

CLI ラッパー、静的ホスト、テストの要件は [Musea のホスティング](./musea-hosting.md) を参照してください。

## 共有構成

共有の既定値は `vite.config.*` のトップレベルの `vue.musea` にまとめます。`musea()` に直接渡したオプションが、その項目の共有設定より優先されます。専用設定は廃止したため、`vue.musea` へ移行してください。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [musea({ storybookCompat: false })],
  vue: {
    musea: {
      include: ["src/**/*.art.vue"],
      exclude: ["node_modules/**", "dist/**"],
      basePath: "/__musea__",
      inlineArt: false,
    },
  },
});
```

共有設定に指定できるのは `include`、`exclude`、`basePath`、`storybookCompat`、`inlineArt` です。`previewCss`、`previewSetup`、`tokensPath`、`theme`、`storybookOutDir` は `musea()` に直接渡します。

`musea-vrt` も同じ Vite 設定から `include`、`exclude`、`vrt` を読み込みます。`vrt.outDir` はスナップショットの保存先で、プラグインに渡した `vrt` の各項目が共有設定より優先されます。`vize musea new` は既定の探索規則で Art ファイルを作成し、専用設定ファイルは生成しません。

<span id="アート-ファイル"></span>

## art ファイル

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

`defineArt(source, options)` は、Musea が読み込むコンポーネントと、そのメタデータを宣言するコンパイラマクロです。メタデータは、従来 `<art>` に書いていた項目です。

コンポーネントは `defineArt("./MyButton.vue", { title: "MyButton" })` のように相対パスの文字列で指定することを推奨します。Musea は生成する実行時コードからそのコンポーネントをインポートし、言語サーバーも同じソースから Props とスロットを推論します。このパスは、パスの補完、見つからないファイルの診断、ドキュメントリンク、定義への移動にも使います。

render 関数で書いたコンポーネントには、`.ts`、`.tsx`、`.js`、`.jsx` ファイルも指定できます。
例: `defineArt("./MyButton.ts", { title: "MyButton" })`。Musea は default export を読み込み、
静的に宣言された runtime `props` と `emits` を分析画面と props パネルに表示します。

`<art title="..." component="...">` も互換性のために引き続き使えます。両方を指定した場合、明示的な `<art>` の属性が `defineArt` のメタデータより優先されます。

<span id="バリアントローカル状態"></span>

### バリアントごとの状態

ルートの `<script setup>` の状態は、デフォルトでバリアントごとに分離されます。各バリアントで別々にセットアップを実行するため、あるバリアントの ref や computed の値が、別のバリアントと共有されることはありません。

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

すべてのバリアントで1つのセットアップインスタンスを共有したい場合に限り、`<script setup isolate="false">` を使ってください。

```art-vue
<script setup lang="ts" isolate="false">
import { ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const sharedCount = ref(0);
</script>
```

<span id="解剖学"></span>

### 構成要素

| 要素/マクロ                      | 目的                                           |
| -------------------------------- | ---------------------------------------------- |
| `defineArt(source, options)`     | 対象のコンポーネントと art のメタデータ        |
| `defineArt(...).title`           | 表示名                                         |
| `defineArt(...).category`        | サイドバーのグループ化                         |
| `defineArt(...).status`          | オプションのステータスバッジ                   |
| `defineArt(...).tags`            | タグの検索とフィルタリング                     |
| `<script setup>`                 | デフォルトのバリアントローカルセットアップ状態 |
| `<script setup isolate="false">` | すべてのバリアント間でセットアップ状態を共有   |
| `<art>`                          | バリアントをまとめるルートのブロック           |
| `<art title component ...>`      | 互換性メタデータ属性                           |
| `<variant>`                      | 名前付きコンポーネントのバリエーション         |
| `default`                        | デフォルトのバリアントをマークします           |
| `args`、`viewport`、`skip-vrt`   | オプションのバリアント構成                     |

バリアントがコンポーネントの仕様の一部である場合は、art ファイルをコンポーネントの隣に置きます。

```txt
src/components/Button.vue
src/components/Button.art.vue
```

多数のコンポーネントを横断する使用例をまとめる場合や、Nuxt がコンポーネントのディレクトリを自動検出する場合は、別の `stories` または `art` ディレクトリに置きます。

```txt
src/components/Button.vue
stories/forms/Button.art.vue
stories/navigation/Menu.art.vue
```

## インラインアート

`inlineArt` を有効にすると、`<art>` ブロックを含む通常の `.vue` ファイルもギャラリーに表示できます。小さなコンポーネントとその使用例を、同じファイルにまとめたい場合に使えます。

```ts
musea({
  inlineArt: true,
});
```

インライン art の中では、`<Self>` で、その art を持つコンポーネント自身をレンダリングします。

## ギャラリーの機能

![Musea コンポーネントの詳細 - バリアント](/musea-component.png)

Musea は次のことを実現できます。

- コンポーネントとバリアントのメタデータ表示
- Props パレットの生成
- デザイントークンの表示
- アクセシビリティチェック
- 見た目の回帰テストの補助
- 必要に応じた Storybook 互換形式の出力

<span id="小道具パレット"></span>

## Props パレット

![Musea の Props パネル](/musea-props.png)

Props パレットは、コンポーネントのメタデータと art の定義から、操作用のコントロールを推測します。

Props Controls の **Copy link** は、現在の値、追加したコントロール、削除した props、slot の内容を
共有リンクに含めます。リンクを開くと Props タブが選択され、ブラウザ内の保存値より URL の状態が
優先されます。**Reset** は現在の既定値に戻し、ほかのクエリと variant のフラグメントを保持したまま
共有状態を URL から削除します。

共有状態は Art ファイルに紐づくバージョン付き JSON です。slot は文字列として表示します。
DOM の HTML・イベントハンドラ用 props、JSON にできない値、エンコード後 24,000 文字を超える状態は
共有できません。不正な URL は理由を表示し、通常の保存値または既定値を使います。URL には公開して
よい値だけを含めてください。

Vue 3.5.41 は、名前が `__proto__` のコンポーネント prop を適用しません。この key をエディターの
データに保存できても、プレビューに適用されたことにはなりません。この prop 名は避けてください。
エディターでの明示的な拒否とリリース検証は
[#8533](https://github.com/ubugeeei-prod/vize/issues/8533) で追跡しています。

## デザイントークン

![Musea のデザイントークン](/musea-tokens.png)

`@vizejs/vite-plugin-musea` は、Style Dictionary と互換性のあるトークンファイルを読み込み、ギャラリーの UI に表示できます。

```ts
musea({
  tokensPath: "src/tokens.json",
});
```

<span id="構成をプレビューする"></span>

## プレビューに CSS とプラグインを追加する

プレビューにプロジェクトの CSS を読み込み、セットアップ用のコードを実行できます。

```ts
musea({
  previewCss: ["src/styles/main.css", "src/styles/musea-preview.css"],
  previewSetup: "musea.preview.ts",
});
```

`vue-i18n` や `vue-router` の導入に加え、[ブランド・テーマ・言語のグローバルコントロール](./musea-globals.md)も設定できます。

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

<span id="visual-regression-testing"></span>

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

CI では一般に、Vite サーバーを起動しておき、そのサーバーに対してスナップショットのコマンドを実行します。

```bash
vp dev --host 0.0.0.0
vp exec musea-vrt --base-url http://localhost:5173 --ci --json
```

開発サーバーを起動し、`musea-vrt --ci --json` でスナップショットを検証します。ベースラインはスナップショットのディレクトリにコミットしてください。実行後は `vrt-report.json` / `vrt-report.html` と `snapshots/current` を確認し、失敗した場合は `snapshots/diff` も確認します。

意図した変更を確認してから、`--update` で再実行するか、対象のバリアントを `approve` で承認します。art ファイルを削除した場合は `clean` を実行し、古いベースラインが検証の漏れを隠さないようにします。

`--ci` は、画像の差分やプレビュー・撮影のエラー（ルートがない、ブラウザの起動に失敗した、セレクターがタイムアウトした場合）でゼロ以外の終了コードを返します。新しいベースラインは `new` と表示されるため、初回はローカルで `--update` を実行してください。

サンプルアプリ `examples/vite-musea` には、Playwright による VRT もあります。`vp run test:vrt` / `vp run test:vrt:update` で実行できます。スナップショットは `e2e/vrt/__snapshots__`、失敗時の成果物は `e2e/vrt/test-results`、HTML レポートは `playwright-report` に保存します。GitHub Actions は失敗時にこれらをアップロードするため、ベースライン・現在の画像・差分をレビューできます。

<span id="アート-ファイルを生成する"></span>

## art ファイルを生成する

ジェネレーターを使用して、既存のコンポーネントから最初の `.art.vue` ドラフトを作成します。

```bash
vp exec musea-vrt generate src/components/Button.vue
```

生成されたファイルは、使用例を作るための下書きです。コミットする前に、バリエーション、タイトル、タグ、Props の範囲を確認してください。

<span id="ストーリーブックの出力"></span>

## Storybook 向けに出力する

Musea の art ファイルを Storybook で使う場合は、Storybook 互換の CSF 生成を有効にします。

```ts
musea({
  storybookCompat: true,
  storybookOutDir: ".storybook/stories",
});
```

## CLI ステータス

Rust CLI にも `vize musea` はありますが、現在推奨するのは Vite プラグインを使う方法です。専用のギャラリー機能が安定するまでは、Rust のサブコマンドは実験的な機能として扱ってください。

Rust のサブコマンドでは、最初の art プロジェクトを生成できます。

```bash
vize musea new
```

## 関連パッケージ

- `@vizejs/vite-plugin-musea`
- `@vizejs/musea-mcp-server`
- `vize_musea`
