---
title: 静的解析
---

<!-- Reviewed translation; source: guide/static-analysis.md -->

# 静的解析

Vize の解析結果は、コンパイラー、リンター、型チェッカー、言語サーバー、Musea で共有します。Vue SFC を一度解析して得た意味情報を診断やコード生成に再利用し、それぞれのコマンドで同じ解析を繰り返すことを避けます。

以下の例では、`vize` npm パッケージをインストールし、アプリケーションの `package.json` に登録したスクリプトから実行します。

<span id="パイプライン"></span>

## 解析の流れ

| レイヤー | 役割 | 利用する機能 |
| --- | --- | --- |
| Armature | Vue テンプレートと SFC の構造を字句解析・構文解析する | コンパイラー、リンター、フォーマッター |
| Croquis | スコープ、バインディング、マクロ情報、ファイル間のグラフを構築する | コンパイラー、lint、型情報を使う検査 |
| Patina | Vue、スクリプト、CSS、アクセシビリティ、SSR、Vapor、Musea、型情報の lint ルールを実行する | `vize lint`、エディターの診断、Oxlint 連携 |
| Canon | 仮想 TypeScript を生成し、診断を元の Vue ファイルに対応付ける | `vize check`、エディターの型チェック |
| Maestro | LSP を通じて診断とエディターの機能を提供する | `vize lsp`、VS Code、Zed |

静的解析には lint 以外も含まれます。テンプレートのバインディング、コンパイラーマクロ、コンポーネントのメタデータ、provide/inject の関係、リアクティビティの流れ、仮想 TypeScript、コンポーネントギャラリーの情報も、共通の解析結果を利用します。

実際のルール名、既定の設定、ファイル間診断のコードは[ルール一覧](../rules/index.md)で確認できます。

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

エラーの検出を中心に CI を始める場合は `essential`、既定の推奨ルールを使う場合は `happy-path` を選びます。チームの規約も検査するなら `opinionated`、Nuxt の自動インポートを前提とするなら `nuxt`、明示したルールだけを有効にするなら `incremental` を使います。

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

Croquis は、解析した各 SFC について、リアクティブな値の生成元、`.value` が必要な箇所、リアクティビティが失われる箇所、エフェクト間の関係を、元のソース位置とともに提供します。共通のコンパクトな JSON モデルを、診断、レポート、エディター、Playground の **Reactivity** タブで利用します。

<span id="緑青ルールモデル"></span>

## Patina のルールモデル

Patina は lint のレイヤーです。各ルールは SFC のソース、テンプレートのルートや要素、ディレクティブ、`v-for`、`v-if`、補間などを訪問して検査します。ルール名、カテゴリ、既定の重大度、説明、自動修正の可否をメタデータとして持ちます。プリセットは、一緒に有効にするルールをまとめたものです。

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
| `opinionated` | `happy-path` に、より強い規約、スクリプト、型情報のルールを追加する |
| `nuxt` | Nuxt の自動インポートに合わせて `opinionated` を調整する |
| `incremental` | ルールを個別に導入するため、最初は何も有効にしない |

<span id="移行プラグマとカスタム-ルール"></span>

## 抑制コメントの移行とカスタムルール

Patina は、対応するルール名について `eslint-disable`、`eslint-enable`、`eslint-disable-next-line`、`eslint-disable-line` を受け付けます。`vue/require-v-for-key` などのルールを移行するときに、すべての抑制コメントを先に書き換える必要はありません。

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

Croquis のファイル間解析を、Patina の診断を通じて lint から利用します。解析対象の Vue ファイル全体からモジュール一覧、インポートグラフ、コンポーネントの利用関係、索引を構築するため、明示的に有効にする機能です。

現在の `vize lint --cross-file` は、provide/inject の対応、要素 ID の一意性、リアクティビティ、非同期処理の競合を検査します。`--cross-file-tree` を追加すると、診断に加えて provide/inject のツリーも表示します。

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
```

下位レイヤーのファイル間解析には、現在の CLI で公開する範囲より多くの機能があります。

| ファイル間のオプション | 対象となる診断や情報 |
| --- | --- |
| `provide_inject` | 対応しない inject、未使用の provide、文字列キー、リアクティブでない値の流れ |
| `unique_ids` | 重複する ID や、ループ内で一意でなくなる ID |
| `reactivity_tracking` | props の分割代入、別名参照、コンポーネント間のリアクティビティ消失 |
| `race_conditions` | provide や共有状態を通じて競合する非同期の状態更新 |
| `fallthrough_attrs` | `$attrs`、`inheritAttrs`、複数ルートへの属性の引き継ぎによる問題 |
| `component_emits` | 未宣言・未使用の emit、対応する emit のないリスナー |
| `event_bubbling` | 処理されずにコンポーネントの境界を越えるイベント |
| `server_client_boundary` | SSR とクライアントの境界付近のブラウザー API やハイドレーションの問題 |
| `error_suspense_boundary` | 適切な Suspense やエラー境界のない非同期コンポーネント |
| `circular_dependencies` | 循環インポートや深いインポートの連鎖 |
| `component_resolution` | 未登録・未解決のコンポーネント |
| `props_validation` | 必須 props の欠落や、子コンポーネントに渡す props の型の不一致 |

単一ファイルの lint は既定で高速に保ちます。ファイル間の検査は成熟したものから明示的に公開し、信頼できるプロジェクト情報を CLI、Oxlint 連携、エディター共通の診断に反映する方針です。

## 型チェック

`vize check` は Vue SFC の仮想 TypeScript を生成し、Corsa のプロジェクトセッションで診断を取得します。`.vue`、`.ts`、`.tsx`、`.d.ts` を検査し、診断を元のソースファイルに対応付けます。

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
    "vize:check": "vize check src --strict",
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
