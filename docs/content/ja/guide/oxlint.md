---
title: Oxlint プラグイン
---

<!-- Reviewed translation; source: guide/oxlint.md -->

# Oxlint プラグイン

`oxlint-plugin-vize` を使うと、Oxlint の JavaScript プラグインとして Vize の Vue lint ルールを実行できます。
Oxlint の JavaScript / TypeScript ルールと Vue の診断を、同じコマンドで検査したい場合に使います。

Vize 単独で lint や型チェックを実行する方法は、[静的解析](./static-analysis.md)を参照してください。

> [!IMPORTANT]
> npm からインストールできますが、統合はまだ初期段階です。元の SFC の位置を表示する処理は改善中です。
> ターミナルで診断を読む場合は、`oxlint-vize -f stylish` を使ってください。

## インストール

[Vite+ インストールガイド](https://viteplus.dev/guide/install) から `vp` を一度インストールし、パッケージを追加します。

```bash
vp install -D oxlint oxlint-plugin-vize
```

`oxlint-plugin-vize` は、オプションの依存関係を通じて、一致する Vize ネイティブバインディングを解決します。
ほとんどのユーザーは、`@vizejs/native` を個別にインストールする必要はありません。

<span id="which-file-to-configure"></span>

## 設定するファイル

| コマンド                | 読み込む設定                           |
| ----------------------- | -------------------------------------- |
| `vp lint`, `vp check`   | `vite.config.ts` の `lint` ブロック    |
| `oxlint`, `oxlint-vize` | `.oxlintrc.json`（または `-c <path>`） |

> [!WARNING]
> Vite+ は `.oxlintrc.json` を読み込みません。`.oxlintrc.json` に `jsPlugins` と `vize/*` のルールを書くと
> 設定できたように見えますが、`vp lint` はこのファイルを無視するため、Oxlint に `vize/*` のルールが渡らず、
> Vize の診断が **0 件**のまま終了コード `0` で終わります。`vp lint --init` も既存の `.oxlintrc.json` を
> 移行しません。新しい `lint` ブロックを書き込み、古いファイルはそのまま残します。

[`vize init`](./init.md) は正しいファイルを選びます。lint コマンドが `vp lint` か `oxlint` かを検出し、
そのコマンドが読み込む設定に書き込みます。両方を使っている場合は両方に書き込みます。lint コマンドが
無視するファイルに書き込むくらいなら、何も書き込みません。

<span id="basic-usage-with-vp-lint"></span>

## `vp lint` での基本的な使い方

> [!WARNING]
> `vp lint` は Oxlint の JS プラグインを直接実行します。Oxlint 1.78 と 1.86 では、
> `<script>` も `<script setup>` もない `.vue` に対して Vize のルールは呼び出されません。
> `lint` ブロックでルールを有効にしても変わらず、同じ lint 経路を使う `vp check` にもこの制約があります。
> テンプレートのみの SFC には、以下のネイティブ Vize タスクか `oxlint-vize` を使ってください。

`createVizeLintConfig()` は Vite+ の `lint` ブロックを丸ごと返すため、プラグインを読み込む `jsPlugins` の
指定が抜けることはありません。デフォルトのプリセットは `"happy-path"`（別名 `"general-recommended"`）です。
スタイルやフレームワークの選択に踏み込まない、安全な Vue の基本セットが必要な場合に使います。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { createVizeLintConfig } from "oxlint-plugin-vize";
export default defineConfig({
  lint: createVizeLintConfig({
    preset: "happy-path",
    rules: {
      "no-console": "warn",
    },
    settings: {
      helpLevel: "short",
    },
  }),
}).withVue({});
```

1 つのプリセットを選んだ場合、`preset` は出力するルール設定と `settings.vize.preset` の両方を決めます。
有効なプリセットに含まれない `vize/*` ルールは実行時に黙って除外されるため、この 2 つを一致させることが
重要です。プリセットが合わないルールを指定すると、何も報告されません。`createVizeLintConfig` は、そのような
設定や未知の `vize/*` ルールに対してエラーを投げ、有効に見えて何も報告しない設定を防ぎます。

- `preset: "incremental"` は、指定したルールだけを実行します。
- `preset: ["happy-path", "ecosystem"]` または `presets: ["happy-path", "ecosystem"]` は、複数のプリセットを
  合わせて使います。この場合、ヘルパーは `settings.vize.preset: "incremental"` を出力し、あるプリセットの
  ルールが別のプリセットによって抑制されないようにします。
- `preset: "all"` は、すべてのプリセットを同時に実行します。
- `plugins` には、使い続ける Oxlint の組み込みプラグインを指定します。`vue` と合成され、置き換えられることは
  ありません。一覧を狭めると、それらのプラグインの診断がすべて黙って消えるためです。`create-vue` の
  プロジェクトでは `["eslint", "typescript", "unicorn", "oxc"]` を渡します。
- 既存の `lint` ブロックに組み込む場合は、結果を展開します（`{ ...createVizeLintConfig(), ignorePatterns: ["dist/**"] }`）。

`vite.config.ts` で Flat Config のように設定を組み合わせる場合は、展開できる断片を使い、Vite+ のオブジェクト形式の
`lint` 設定にまとめ直します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { defineVizeLintConfig, flatConfigs } from "oxlint-plugin-vize";
export default defineConfig({
  lint: defineVizeLintConfig(...flatConfigs.recommended, ...flatConfigs.ecosystem, {
    ignorePatterns: ["dist/**"],
    rules: {
      "no-console": "warn",
    },
  }),
}).withVue({});
```

断片には `flatConfigs.recommended`、`flatConfigs.happyPath`、`flatConfigs.essential`、`flatConfigs.ecosystem`、
`flatConfigs.nuxt`、`flatConfigs.opinionated`、`flatConfigs.all` と、`configs` と同じ `*WithTypeAware` の
バリエーションがあります。

テンプレートのみの SFC を含めてネイティブに Vue を lint するには、`vize` と `@vizejs/vite-plugin` をインストールし、
Vite+ の設定を使います。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({ linter: { preset: "happy-path" } });
```

生成されたタスクは `vp run lint` で実行します。既存の package script が `lint` を使う場合は
`vp run vize:lint` になり、明示したタスク名が優先されます。ルール設定は[ルール一覧](../rules/index.md)を参照してください。
`vp lint` を直接呼ぶと、引き続き Oxlint の JS プラグイン経路が選ばれます。

<span id="basic-usage-with-oxlint-and-oxlint-vize"></span>

## `oxlint` と `oxlint-vize` での基本的な使い方

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "helpLevel": "short"
    }
  },
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn",
    "no-console": "warn"
  }
}
```

Oxlint の設定を JavaScript や TypeScript で書く場合は、パッケージが提供するプリセットのルール設定を使えます。

```js
import { configs } from "oxlint-plugin-vize";

export default {
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      helpLevel: "short",
      preset: "opinionated",
      typeAware: true,
    },
  },
  rules: configs.opinionatedWithTypeAware,
};
```

次のプリセットを利用できます。

- `configs.recommended`
- `configs.happyPath`
- `configs.essential`
- `configs.opinionated`
- `configs.nuxt`
- `configs.all`
- `configs.recommendedWithTypeAware`
- `configs.happyPathWithTypeAware`
- `configs.ecosystemWithTypeAware`
- `configs.opinionatedWithTypeAware`

## 推奨されるコマンド

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

`oxlint-vize` は、スクリプトを持たない `.vue` ファイルの扱いを補う `oxlint` の薄いラッパーです。
Oxlint 本体の JavaScript プラグインの対応範囲も引き続き改善されています。

## 設定

設定は `settings.vize` を通じて渡します。

```json
{
  "settings": {
    "vize": {
      "locale": "ja",
      "preset": "general-recommended",
      "helpLevel": "short",
      "typeAware": true
    }
  }
}
```

- `locale` は診断の言語を指定します。
- `preset` には `"general-recommended"`/`"happy-path"`、`"essential"`、`"ecosystem"`、`"incremental"`、
  `"opinionated"`、`"nuxt"`、`"all"` を指定できます。
- 実行時の `preset` がない場合、明示的に設定したルールだけを `"incremental"` として実行します。
  Oxlint が `extends` を通じて設定を引き継がない場合も同じです。
  設定ヘルパーのデフォルトは引き続き `"general-recommended"` です。
- `incremental` は、明示的に設定したルールだけを実行します。
- `rules` には、ルール名（`vize/` の接頭辞の有無は問いません）または Oxlint のルール設定を指定できます。
  `vize/` のキーを持つ設定はそのエントリーだけを選びます。ネイティブの設定では `vue/`、`script/`、`css/`、
  `style/`、`type/`、`nuxt/`、`ecosystem/` の名前を使えます。一致するルールは、ファイルごとに 1 回の
  ネイティブ lint 呼び出しにまとめて実行します。どの診断をどの重大度で報告するかは、Oxlint の最上位の
  `rules` が引き続き決めます。
- `all` は設定上 `incremental` の別名として扱います。すべてのルールも出力したい場合は、`configs.all` または
  `createVizeLintConfig({ preset: "all" })` を使ってください。
- `helpLevel` には `"full"`、`"short"`、`"none"` を指定できます。
- `typeAware: true` は、TypeScript のネイティブコンパイラー（Corsa）を使う `vize/type/*` ルールを有効にします。
- `corsaPath` は、型情報を使う lint で使用するネイティブ TypeScript の実行ファイルを指定します。
- `showHelp` と `settings.patina` は、後方互換性のために引き続き受け付けます。

一部のルールだけを使う場合は、ルール設定をネイティブの一括実行と共有して、ルールのオプションを保ちます。

```ts
const vueRules = {
  "vize/vue/require-v-for-key": "error",
  "vize/vue/no-v-html": "warn",
  "vize/vue/attribute-hyphenation": ["error", "always"],
};

export default {
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: { preset: "incremental", helpLevel: "none", rules: vueRules },
  },
  rules: vueRules,
};
```

`createVizeLintConfig` と `defineVizeLintConfig` は、最終的な Vize のルール設定（ルールのオプションと無効化した
エントリーを含む）から、この選択を自動で生成します。名前だけを列挙した場合は、各ルールをデフォルトの
オプションでまとめて実行し、異なるオプションを設定したルールは別に実行します。ルールを追加したり
オプションを変えたりするファイル単位の上書きも別に実行するため、一括実行によって診断が隠れることは
ありません。ルールの無効化や重大度の変更だけを行う上書きは、引き続き Oxlint の報告方針に従います。

## 現在の制限事項

- Oxlint 1.78 と 1.86 の直接の `oxlint`、`vp lint`、対応する `vp check` の lint 経路では、
  スクリプトのない `.vue` では Vize のコールバックが呼び出されません。テンプレートのみの SFC には、
  ネイティブの `vp run lint` タスクか `oxlint-vize` を使ってください。
- Oxlint JS プラグインの診断範囲は、抽出されたスクリプトに限られるため、テンプレートとスタイルの
  診断では、出力形式によって元の SFC の位置が失われる場合があります。
- `stylish` は、Oxlint と Vize の診断をターミナルで読むための推奨形式です。JSON などの機械可読形式では、
  元のテンプレートやスタイルの位置情報に制約があります。
- 型情報を使うルールは実験段階です。SFC 全体を対象にこれらのルールを実行するには、
  `*WithTypeAware` プリセットと `settings.vize.typeAware: true` を設定します。

<span id="地域開発"></span>

## ローカルでの開発

```bash
nix develop
vp install --frozen-lockfile
vp run --filter './npm/native' build
vp run --filter './npm/oxlint' build
```
