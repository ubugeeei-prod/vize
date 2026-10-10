---
title: Oxlint プラグイン
---

<!-- Reviewed translation; source: guide/oxlint.md -->

# Oxlint プラグイン

`oxlint-plugin-vize` を使うと、Oxlint の JavaScript プラグインとして Vize の Vue ルールを実行できます。
Oxlint の JavaScript / TypeScript ルールと Vue の診断を、同じコマンドで検査したい場合に使います。

Vize 単独で lint や型チェックを実行する方法は、[静的解析](./static-analysis.md)を参照してください。

> [!IMPORTANT]
> npm からインストールできますが、統合はまだ初期段階です。元の SFC の位置を表示する処理は改善中です。
> ターミナルで診断を読む場合は、`oxlint-vize -f stylish` を使ってください。

## インストール

[Vite+ インストール ガイド](https://viteplus.dev/guide/install) から `vp` を一度インストールし、パッケージを追加します。

```bash
vp install -D oxlint oxlint-plugin-vize
```

`oxlint-plugin-vize` は、オプションの依存関係を通じて、一致する Vize ネイティブ バインディングを解決します。
ほとんどのユーザーは、`@vizejs/native` を個別にインストールする必要はありません。

## 基本的な使い方

> [!WARNING]
> `vp lint` は Oxlint の JS プラグインを直接実行します。Oxlint 1.78 と 1.86 では、
> `<script>` も `<script setup>` もない `.vue` に対して Vize のルールは呼び出されません。
> `lint` ブロックでルールを有効にしても変わらず、同じ lint 経路を使う `vp check` にもこの制約があります。
> テンプレートのみの SFC には、以下のネイティブ Vize タスクか `oxlint-vize` を使ってください。

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

JS または TS Oxlint 構成を使用する場合、パッケージはプリセット ルール マップもエクスポートします。

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

利用可能なプリセットのエクスポートには次のものがあります。

- `configs.recommended`
- `configs.essential`
- `configs.opinionated`
- `configs.nuxt`
- `configs.all`
- `configs.recommendedWithTypeAware`
- `configs.ecosystemWithTypeAware`
- `configs.opinionatedWithTypeAware`

## 推奨されるコマンド

Vite+ でテンプレートのみの SFC も検査するには、`vize` と `@vizejs/vite-plugin` を追加し、
ネイティブ Vize タスクを設定します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: { vize: { preset: "happy-path" } },
});
```

生成されたタスクは `vp run lint` で実行します。既存の package script が `lint` を使う場合は
`vp run vize:lint` になり、明示したタスク名が優先されます。ルール設定は [Rules](../rules/index.md) を参照してください。
`vp lint` を直接呼ぶと、引き続き Oxlint の JS プラグイン経路が選ばれます。

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

`oxlint-vize` は、スクリプトを持たない `.vue` ファイルにも対応するための `oxlint` ラッパーです。
Oxlint 本体の JavaScript プラグインの対応範囲も引き続き改善されています。

## 設定

設定は `settings.vize` を通じて渡されます。

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

- `locale` は診断言語を制御します。
- `preset` は、`"general-recommended"`/`"happy-path"`、`"essential"`、`"ecosystem"`、`"incremental"`、`"opinionated"`、または `"nuxt"` を受け入れます。
- `preset` のデフォルトは `"general-recommended"` です。
- `incremental` は、明示的に構成したルールのみを実行します。
- `helpLevel` は、`"full"`、`"short"`、または `"none"` を受け入れます。
- `typeAware: true` は、共有 Patina パス中に Corsa 支援の `vize/type/*` ルールを有効にします。
- `corsaPath` は、型情報を使う lint用に Corsa または `tsgo` 実行可能ファイルを選択します。
- `showHelp` および `settings.patina` は、下位互換性のために引き続き受け入れられます。

## 現在の制限事項

- Oxlint 1.78 と 1.86 の直接の `oxlint`、`vp lint`、対応する `vp check` の lint 経路では、
  scriptless `.vue` の Vize callback が呼び出されません。テンプレートのみの SFC には、
  ネイティブの `vp run lint` タスクか `oxlint-vize` を使ってください。
- Oxlint JS プラグインは抽出されたスクリプト プログラムに範囲を固定するため、テンプレートとスタイル
  診断では、出力形式によって元の SFC の位置が失われる場合があります。
- `stylish` は、Oxlint と Vize の診断をターミナルで読むための推奨形式です。JSON などの機械可読形式では、
  元のテンプレートやスタイルの位置情報に制約があります。
- 型情報を使うルールのエクスポートは実験的なものです。 `*WithTypeAware` 構成を使用して設定します
  `settings.vize.typeAware: true` 共有フルファイル パスでこれらのルールを積極的に実行する場合。

source-built n8n fixture replay は、ライセンス対象の 1,369 SFC、51 Vize ルールと options、
6 ファイルのルール override をネイティブ bridge と `oxlint-vize` で検証しています。19 の scriptless 入力も含みますが、
それらの直接の SDK callback を検証したことにはなりません。n8n 固有の 2 プラグインと workspace 全体の設定は検証対象外です。

## 地域開発

```bash
nix develop
vp install --frozen-lockfile
vp run --filter './npm/native' build
vp run --filter './npm/oxlint' build
```
