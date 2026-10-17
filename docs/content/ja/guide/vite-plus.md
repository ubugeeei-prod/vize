---
title: Vite+ 統合
description: vite.config.ts に Vue コンパイル・型チェック・lint・フォーマットをまとめる。
---

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

既存の Vue plugin の import と `vue()` を削除します。
[移行ガイド](./migration.md)にそのまま使える差分を記載しています。
CLI、エディタ、統合タスクは `vite.config.*` と `tsconfig.json` を共有します。

## 設定

```ts
import { defineConfig, type VueConfigObject } from "@vizejs/vite-plugin/vite-plus";

const vpConfig = {
  server: { port: 3000 },
  lint: {
    vue: { locale: "ja", helpLevel: "short" },
    rules: { "no-debugger": "error" },
  },
} satisfies VueConfigObject;

export default defineConfig(vpConfig).withVue({
  compiler: { sourceMap: true },
  linter: {
    preset: "essential",
    rules: { "vue/no-v-html": "error" },
  },
  formatter: { printWidth: 100 },
});
```

`.withVue(vueConfig)` は、既存の Vite+ 設定を保ちながら `vue:` にネイティブ設定を追加します。
インラインの `vue: { ... }` と同じ設定です。共有するコンパイラ・lint・フォーマッタの設定は
`vue.compiler`・`vue.linter`・`vue.formatter` に、エディター設定は `vue.languageServer` に置きます。
`lint.vue` は診断言語などのタスク設定、その他の `lint`・`fmt` は Vite+ の Oxlint・Oxfmt 設定です。
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

**組み込みの `vp check`・`vp lint`・`vp fmt` は生成タスクを実行しません。**
Vize を含む検査には上の `vp run` を使います。
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
`vue.compiler` を省略して `compiler: false` を指定すると、既存の Vue compiler plugin を保持します。

## 次の設定

- [移行](./migration.md) — 既存の import・plugin・script から置き換える。
- [設定](./configuration.md) — 必要な設定場所を選ぶ。
- [型宣言・共有設定・editor の詳細（英語）](../../guide/vite-plus.md) — `pack.vue`、fluent API と高度なオプション。
- [エディター設定（英語）](../../guide/vite-plus-editor.md) — エディターと統合設定の役割。
