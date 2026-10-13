---
title: Nuxt
description: モジュールをひとつ追加するだけで、Nuxt プロジェクトの Vue コンポーネントを Vize でコンパイル・lint・プレビューできます。
---

<!-- Reviewed translation; source: integrations/nuxt.md; scope: complete document -->

# Nuxt

`@vizejs/nuxt` は、Nuxt プロジェクトに Vize を導入するためのモジュールです。Nuxt 標準の Vue コンパイラの代わりに、Rust で書かれた Vize のコンパイラで `.vue` ファイルをコンパイルします。あわせて Nuxt 向けの lint 設定を生成し、コンポーネントギャラリーの [Musea](../guide/musea.md) も利用できます。自動インポート、コンポーネント、ミドルウェア、SSR は、これまでどおり Nuxt が処理します。

> [!WARNING]
> Vize は開発中で、まだ本番利用できる段階ではありません。導入する前に、自分のプロジェクトでビルド結果と診断を見比べ、[対応状況](../stability.md)を確認してください。

## セットアップ

### 1. モジュールをインストールする

[Vite+ のインストールガイド](https://viteplus.dev/guide/install)に沿って `vp` を用意し、モジュールを追加します。

```bash
vp install @vizejs/nuxt
```

### 2. モジュールを登録する

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

`compiler: true` は既定値なので、`modules: ["@vizejs/nuxt"]` だけでも動きます。このページで紹介するオプションは、すべて `vize` キーの下に書きます。

### 3. Nuxt を起動する

いつもどおり開発サーバーを起動します。

```bash
vp run dev
```

モジュールは Nuxt の Vite 設定に `@vizejs/vite-plugin` を追加し、Nuxt 自身の変換処理はそのまま残します。ページ、レイアウト、自動インポートされるコンポーザブルやコンポーネントは、これまでと同じように動作します。

### 4. Vize がコンパイルしていることを確認する

コンパイラのデバッグログを有効にすると、Vize が処理したファイルを確認できます。

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: { debug: true },
  },
});
```

Nuxt ではコンポーネントを必要になった時点でコンパイルするため、`[vize] load: on-demand compiling ...` というログは、リクエストで実際に読み込まれたファイルにだけ出力されます。確認できたら `debug` は外してください。

<span id="options"></span>

## オプション

オプションはすべて `nuxt.config.ts` の `vize` の下に書きます。ほとんどのプロジェクトで必要になるのは、最初の 4 つです。

| オプション            | 型                                      | 既定値                     | 説明                                                                                                                             |
| --------------------- | --------------------------------------- | -------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `compiler`            | `boolean \| VizeNuxtCompilerOptions`    | `true`                     | Vue SFC を Vize でコンパイルします。オブジェクトを渡すと、その内容を `@vizejs/vite-plugin` に引き継ぎます。                      |
| `lint`                | `boolean \| VizeNuxtLintOptions`        | `true`                     | Nuxt 向けの Oxlint 設定を生成します。[lint](#lint) を参照してください。                                                          |
| `checker`             | `boolean \| VizeNuxtLintCheckerOptions` | `false`                    | 開発サーバーと並行して lint を実行し、ターミナルとブラウザーのオーバーレイに表示します。[開発中の lint](#lint-while-developing) を参照してください。 |
| `musea`               | `boolean \| MuseaOptions`               | `false`                    | Musea のギャラリーを配信します。[Musea](#musea) を参照してください。                                                             |
| `ui`                  | `boolean \| VizeNuxtUiOptions`          | `false`                    | `@vizejs/ui` のコンポーネントを自動登録します。[自動インポート](../guide/auto-imports.md#nuxt)を参照してください。               |
| `composables`         | `boolean \| VizeNuxtComposablesOptions` | `false`                    | `@vizejs/composable` の関数を自動インポートします。[自動インポート](../guide/auto-imports.md#nuxt)を参照してください。           |
| `bridge`              | `boolean \| VizeNuxtBridgeOptions`      | `true`                     | Nuxt の自動インポート、コンポーネント解決、i18n、非同期データのキー生成を、Vize がコンパイルしたモジュールにも適用します。       |
| `unocss`              | `boolean \| VizeNuxtUnoCssOptions`      | `true`                     | UnoCSS が元の `.vue` ソースを読めるようにします。`originalSource: false` で無効化、`maxBytes` で 1 ファイルあたりの上限を指定します。 |
| `dev.stylesheetLinks` | `boolean`                               | `true`                     | 開発時の SSR HTML から、重複したスタイルシートのリンクや安全でないリンクを取り除きます。                                         |
| `compatibility`       | `VizeNuxtCompatibilityOptions`          | 自動検出                   | 検出された Nuxt と Vue のメジャーバージョンを上書きします。[Nuxt 2 と Vue 2](#nuxt-2-and-vue-2) を参照してください。             |
| `nuxtMusea`           | `NuxtMuseaOptions`                      | `{ route: { path: "/" } }` | Musea のプレビューから見えるルート、ランタイム設定、fetch のモックを指定します。                                                 |

### コンパイラ

`compiler` には `@vizejs/vite-plugin` のオプションをそのまま渡せます。プロジェクトのルート、オンデマンドのコンパイル、依存パッケージ内の `.vue` ファイルの扱いは、Nuxt 向けの既定値が使われます。

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compatibility: {
      // Usually inferred automatically.
      // Nuxt 2 defaults to Vue 2 compatibility mode; Nuxt 3/4 defaults to Vue 3.
      vueVersion: 3,
    },
    compiler: {
      // Any @vizejs/vite-plugin option can be passed here.
      configMode: "auto",
      customRenderer: false,
      debug: false,
      handleNodeModulesVue: true,
      ignorePatterns: ["node_modules/**", ".nuxt/**", ".output/**"],
      precompileBatchSize: 64,
      scanPatterns: [], // Nuxt defaults to on-demand compilation
      sourceMap: true,
      vapor: false,
    },
    bridge: {
      autoImports: true,
      components: true,
      i18n: true,
      stableInjectedKeys: true,
    },
    unocss: {
      originalSource: {
        maxBytes: 2 * 1024 * 1024,
      },
    },
    dev: {
      stylesheetLinks: true,
    },
    musea: false,
  },
});
```

各コンパイラオプションの意味は [Vite プラグインのガイド](../guide/vite-plugin.md#compiler-options)にまとめています。`compiler: false` を指定すると、コンパイルは Nuxt 標準のコンパイラに任せたまま、lint や Musea だけを使えます。

<span id="lint"></span>

### lint

`lint` が有効な場合、モジュールはプロジェクトのルートに `.oxlint.vize.json` を書き出します。これは `@nuxt/eslint` が生成する設定の Oxlint 版で、Nuxt のバージョンとディレクトリ構成に合わせたルールが入ります。このファイルは `.gitignore` に追加してください。

プロジェクトに Oxlint や Vite の設定ファイルがまだない場合は、生成したファイルを読み込む `oxlint.config.mts` もあわせて作成します。既存の設定ファイルを書き換えることはありません。その場合は、自分の設定から `.oxlint.vize.json` を読み込んでください。

| オプション   | 既定値                | 説明                                                                                        |
| ------------ | --------------------- | ------------------------------------------------------------------------------------------- |
| `configFile` | `".oxlint.vize.json"` | 生成する設定ファイルの出力先です。lint のルート、またはその親ディレクトリに置く必要があります。 |
| `autoInit`   | `true`                | Oxlint や Vite の設定ファイルがないときに `oxlint.config.mts` を作成します。                |
| `rootDir`    | Nuxt のルート         | 生成される glob の基準になるディレクトリです。                                              |
| `devtools`   | `{ enabled: "lazy" }` | 解決後の lint 設定を Nuxt DevTools のタブに表示します。                                     |

`lint` には `@nuxt/eslint-config` と同じ機能スイッチも指定できます。`standalone`（既定は `true`）、`import`（既定は `true`）、`typescript`、`stylistic`、`tooling`、`formatters`、`nuxt` です。

lint の設定を Nuxt のルートの外に置いている場合は、`lint.autoInit: false` を指定してください。ルールの実行方法は [Oxlint プラグインのガイド](../guide/oxlint.md)で説明しています。

<span id="lint-while-developing"></span>

### 開発中の lint

`checker` を有効にすると、`@nuxt/eslint` の checker と同じように、開発サーバーと並行して Oxlint と Vize の lint ルールを実行します。

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    checker: true,
  },
});
```

| オプション    | 既定値                                    | 説明                                                         |
| ------------- | ----------------------------------------- | ------------------------------------------------------------ |
| `lintOnStart` | `true`                                    | 開発サーバーの起動時にプロジェクト全体を lint します。       |
| `cache`       | `true`                                    | 起動後は、変更されたファイルだけを lint します。             |
| `include`     | `<srcDir>/**/*.{js,jsx,ts,tsx,vue}`       | 検査の対象にするファイルです。                               |
| `exclude`     | `**/node_modules/**` とビルドディレクトリ | 検査の対象から外すファイルとディレクトリです。               |
| `emitWarning` | `true`                                    | 警告をターミナルに出力し、ブラウザーのオーバーレイにも表示します。 |
| `emitError`   | `true`                                    | エラーをターミナルに出力し、ブラウザーのオーバーレイにも表示します。 |
| `fix`         | `false`                                   | 報告する前に、安全な修正を適用します。                       |
| `formatter`   | `"stylish"`                               | Oxlint の出力形式です。                                      |

<span id="musea"></span>

## Musea

コンポーネントギャラリーは `musea` で有効にします。

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
    musea: {
      include: ["**/*.art.vue"],
      tokensPath: "assets/tokens.json",
      previewCss: ["assets/styles/main.css", "assets/styles/musea-preview.css"],
      previewSetup: "musea.preview.ts",
    },
    nuxtMusea: {
      route: { path: "/" }, // Musea UI route within __musea__
    },
  },
});
```

開発中は `/__musea__/` でギャラリーを開けます。art ファイル、デザイントークン、視覚的な回帰テストについては [Musea のガイド](../guide/musea.md)を参照してください。

### art ファイルの置き場所

Nuxt はコンポーネントディレクトリ内の `.vue` ファイルをすべて走査しますが、Musea の art ファイルも拡張子は `.vue` です。`*.art.vue` はコンポーネントディレクトリの外に置き、Musea にその場所を指定してください。

```txt
app/components/Tag.vue
stories/shared/Tag.art.vue
```

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    musea: {
      include: ["stories/**/*.art.vue"],
    },
  },
});
```

Musea を有効にすると、モジュールは `**/*.art.vue` を Nuxt のコンポーネント走査からも除外します。そのため、コンポーネントの隣に残っている art ファイルがコンポーネントとして登録されることはありません。

### プレビューで Nuxt の機能を使う

コンポーネントは `NuxtLink`、`useRoute`、`useNuxtApp`、`useRuntimeConfig`、データ取得のコンポーザブル、Nuxt の組み込みコンポーネントなどに依存していることがよくあります。Musea を単独の Vite 設定で動かす場合は、`@vizejs/musea-nuxt` を追加し、`previewSetup` からモックをインストールします。

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { musea } from "@vizejs/vite-plugin-musea";
import { nuxtMusea } from "@vizejs/musea-nuxt";

export default defineConfig({
  plugins: [
    nuxtMusea({
      route: { path: "/preview" },
      runtimeConfig: { public: { apiBase: "/api" } },
      fetchMocks: {
        "/api/user": { id: 1, name: "Ada" },
      },
    }),
    musea({
      previewSetup: "musea.preview.ts",
    }),
  ],
});
```

```ts
// musea.preview.ts
import { installNuxtMuseaMocks } from "@vizejs/musea-nuxt";
import { createI18n } from "vue-i18n";
import type { MuseaPreviewSetup } from "@vizejs/vite-plugin-musea";

export default ((app) => {
  installNuxtMuseaMocks(app, {
    route: { path: "/preview" },
    runtimeConfig: { public: { apiBase: "/api" } },
  });

  const i18n = createI18n({
    locale: "ja",
    messages: {
      ja: {
        /* ... */
      },
      en: {
        /* ... */
      },
    },
  });
  app.use(i18n);
}) satisfies MuseaPreviewSetup;
```

Nuxt アプリの中では、モジュールはこれらのモックをグローバルにはインストールしません。Nuxt 自身の `#imports` を置き換えてしまうためです。その場合は、本物の Nuxt のコンポーザブルが使われます。

<span id="nuxt-2-and-vue-2"></span>

## Nuxt 2 と Vue 2

モジュールは Nuxt 2、3、4 に対応しています。Vize のコンパイラは Vue 3 向けのコードを出力するため、Nuxt 2 では `.vue` のコンパイルを既存のコンパイラ（`@vitejs/plugin-vue2`、`vue-loader`、または Nuxt 自身）に任せ、lint と Musea だけを提供します。さらに古い Vue のプロジェクトでも、`vueVersion: 0.11`、`vueVersion: 1`、`vueVersion: "legacy"` を指定すれば同じように動作します。

バージョンは自動的に検出されます。ラッパーによって Nuxt Kit からバージョンが見えない場合は、明示的に指定してください。

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compatibility: {
      nuxtVersion: 2,
      vueVersion: 2,
    },
  },
});
```

## Vite プラグインを直接使う

Nuxt は Vite でビルドするので、モジュールを使わずに Vite プラグインを直接登録することもできます。その場合、ここまでに紹介した Nuxt 向けのブリッジや lint 設定の生成は使えません。

```ts
// nuxt.config.ts
import vize from "@vizejs/vite-plugin";

export default defineNuxtConfig({
  vite: {
    plugins: [vize()],
  },
});
```

## 仕組み

1. **Vite プラグイン**: モジュールが `@vizejs/vite-plugin` を登録し、`.vue` ファイルをコンパイルします。
2. **Vue プラグインとの互換性**: プラグインは `@vitejs/plugin-vue` と同じ API を公開するので、Nuxt が内部で行う Vue プラグインの確認もそのまま通ります。
3. **SSR**: サーバー側のビルドも Vize がコンパイルし、クライアントとサーバーの環境変数は分けて扱います。
4. **Nuxt の変換処理**: 自動インポート、コンポーザブル、ミドルウェアは Nuxt 自身の変換処理が担当します。この処理は、Vize がコンポーネントをコンパイルした後に実行されます。

公開されている `@vizejs/nuxt` は、`vize`、`@vizejs/vite-plugin`、`@vizejs/native` などの Vize のパッケージを、すべて同じバージョンに固定しています。

## 利用例: Vue Fes Japan 2026

[Vue Fes Japan 2026](https://vuefes.jp/2026) の Web サイトは、Nuxt 4 でこのモジュールを使っています。

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: false, // compiler disabled (using Nuxt's default)
    musea: {
      include: ["**/*.art.vue"],
      inlineArt: false,
      tokensPath: "assets/tokens.json",
      previewCss: ["assets/styles/main.css", "assets/styles/musea-preview.css"],
      previewSetup: "musea.preview.ts",
    },
  },
});
```

コンポーネントの開発とドキュメントには Musea を使い、本番ビルドは Nuxt 標準のコンパイラのままにしています。

## トラブルシューティング

- **pnpm で Pkl の設定を使う**: `@vizejs/nuxt` が依存する `vize` パッケージには、既定の `vize.pkl` が含まれています。pnpm では、このファイルが `node_modules/vize` に置かれないことがあります。Pkl の設定を使う場合は、`vize` を直接インストールしてください。

  ```bash
  vp install vize
  ```

- **`lint.configFile` に指定した設定が反映されない**: 設定ファイルは `lint.rootDir`、またはその親ディレクトリに置く必要があります。`.nuxt/` の中に置いた設定には、Oxlint のプロジェクト単位の除外設定が適用されません。
- **開発時の SSR HTML に含まれるアセットのリンク**: `dev.stylesheetLinks` は、`%40fs/` やエンコードされた `assets/` パスのような、URL エンコードされた正しい Nuxt のアセットリンクを残します。デコードすると null バイトやパストラバーサルを含むリンクは取り除きます。

解決しない場合は、[トラブルシューティング](../guide/troubleshooting.md)を確認するか、[問題を報告](../guide/testing.md)してください。
