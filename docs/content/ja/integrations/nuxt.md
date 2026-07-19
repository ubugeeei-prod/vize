---
title: ナクスト
---

<!-- Generated translation; source: integrations/nuxt.md -->

# Nuxt の統合

> **⚠️ 進行中の作業:**Vize は積極的に開発中であり、まだ運用環境で使用する準備ができていません。 Nuxt プロジェクトに採用する前に徹底的にテストしてください。

Vize は、`@vizejs/nuxt` モジュールを通じてファーストクラスの Nuxt 統合を提供します。これにより、Nuxt のデフォルトの Vue コンパイラが Vize の Rust ネイティブ コンパイラに置き換えられ、Nuxt プロジェクトでも同様の速度向上が実現します。

## はじめる

### 1. モジュールをインストールする

[Vite+ インストール ガイド](https://viteplus.dev/guide/install) から `vp` を一度インストールし、モジュールを追加します。

```bash
vp install @vizejs/nuxt
```

pnpm で `pkl` 構成を使用する場合は、`vize` パッケージ自体をインストールする必要がある場合があります。
`@vizejs/nuxt` は、デフォルト設定で `vize.pkl` を提供する `vize` をインストールしますが、pnpm を使用する場合、`vize.pkl` の場所が異なる場合があります。

```bash
vp install vize
```

### 2. Nuxt モジュールを登録する

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

### 3. Nuxtを起動する

通常どおり開発サーバーを起動します。

```bash
vp run dev
```

このモジュールは `@vizejs/vite-plugin` を Nuxt の Vite 構成に挿入し、Nuxt 固有の変換を保持します
パイプライン内にあるため、自動インポート、コンポーネント、ミドルウェア、SSR の動作は引き続き機能します。
ナクスト。
開発中、サーバー応答のクリーンアップにより、次のような有効な URL エンコードされた Nuxt アセット リンクが保存されます。
`%40fs/` およびエンコードされた `assets/` パスとして、デコードされた null バイトまたはトラバーサル パスを削除します。

## モジュールオプション

`@vizejs/nuxt` は単純な `compiler: true | false` スイッチを保持しますが、モジュール オプションも公開します
より厳密な制御が必要なプロジェクト向けの Vize コンパイラーと Nuxt 互換性ブリッジ:

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

| オプション            | タイプ                               | デフォルト                 | 説明                                                                                                                                                                                                                                                         |
| --------------------- | ------------------------------------ | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `compatibility`       | `VizeNuxtCompatibilityOptions`       | 自動検出                   | 異常なラッパーに対して検出された Nuxt/Vue メジャー バージョンをオーバーライドします。 Nuxt 2 はデフォルトで Vue 2 ホストコンパイラー互換性を持っています。 Nuxt 3/4 のデフォルトは Vue 3 です。Vue 0.11/1/2 はすべてホスト コンパイラー モードを使用します。 |
| `compiler`            | `boolean \| VizeNuxtCompilerOptions` | `true`                     | Vize を Vue SFC コンパイラーとして有効にします。オブジェクトを渡すと、`root`、`devUrlBase`、オンデマンド `scanPatterns`、依存関係 SFC 処理の Nuxt デフォルトを維持しながら、オプションが `@vizejs/vite-plugin` に転送されます。                              |
| `bridge`              | `boolean \| VizeNuxtBridgeOptions`   | `true`                     | Vize 仮想モジュール上の自動インポート、コンポーネント インポート、i18n ヘルパー、安定した非同期データ キー用の Nuxt 変換ブリッジを制御します。                                                                                                               |
| `unocss`              | `boolean \| VizeNuxtUnoCssOptions`   | `true`                     | Vize 仮想モジュールの UnoCSS ブリッジを制御します。 `originalSource: false` はソース SFC の読み取りを無効にします。 `maxBytes` はメモリ使用量を制限します。                                                                                                  |
| `dev.stylesheetLinks` | `boolean`                            | `true`                     | Vize で生成された Nuxt アセット URL の開発専用 SSR HTML スタイルシート リンクのクリーンアップを有効にします。                                                                                                                                                |
| `musea`               | `boolean \| MuseaOptions`            | `false`                    | Musea ギャラリーの統合を選択します。 Musea のデフォルトに `true` を使用するか、パターン、トークン、プレビュー CSS、ルーティングなどを構成するオブジェクトを渡します。                                                                                        |
| `nuxtMusea`           | `NuxtMuseaOptions`                   | `{ route: { path: "/" } }` | Musea プレビュー ヘルパーで使用される Nuxt モック シェイプを文書化します。 Nuxt モジュールはモック レイヤーをグローバルにインストールしません。そうすると、Nuxt 自体の `#imports` がシャドウされるためです。                                                 |

## 高度なセットアップ

### Nuxt 2 とレガシー Vue

Nuxt 2 プロジェクトは Vue 2 コンパイラ出力を使用します。 Vize のネイティブ SFC コンパイラは Vue 3 をターゲットとしているため、Nuxt
モジュールは、Nuxt 2 を検出すると、ホスト コンパイラーの置き換えを自動的に回避します。 Nuxt 2 Bridge の場合
または他の Vite ベースの Vue 2 セットアップでは、Vite プラグインは `vueVersion: 2` を受け取ります。
`@vitejs/plugin-vue2`、`vue-loader`、または `.vue` ファイルを担当する Nuxt 独自のコンパイラー。

同じホスト コンパイラー モードは、`vueVersion: 0.11` 経由で古い Vue プロジェクトでも利用できます。
`vueVersion: 1`、または `vueVersion: "legacy"`。

Nuxt Kit からバージョンを隠す方法でプロジェクトが Nuxt をラップしている場合は、互換性を設定してください
明示的にオーバーライドします。

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

### Vite プラグインを直接使用する

あるいは、Vite プラグインを直接使用することもできます。 Nuxt は内部で Vite を使用しているため、これは機能しますが、Nuxt 固有の最適化がいくつか欠けています。

```ts
// nuxt.config.ts
import vize from "@vizejs/vite-plugin";

export default defineNuxtConfig({
  vite: {
    plugins: [vize()],
  },
});
```

## Musea の統合

Nuxt モジュールは Musea (コンポーネント ギャラリー) の統合もサポートしています。

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

構成すると、開発中に `/__musea__/` で Musea ギャラリーが利用可能になります。

### アートファイルの配置

Nuxt コンポーネントの自動検出は、構成されたコンポーネント ディレクトリ内の `.vue` ファイルをスキャンします。なぜなら
Musea アート ファイルも `.vue` で終わります。`*.art.vue` ファイルは Nuxt のこれらのディレクトリの外に保管してください。
プロジェクトを作成し、Musea にその場所を指定します。

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

Musea が `@vizejs/nuxt` を通じて有効になっている場合、モジュールは `**/*.art.vue` も Nuxt のモジュールから除外します。
コンポーネント スキャナーを使用するため、同じ場所に配置されたレガシー ファイルは Nuxt の Webpack または Vite コンポーネント パイプラインに到達しません。

### Nuxt のプレビューセットアップ

Nuxt プロジェクトでは、Musea プレビュー環境で利用できる必要がある機能がよく使用されます。
(`NuxtLink`、`useRoute`、`useNuxtApp`、`useRuntimeConfig`、データ コンポーザブル、および組み込み Nuxt
コンポーネント）。スタンドアロン Musea Vite 構成で `@vizejs/musea-nuxt` を使用し、そのプレビューをインストールします
`previewSetup` のモック レイヤー:

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

## 仕組み

Nuxt モジュールがインストールされている場合:

1.**Vite プラグイン挿入**— モジュールは `@vizejs/vite-plugin` を Vite プラグインとして登録し、`.vue` ファイルのコンパイルをインターセプトします。2.**互換性シム**— プラグインは `@vitejs/plugin-vue` 互換性 API を公開するため、Nuxt の内部チェック (Vue プラグインのプローブ) が正しく機能します。3.**SSR サポート**— Vize の `vize_atelier_ssr` はサーバー側のコンパイルを処理します。このプラグインは、クライアントとサーバーの環境変数を分離して、相互汚染を防ぎます。4.**Nuxt 機能の保持**— 自動インポート、コンポーザブル、ミドルウェア、その他の Nuxt 機能は、Vize のコンパイル後に実行される Nuxt 独自の変換レイヤーを通じて機能します。

## 実際の例

[Vue Fes Japan 2026](https://vuefes.jp/2026) カンファレンス Web サイトでは、Nuxt 4 で Vize を使用しています。

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

この構成では、実稼働ビルド用の Nuxt のデフォルト コンパイラを維持しながら、コンポーネントの開発とドキュメントに Musea を使用します。

## 注意事項

- Vize は現在開発中です - 本番の Nuxt プロジェクトで使用する前に徹底的にテストしてください
- SSR コンパイルは `vize_atelier_ssr` 経由でサポートされます
- Nuxt 固有の機能 (自動インポート、コンポーザブル、ミドルウェア) は、Nuxt 独自の変換レイヤーを通じて機能します
- Nuxt モジュールは Nuxt 2、Nuxt 3、および Nuxt 4 をサポートします。Vize のネイティブ SFC コンパイラーは Vue 3 出力をターゲットとしているため、Nuxt 2 はホスト コンパイラー互換モードを使用します。
