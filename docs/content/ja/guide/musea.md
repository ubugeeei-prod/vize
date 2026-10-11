---
title: Musea
---

# Musea

> **⚠️ 進行中の作業:** Musea はまだ進化中です。ファイル形式、API、UI の動作は変更される可能性があります。

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

![Musea コンポーネント ギャラリー — ホーム](/musea-home.png)

Musea は、`*.art.vue` ファイルを使用して、Vue ネイティブ構文でコンポーネント バリアントを記述します。

## インストール

[Vite+ インストール ガイド](https://viteplus.dev/guide/install) から `vp` を一度インストールし、パッケージを追加します。

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

## Vite+ で設定する

[Vite+ 連携 (英語)](/guide/vite-plus/)を使うプロジェクトでは、既存の `plugins` に `musea()` を追加します。
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

複数のツールで既定値を共有したい場合は、共有設定を使います。
Vite+ のギャラリーだけなら、上の `vite.config.ts` にオプションをまとめられます。

`musea()` に渡すオプションは、共有設定より優先されます。複数のツールで使う既定値は `vize.config.ts` に、プレビュー専用の設定は `vite.config.ts` に置いてください。

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

共有設定に指定できる項目は、`include`、`exclude`、`basePath`、`storybookCompat`、`inlineArt` です。`previewCss`、`previewSetup`、`tokensPath`、`theme`、`storybookOutDir` は、`musea()` に直接渡してください。

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

`defineArt(source, options)` は、Musea が読み込むコンポーネントと、そのメタデータを宣言するコンパイラマクロです。メタデータは、従来 `<art>` に書いていた項目です。

コンポーネントは `defineArt("./MyButton.vue", { title: "MyButton" })` のように相対パスの文字列で指定することを推奨します。Musea は生成する実行時コードからそのコンポーネントをインポートし、言語サーバーも同じソースから Props とスロットを推論します。このパスは、パスの補完、見つからないファイルの診断、ドキュメントリンク、定義への移動にも使います。

render 関数で書いたコンポーネントには、`.ts`、`.tsx`、`.js`、`.jsx` ファイルも指定できます。
例: `defineArt("./MyButton.ts", { title: "MyButton" })`。Musea は default export を読み込み、
静的に宣言された runtime `props` と `emits` を分析画面と props パネルに表示します。

`<art title="..." component="...">` も互換性のために引き続き使えます。両方を指定した場合、明示的な `<art>` の属性が `defineArt` のメタデータより優先されます。

### バリアントローカル状態

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

インライン アート内で、`<Self>` を使用してホスト コンポーネントをレンダリングします。

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

パレット パイプラインは、コンポーネントのメタデータとアート定義からインタラクティブなコントロールを推測できます。

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

## 構成をプレビューする

プロジェクト CSS を挿入し、セットアップ コードをプレビューできます。

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

開発サーバーを起動し、`musea-vrt --ci --json` でスナップショットを検証します。ベースラインはスナップショットのディレクトリにコミットしてください。実行後は `vrt-report.json` / `vrt-report.html` と `snapshots/current` を確認し、失敗した場合は `snapshots/diff` も確認します。

意図した変更を確認してから、`--update` で再実行するか、対象のバリアントを `approve` で承認します。art ファイルを削除した場合は `clean` を実行し、古いベースラインが検証の漏れを隠さないようにします。

`--ci` は、画像の差分やプレビュー・撮影のエラー（ルートがない、ブラウザの起動に失敗した、セレクターがタイムアウトした場合）でゼロ以外の終了コードを返します。新しいベースラインは `new` と表示されるため、初回はローカルで `--update` を実行してください。

サンプルアプリ `examples/vite-musea` には、Playwright による VRT もあります。`vp run test:vrt` / `vp run test:vrt:update` で実行できます。スナップショットは `e2e/vrt/__snapshots__`、失敗時の成果物は `e2e/vrt/test-results`、HTML レポートは `playwright-report` に保存します。GitHub Actions は失敗時にこれらをアップロードするため、ベースライン・現在の画像・差分をレビューできます。

## アート ファイルを生成する

ジェネレーターを使用して、既存のコンポーネントから最初の `.art.vue` ドラフトを作成します。

```bash
vp exec musea-vrt generate src/components/Button.vue
```

生成されたファイルは、使用例を作るための下書きです。コミットする前に、バリエーション、タイトル、タグ、Props の範囲を確認してください。

## ストーリーブックの出力

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
