---
title: 設定
description: 既存の Vite 設定と TypeScript プロジェクトで Vue の設定を共有します。
---

<!-- Reviewed translation; source: guide/configuration.md -->

# 設定

Vue ツールの設定は `vite.config.*` の `vue:` に、TypeScript の設定は
`tsconfig.json` にまとめます。CLI、Vite 連携、エディタ拡張は、同じプロジェクト設定を
読みます。Vize 専用の設定ファイルは不要です。

<span id="vite-の設定"></span>
<span id="導入する機能を選ぶ"></span>

実際の設定例は[共有 Vue 設定](./vue-configuration.md)を参照してください。

## 既存の Vite+ 設定を使う

Vize のヘルパーに既存の設定を渡します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

const vpConfig = {
  server: { port: 3000 },
};

const vueConfig = {
  compiler: { sourceMap: true },
  linter: { preset: "essential" },
  formatter: { printWidth: 100 },
};

export default defineConfig(vpConfig).withVue(vueConfig);
```

`withVue` は既存の Vite+ オプション、プラグイン、フックを保ちます。
オブジェクト、設定関数、Promise、async 関数を使えます。
`vue: { ... }` を直接書く方法も同じです。パス、スコープ、設定の合成については
[Vue の設定を共有する](./vue-configuration.md)を参照してください。

<span id="typescript-and-vue-template-checking"></span>


## TypeScript と Vue テンプレートの型チェック

標準の `extends` と `references` を含め、既存の TypeScript プロジェクトを使います。
Vue テンプレートのコンパイラ設定は `vueCompilerOptions` に記述します。

```json
{
  "compilerOptions": {
    "strict": true,
    "noEmit": true,
    "jsx": "preserve"
  },
  "vueCompilerOptions": {
    "strictTemplates": true
  },
  "include": ["src/**/*"]
}
```

`vue-tsc` を使っているプロジェクトも、その TypeScript 設定を保ったまま移行できます。
TypeScript の厳密さは `compilerOptions`、Vue 固有の設定は `vueCompilerOptions` に置きます。
対応するオプションと Vize 側での上書きは[リファレンス](./configuration-reference.md)を
参照してください。

<span id="ルールを1つ変更する"></span>

<span id="lint-rule-options"></span>

### ルールのオプション

ルールを変更するには `vue.linter.rules` を使います。ルールごとのオプションは[ルールのオプション](../rules/options.md)を参照してください。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: { rules: { "no-debugger": "error" } },
}).withVue({
  linter: { rules: { "vue/no-v-html": "error" } },
});
```

## 通常の Vite

既存の Vite 設定を保ち、Vue のコンパイルプラグインを置き換えます。

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  vue: {
    linter: { preset: "essential" },
    formatter: { printWidth: 100 },
  },
});
```

<span id="単独-cli"></span>
<span id="standalone-cli"></span>

## CLI とエディタ

対象パッケージのルートでコマンドを実行します。エディタは選択したワークスペース内で
文書の所属プロジェクトを解決し、Vite と TypeScript の設定を読みます。
モノレポでも各パッケージの `tsconfig.json`、参照、Vite 設定を保ちます。
TypeScript プロジェクトを明示的に選ぶ場合は `--tsconfig` を使ってください。

```bash
vp exec vize check
vp exec vize lint src
vp exec vize fmt --check src
```

Vite 設定内のネイティブの相対パスは、選択された Vite の `root` を基準にします。
相対指定の `root` 自体は設定ファイルのディレクトリから解決します。
CLI に直接渡したファイル名と `--tsconfig` は、コマンド実行時のディレクトリを基準にします。
CLI のフラグと明示的なエディタの機能スイッチは、共有設定を上書きします。

## 廃止された設定を移行する

`vize.config.*` とトップレベルの `vize:` は廃止しました。
ネイティブの設定を `vite.config.*` の `vue:` へ、TypeScript プロジェクトの設定を
`tsconfig.json` へ移してください。移行して動作を確認してから、専用設定ファイルを削除します。
Vite 設定と同じ場所に残った専用ファイルはエラーです。専用設定しかないプロジェクトと
同じ移行エラーになります。`vize init` と `vize setup` は元のファイルを保ち、移行先を案内します。

Vize 独自の `extends` はありません。共有する通常の JavaScript オブジェクトを import し、
`defineConfig(vpConfig).withVue(vueConfig)` に渡してください。
TypeScript 標準の `extends` とプロジェクトの `references` は引き続き使えます。

個別のオプションは[Vue の設定を共有する](./vue-configuration.md)、
[設定リファレンス](./configuration-reference.md)、
[コンパイラのリファレンス](./compiler-configuration-reference.md)を参照してください。

<span id="詳細リファレンス"></span>

追加の機能は[Experimentals](./experimentals.md)を参照してください。

> [!NOTE]
> CLI とエディターは既存の Vite・TypeScript プロジェクト設定を読み込みます。廃止された専用設定は移行してから削除してください。

[共有設定のリファレンス](./configuration-reference.md)に、検索規則、Vite・TypeScript 設定、スコープ別の設定、Vue の型解決、LSP・Musea の設定をまとめています。コンパイラの設定と構文は[コンパイラのリファレンス](./compiler-configuration-reference.md)、実験的な機能は[実験的な機能のガイド](./experimentals.md)を参照してください。
