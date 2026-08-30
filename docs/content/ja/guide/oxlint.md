---
title: Oxlint プラグイン
---

<!-- Generated translation; source: guide/oxlint.md -->

# Oxlint プラグイン

`oxlint-plugin-vize` を使用すると、Oxlint は Oxlint の JS プラグイン システムを通じて Vize Patina 診断を実行できます。
Oxlint の Rust ネイティブ JS および TS ルールと Vize の Vue 対応ルールが必要な場合に使用します。
診断を 1 回の実行で実行できます。

Oxlint 外部のネイティブ lint および型チェック パイプラインについては、を参照してください。
[静的解析](./static-analysis.md)。

> [!重要]
> パッケージは npm で入手できますが、統合はまだ初期段階です。人間が読める端末の場合
> 出力では、オリジナルの SFC 範囲の忠実度が向上し続ける一方で、`oxlint-vize -f stylish` を優先します。

## インストール

[Vite+ インストール ガイド](https://viteplus.dev/guide/install) から `vp` を一度インストールし、パッケージを追加します。

```bash
vp install -D oxlint oxlint-plugin-vize
```

`oxlint-plugin-vize` は、オプションの依存関係を通じて、一致する Vize ネイティブ バインディングを解決します。
ほとんどのユーザーは、`@vizejs/native` を個別にインストールする必要はありません。

## 基本的な使い方

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

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

`oxlint-vize` は、スクリプトレスの `.vue` エッジ ケースをスムーズにする、`oxlint` の薄いラッパーです。
一方、上流の JS プラグインのカバー範囲は引き続き改善されています。

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
- `preset` は、`"general-recommended"`、`"essential"`、`"ecosystem"`、`"incremental"`、`"opinionated"`、または `"nuxt"` を受け入れます。
- `preset` のデフォルトは `"general-recommended"` です。
- `incremental` は、明示的に構成したルールのみを実行します。
- `helpLevel` は、`"full"`、`"short"`、または `"none"` を受け入れます。
- `typeAware: true` は、共有 Patina パス中に Corsa 支援の `vize/type/*` ルールを有効にします。
- `corsaPath` は、型を認識したリンティング用に Corsa または `tsgo` 実行可能ファイルを選択します。
- `showHelp` および `settings.patina` は、下位互換性のために引き続き受け入れられます。

## 現在の制限事項

- `<script>` または `<script setup>` がない生の `oxlint` では、依然として一部の `.vue` ファイルが欠落する可能性があります。使用する
  プロジェクトにテンプレートのみの SFC が含まれている場合は、`oxlint-vize`。
- Oxlint JS プラグインは抽出されたスクリプト プログラムに範囲を固定するため、テンプレートとスタイル
  診断では、すべてのフォーマッタで元の SFC 範囲がまだ保持されていません。
- `stylish` は、現在、Oxlint と Vize の混合出力に最適な人間が判読できるフォーマッタです。 JSONと
  他の機械可読形式は、元のテンプレート/スタイルのベストエフォートとして扱われる必要があります。
  ポジション。
- タイプ認識ルールのエクスポートは実験的なものです。 `*WithTypeAware` 構成を使用して設定します
  `settings.vize.typeAware: true` 共有フルファイル パスでこれらのルールを積極的に実行する場合。

## 地域開発

```bash
nix develop
vp install --frozen-lockfile
vp run --filter './npm/native' build
vp run --filter './npm/oxlint' build
```
