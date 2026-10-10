---
title: 自動インポート
---

<!-- Reviewed translation; source: guide/auto-imports.md; scope: complete document -->

# 自動インポート

自動インポートを使うと、`@vizejs/ui` のコンポーネントや `@vizejs/composable` の関数を、
毎回 import 文を書かずに利用できます。アプリの各所でこれらのライブラリを使う場合に便利です。
利用箇所が少なければ、通常の明示的な import でも構いません。

下の [Nuxt の設定](#nuxt) または [Vite の設定](#vite) を追加し、
テンプレートで `<VzDialogTrigger>` などを使ってみてください。
ソースをコピーしている場合は [ローカルソースの設定](#コピーしたソースを使う) に進んでください。

リゾルバーはビルド時に実行され、`@vizejs/ui/dialog` や `@vizejs/composable/use-toggle` のような
直接のサブパスを選びます。手書きの import と同じようにツリーシェイキングできます。

## Nuxt

`@vizejs/nuxt` の `ui` と `composables` を明示的に有効にします。

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    ui: { prefix: "Vz", exclude: ["tooltip"] },
    composables: true,
  },
});
```

- `ui` は `addComponent` で `@vizejs/ui` のコンポーネント（`<VzDialogTrigger>`、`<VzButton>` など）を登録し、各ファミリーが公開する `use*` 関数（`useFieldWiring`、`useSafeAreaInsets` など）も自動インポートします。`prefix`、ファミリー単位の `include` / `exclude`、関数の登録を省く `composables: false`、[ローカルソースのオプション](#コピーしたソースを使う) を指定できます。
- `composables` は `addImports` で `@vizejs/composable` の実行時エクスポートを登録します。対象は `include` / `exclude` のエントリー名（例: `"use-toggle"`）や `names` で絞れ、ローカルソースのオプションも使えます。両方が同じ名前を提供する場合は `@vizejs/ui` の登録が優先されます。

Nuxt は型付きの `components.d.ts` と `imports.d.ts` を生成するため、テンプレートとスクリプトで型情報を利用できます。登録対象はプロジェクトからパッケージを解決して決まるので、インストールしたバージョンに従います。

## Vite

`unplugin-vue-components` と `unplugin-auto-import` にリゾルバーを渡します。
次の例は、Vue プラグインを設定済みの `vite.config.ts` に追加する部分です。

```ts
import Components from "unplugin-vue-components/vite";
import AutoImport from "unplugin-auto-import/vite";
import { VizeUiResolver, VizeUiComposablesResolver } from "@vizejs/ui/resolver";
import { VizeComposableResolver } from "@vizejs/composable/resolver";

// 既存の plugins 配列に追加
Components({ resolvers: [VizeUiResolver({ prefix: "Vz" })] });
AutoImport({ resolvers: [VizeUiComposablesResolver(), VizeComposableResolver()] });
```

`AutoImport({ imports: [...] })` の形式を使う場合は、`vizeUiImports()` と
`vizeComposableImports()` が `{ "<module>": ["name", ...] }` の設定を返します。
独自の構成では `createVizeUiComponentDeclarations()` と `createVizeComposableDeclarations()` で、通常はプラグインが生成する `GlobalComponents` とグローバルな `.d.ts` 宣言を生成できます。

## コピーしたソースを使う

[`vize lib pull` のソース配布](./lib-pull.md) でコードをアプリ内にコピーした場合は、
`source: "local"` を指定してください。

```ts
VizeUiResolver({ source: "local" }); // ./vize-lib.lock.json を読む
VizeComposableResolver({ source: "local", root: projectRoot, lockfile: "vize-lib.lock.json" });
```

`vize-lib.lock.json` に記録された項目は `<root>/<dir>/<entry>` に解決されます。たとえば `src/components/vize/families/actions/button/button.ts` のような、コピーしたソースを使います。
その他の項目はパッケージにフォールバックします。フォールバックを止めるには `fallback: false` を指定します。
Nuxt の `ui` と `composables` にも同じオプションがあります。

## カタログの更新（開発者向け）

`@vizejs/ui` のリゾルバーマニフェストは、ファミリーカタログから
`pnpm --filter @vizejs/ui generate:resolver` で生成します。カタログ内の項目が欠けるとテストが失敗します。
コンポーザブルのリゾルバーは `COMPOSABLE_CATALOG` を直接参照します。
