---
title: Vite+ 統合
description: vite.config.ts に Vue コンパイル・型チェック・lint・フォーマットをまとめる。
---

<!-- Reviewed translation; source: guide/vite-plus.md; scope: complete document -->

# Vite+ 統合

Vite+ と Vize を1つの `vite.config.ts` で設定します。
プロジェクトの Vite+（0.2.3 以降）を使い、Vize が Vite+ を入れ替えることはありません。

```bash
vp install -D @vizejs/vite-plugin
```

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

既存の Vue プラグインの import と `vue()` を削除します。
[移行ガイド](./migration.md)にそのまま使える差分を記載しています。
CLI、エディタ、統合タスクは `vite.config.*` と `tsconfig.json` を共有します。

## 設定

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { defineVizeLintConfig, flatConfigs } from "oxlint-plugin-vize";

const config = defineConfig({
  server: { port: 3000 },
  lint: {
    rules: { "no-debugger": "error" },
    ignorePatterns: ["dist/**"],
  },
  fmt: { ignorePatterns: ["dist/**"] },
}).withVue({
  lint: defineVizeLintConfig(
    ...flatConfigs.recommended,
    ...flatConfigs.ecosystem,
    {
      ignorePatterns: ["dist/**"],
      rules: { "no-console": "warn" },
    },
  ),
  fmt: { singleQuote: true },
  entries: [
    {
      files: ["src/admin/**/*.vue"],
      linter: { rules: { "vue/no-v-html": "off" } },
    },
  ],
});

export default config;
```

ルートのオブジェクトは Vue 固有の項目を含まない通常の Vite+ 設定として保ちます。
`.withVue()` は Vue の flat lint レイヤーをルート設定へ合成するため、ルートの
`no-debugger` と `flatConfigs.recommended`・`flatConfigs.ecosystem`・Vue 固有の
`no-console` が同時に有効です。`compile`・`typeCheck`・`lint`・`fmt` を使います。
`lint` はホストの Oxlint ブロックで、`settings.vize`・`vize/*` ルール・override は
ネイティブ linter にも反映されます。ファイル単位のネイティブ上書きは
`entries[].linter` に置きます。従来の `compiler`・`typeChecker`・`linter`・`formatter` は
互換 alias として使えますが、新旧の同じ項目を同時には指定できません。
[共有 Vue 設定](./vue-configuration.md)にパスとスコープの扱いをまとめています。

TypeScript の設定は既存の `tsconfig.json` を使います。

```json
{
  "compilerOptions": { "strict": true, "noEmit": true },
  "vueCompilerOptions": { "strictTemplates": true },
  "include": ["src/**/*"]
}
```

### 既存の設定を組み合わせる

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import vpConfig from "./vite.base.ts";
import vueConfig from "./vue.settings.ts";

export default defineConfig(vpConfig).withVue(vueConfig);
```

通常の JavaScript オブジェクトを import して共有してください。オブジェクト、Promise、関数、
async 関数を渡せます。既存のプラグインやフックを保持し、Vue のプラグインを一度だけ組み込みます。
独自の設定 `extends`、専用の `vize.config.*`、トップレベルの `vize:` は廃止しました。
TypeScript 標準の `extends`・`references` は引き続き使えます。

単独の `withVue(vueConfig, vpConfig)` も使えますが、引数の順序が異なるため `defineConfig` の別名ではありません。
`withVize` は従来の config-first helper の互換名です。新しい設定には fluent API を使ってください。

<span id="tasks"></span>

## 実行するタスク

| コマンド | 実行するもの |
| --- | --- |
| `vp dev`, `vp build` | Vize で Vue をコンパイル |
| `vp run check` | 型チェック、lint、フォーマット検査 |
| `vp run typecheck` | Vue の型チェック |
| `vp run lint` | Vize と Oxlint の lint |
| `vp run lint:fix` | 両方の lint 修正 |
| `vp run fmt` | Vue は Vize、その他は Oxfmt で書き込み |
| `vp run fmt:check` | 書き込まずフォーマットを検査 |
| `vp run editor:setup` | 推奨拡張機能と Vue エディター設定 |

`vp run check -- --fix` は lint 修正とフォーマットを適用します。
パスは `vp run lint -- src` のように `--` の後ろへ渡せます。

> [!WARNING]
> 組み込みの `vp check`・`vp lint`・`vp fmt` は生成タスクを実行しません。
> Vize を含む検査には上の `vp run` を使います。

既存の package script は保持されます。`check` がすでにあれば生成名は `vize:check`、
`lint` があれば `vize:lint` です。明示的な `run.tasks` が優先されます。

生成タスクの名前変更や無効化は、第2引数で設定します。

```ts
export default defineConfig({}, { tasks: { check: "verify", preview: false } });
```

`tasks: false` ですべての生成タスクを無効化できます。

<span id="lint-and-formatter-ownership"></span>

## lint とフォーマットの担当範囲

Vize のネイティブ linter は Oxlint と一緒に動作します。`oxlint-plugin-vize` の別登録は不要です。
Vue ルールの重複はデフォルトで調整され、明示的な `lint.rules` や overrides が優先されます。

Vize は Vue ファイル、Oxfmt はその他のファイルをフォーマットします。
`fmt.ignorePatterns` は両方に適用されます。
`vue.linter.enabled: false` でネイティブ lint を無効にできます。
`lint.vue: false`・`fmt.vue: false` を使う場合は、対応する共有ネイティブ設定がツールを明示的に有効化しないようにします。
`vue.compiler` を省略して `compiler: false` を指定すると、既存の Vue コンパイラプラグインを保持します。

<span id="library-declarations-and-maps"></span>

## ライブラリの型宣言とマップ

```ts
export default defineConfig({
  pack: {
    entry: ["src/index.ts"],
    outDir: "dist",
    format: ["esm"],
    vue: {
      dts: true,
      declarationMap: true,
      sourcemap: true,
    },
  },
});
```

Vue ライブラリのバンドルには Vize の Rolldown 連携を使います。`pack.vue` を指定すると、ネイティブの型宣言生成が
デフォルトで有効になります。無効にする場合は `dts: false` を指定します。型宣言はバンドルが成功した後に出力され、
型エラーがあると pack コマンドは失敗します。そのエントリーでは tsdown の型宣言処理の代わりにネイティブの生成を
使うため、Vue SFC も扱えます。

出力先は `declarationDir` で変更でき、指定しない場合は `pack.outDir` を使います。`tsconfig` で TypeScript
プロジェクトを選べます。`declarationMap` は、作成済みの tsconfig を書き換えずにそのプロジェクトのマップ設定を
上書きします。`sourcemap` はコンパイラーとバンドルのマップを制御します。`pack` 配列の各エントリーで `vue` を
個別に設定でき、`pack.vue: false` のエントリーは Vite+ に任せます。

<span id="editor-setup"></span>

## エディターの設定

`vp run editor:setup` を実行すると、Vize と Vite Plus の拡張機能の推奨設定と、Vue 向けのエディター設定を追加します。
コメント、既存の推奨設定、明示的な設定、他の言語に選んだフォーマッターは保持されます。

推奨する役割分担と、エディターとネイティブ設定を共有する方法は[エディター設定](./vite-plus-editor.md)を参照してください。
既存のプロジェクトから置き換える手順は[移行](./migration.md)、設定場所の選び方は[設定](./configuration.md)にまとめています。
