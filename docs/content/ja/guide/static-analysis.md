---
title: 静的解析
---

<!-- Reviewed translation; source: guide/static-analysis.md -->

# 静的解析

lint で Vue の誤りやチームの規約を検査し、型チェックで props、イベント、テンプレートの式の型を検査します。どちらも元のソース上に診断を表示するため、CI とエディターで同じコードを確認できます。

以下の例では、プロジェクトに `vize` npm パッケージをインストールし、`package.json` のスクリプトから実行します。

<span id="パイプライン"></span>

## 検査の選び方

| 検査したい内容 | コマンド・連携 | 設定ガイド |
| --- | --- | --- |
| Vue のルール、アクセシビリティ、スクリプト・CSS の規約 | `vize lint src` | [lint の実行設定](./lint-execution.md) |
| props、イベント、テンプレートの型 | Content Mapper または `vize check` | [型チェック](./type-checking.md) |
| コンポーネント間の関係 | `vize lint --cross-file src` | [コンポーネントツリーと複雑度](./cross-file-complexity.md) |
| 編集中の診断 | Vize の言語サーバー | [エディターの設定](./language-server.md) |
| 既存の Oxlint と同時に Vue を検査 | `oxlint-plugin-vize` | [Oxlint 連携](./oxlint.md) |

各ルールの既定の重大度、プリセット、オプション、例は[ルールリファレンス](../rules/index.md)で確認できます。

<span id="糸くず"></span>

## lint

まずは既定のプリセットで実行します。

```json
{
  "scripts": {
    "vize:lint": "vize lint src"
  }
}
```

```bash
vp run vize:lint
```

エラーの検出を中心に CI を始める場合は `essential`、既定の推奨ルールを使う場合は `happy-path` を選びます。Router、Pinia、Nuxt、i18n のルールも加えるなら `ecosystem` です。チームの規約も検査するなら `opinionated`、Nuxt の自動インポートを前提とするなら `nuxt`、明示したルールだけを有効にするなら `incremental` を使います。

```json
{
  "scripts": {
    "vize:lint:ci": "vize lint --preset essential --max-warnings 0 src",
    "vize:lint:opinionated": "vize lint --preset opinionated --help-level short src",
    "vize:lint:fix": "vize lint --fix src",
    "vize:lint:json": "vize lint --format json src"
  }
}
```

```bash
vp run vize:lint:ci
vp run vize:lint:opinionated
vp run vize:lint:fix
vp run vize:lint:json
```

基本の lint が安定してから、ファイル間の検査や型情報を使う検査を追加します。

```json
{
  "scripts": {
    "vize:lint:cross-file": "vize lint --cross-file src",
    "vize:lint:cross-file-tree": "vize lint --cross-file --cross-file-tree src",
    "vize:lint:strict-reactivity": "vize lint --strict-reactivity src"
  }
}
```

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
vp run vize:lint:strict-reactivity
```

ファイル間の lint は、複数の Vue ファイルにまたがる provide/inject やリアクティビティの流れを調べます。`--strict-reactivity` はネイティブの型チェッカーを使うリアクティビティ消失のルールを有効にするため、通常のテンプレートやスクリプトの lint より時間がかかります。

<span id="反応性オーバーレイ"></span>

## リアクティビティの解析情報

解析器は、各 SFC について、リアクティブな値の生成元、`.value` が必要な箇所、リアクティビティが失われる箇所、エフェクト間の関係を、元のソース位置とともに記録します。同じデータを、診断、レポート、エディター、Playground の **Reactivity** タブで利用します。

<span id="緑青ルールモデル"></span>

<span id="patina-のルールモデル"></span>

## lint ルールとプリセット

各 lint ルールは特定の問題を検査し、重大度、オプション、例を持ちます。プリセットでルールをまとめて選び、個別のルール設定で有効・無効や重大度を変更できます。自動修正に対応したルールは `vize lint --fix src` で適用し、コミット前に差分を確認してください。

| 分野 | ルールの例 | 検査する内容 |
| --- | --- | --- |
| Vue の正確性 | `vue/require-v-for-key`、`vue/valid-v-model`、`vue/no-use-v-if-with-v-for` | コンポーネント内のテンプレートの意味 |
| Vue のセキュリティ | `vue/no-v-html`、`vue/no-unsafe-url` | XSS につながる HTML や URL の使用 |
| Vue の構造 | `vue/sfc-element-order`、`vue/require-scoped-style`、`vue/no-unused-components` | SFC の構造、コンポーネントの利用、保守性 |
| スクリプトの規約 | `script/no-options-api`、`script/no-get-current-instance`、`script/prefer-import-from-vue` | Composition API とコンパイラーマクロの使い方 |
| CSS | `css/no-important`、`css/no-hardcoded-values`、`css/prefer-logical-properties` | style ブロックとデザインシステムに適した CSS |
| アクセシビリティ | `a11y/img-alt`、`a11y/anchor-has-content`、`a11y/label-has-for` | アクセシブルなマークアップと操作 |
| HTML | `html/deprecated-element`、`html/id-duplication`、`html/no-empty-palpable-content` | HTML の妥当性と要素の意味 |
| SSR | `ssr/no-browser-globals-in-ssr`、`ssr/no-hydration-mismatch` | サーバーとクライアントの描画で生じる問題 |
| Vapor | `vapor/no-vue-lifecycle-events`、`vapor/no-inline-template`、`vapor/require-vapor-attribute` | Vapor テンプレートの制約 |
| Musea | `musea/require-title`、`musea/valid-variant`、`musea/prefer-design-tokens` | ギャラリーとバリアントの記述 |
| 型情報を使う解析 | `type/require-typed-props`、`type/require-typed-emits`、`type/no-reactivity-loss` | 意味情報や型チェッカーの結果が必要な検査 |

組み込みのプリセットを使って、段階的に導入できます。

| プリセット | 内容 |
| --- | --- |
| `essential` | エラーを中心に、Vue の正確性、セキュリティ、最小限の HTML を検査する |
| `happy-path` | 正確性、セキュリティ、アクセシビリティ、SSR、意味の検査を含む既定の推奨セット |
| `ecosystem` | `happy-path` に Router、Pinia、Nuxt、i18n のルールを追加する |
| `opinionated` | `happy-path` に、より強い規約、スクリプト、型情報のルールを追加する |
| `nuxt` | Nuxt の自動インポートに合わせて `opinionated` を調整する |
| `incremental` | ルールを個別に導入するため、最初は何も有効にしない |

<span id="移行プラグマとカスタム-ルール"></span>

## 抑制コメントの移行とカスタムルール

Vize は、対応するルール名について `eslint-disable`、`eslint-enable`、`eslint-disable-next-line`、`eslint-disable-line` を受け付けます。`vue/require-v-for-key` などのルールを移行するときに、すべての抑制コメントを先に書き換える必要はありません。

プロジェクト固有の JavaScript ルールを実行する安定した API は、まだありません。移行中は、そのルールを ESLint や Oxlint で `vize lint` と併用するか、`incremental` プリセットで方針に合う組み込みルールだけを有効にします。`rules` で、組み込みルールごとに重大度を指定できます。

実行環境のグローバル変数を禁止する場合は、組み込みの `script/no-restricted-globals` を明示的に有効にできます。`process`、`localStorage`、`sessionStorage` の直接参照を既定で検出します。これらだけを検査するための `no-access-process`、`no-access-local-storage`、`no-restricted-globals` などを置き換えられます。

複数のルールは `linter.ruleOptions` で設定できます。型付きオプションの全体は[ルールオプション](../rules/options.md)を参照してください。以下の例では、`script/no-restricted-globals` の既定の禁止リストを置き換え、`script/no-restricted-members` で指定した `<object>.<property>` へのアクセスを検出します。

```json
{
  "linter": {
    "rules": {
      "script/no-restricted-globals": "error",
      "script/no-restricted-members": "error"
    },
    "ruleOptions": {
      "script/no-restricted-globals": {
        "globals": [
          { "name": "process", "message": "Read env via a typed helper." },
          { "name": "alert" }
        ]
      },
      "script/no-restricted-members": {
        "members": [
          { "object": "window", "property": "localStorage", "message": "Use authStorage." }
        ]
      }
    }
  }
}
```

<span id="クロスファイルルール"></span>

## ファイル間のルール

ファイル間の lint は、単一のコンポーネントだけでは判定できない関係を検査します。既定では無効です。関連する Vue ファイルを同じ実行に含めて有効にしてください。

現在の `vize lint --cross-file` は、provide/inject の対応、要素 ID の一意性、リアクティビティ、非同期処理の競合を検査します。`--cross-file-tree` を追加すると、診断に加えて provide/inject のツリーも表示します。

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
```

レポートの読み方は[コンポーネントツリーと複雑度](./cross-file-complexity.md)、利用可能な検査は[ファイル間ルールの例](../rules/cross-file.md)を参照してください。

## 型チェック

> [!TIP]
> 型チェックは [Content Mapper](./content-mapper.md) を使います。TypeScript 7.1 は `tsc --runExternalCode --noEmit` を実行します。tsrs は同じ `tsconfig` の drop-in で、`tsrs --runExternalCode --noEmit` です。設定と `vue-tsc` からの対応は[型チェック](./type-checking.md)にあります。

```json
{
  "scripts": {
    "typecheck": "tsc --runExternalCode --noEmit",
    "typecheck:tsrs": "tsrs --runExternalCode --noEmit"
  }
}
```

`vize check` はプロジェクトの `.vue`、`.ts`、`.tsx`、`.d.ts` を検査し、元のソース上に診断を表示します。

```json
{
  "scripts": {
    "vize:check": "vize check",
    "vize:check:src": "vize check src",
    "vize:check:app": "vize check --tsconfig tsconfig.app.json",
    "vize:check:json": "vize check --format json --quiet",
    "vize:check:virtual-ts": "vize check --show-virtual-ts src/components/App.vue",
    "vize:check:profile": "vize check --profile src",
    "vize:check:single-server": "vize check --servers 1 src",
    "vize:check:declarations": "vize check --declaration --declaration-dir dist/types"
  }
}
```

```bash
vp run vize:check
vp run vize:check:src
vp run vize:check:app
vp run vize:check:json
```

パスを指定しない場合は、利用できる `tsconfig.json` の `files`、`include`、`exclude` に従って対象を決めます。生成コードを調べるときは `--show-virtual-ts`、実行時間や仮想ファイルの結果を `node_modules/.vize` に残したいときは `--profile` を使います。

```bash
vp run vize:check:virtual-ts
vp run vize:check:profile
vp run vize:check:single-server
```

型チェッカーが構築したプロジェクトから型宣言を出力できます。

```bash
vp run vize:check:declarations
```

プロジェクト全体で使うテンプレートの値や型宣言は、TypeScript のプロジェクト設定から参照できるようにします。`tsconfig` に含まれる場所にアンビエント宣言を置き、必要ならそのプロジェクトファイルを型チェッカーに指定します。

```json
{
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue", "src/**/*.d.ts"]
}
```

```ts
// src/types/vue-app.d.ts
declare module "vue" {
  interface ComponentCustomProperties {
    $t: (key: string) => string;
    $route: { path: string };
  }
}
```

```bash
vp run vize:check:app
```

<span id="npm-パッケージ-スクリプトと-rust-cli-の比較"></span>

## npm のスクリプトと Rust CLI

npm の `vize` は、パッケージに含まれる NAPI バインディングを通じて Rust CLI を呼び出します。アプリケーションのスクリプトに登録して実行できます。

```json
{
  "scripts": {
    "vize:lint": "vize lint src",
    "vize:check": "vize check src",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:lint
vp run vize:check
vp run vize:ready
```

Rust CLI を直接インストールした場合も、同じコマンドを実行できます。

```bash
nix run github:ubugeeei-prod/vize#vize -- check --tsconfig tsconfig.app.json --profile src
vize check --tsconfig tsconfig.app.json --profile src
vize lsp
```

アプリケーションの依存関係と実行環境を管理したい場合は npm のスクリプトを使います。Rust CLI は Nix などで直接インストールする方法もあります。npm の実行時には同梱の TypeScript ランタイムを探し、`CORSA_PATH` などの明示的な指定があればその設定を優先します。

<span id="オクスリント"></span>

## Oxlint

すでに Oxlint を使っているチームは、`oxlint-plugin-vize` で Vue の診断を同じコマンドに追加できます。

```bash
vp install -D oxlint oxlint-plugin-vize
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "preset": "essential",
      "helpLevel": "short"
    }
  },
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn"
  }
}
```

<span id="導入パス"></span>

## 導入の順序

1. `vize lint --preset essential src` などを `vize:lint:ci` に登録し、CI で実行します。
2. エラーが解消してから `happy-path` や `opinionated` に切り替えます。
3. プロジェクトの `tsconfig.json` を使う `vize:check` を登録します。
4. エディターでは先に lint を有効にし、CI の結果が安定してから型チェックを追加します。
5. より深い解析が必要なプロジェクトでは、ファイル間の検査と厳密なリアクティビティの検査を追加します。

`vize:ready` に `vize ready src` を登録すると、`fmt --write`、`lint`、`check`、`build` を順番に実行し、最初に失敗した段階で停止します。
