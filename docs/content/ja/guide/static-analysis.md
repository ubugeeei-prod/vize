---
title: 静的解析
---

<!-- Generated translation; source: guide/static-analysis.md -->

# 静的解析

Vize の分析スタックは、コンパイラー、リンター、型チェッカー、エディター サーバー、および Musea によって共有されます。
ツーリング。目標は、Vue SFC を一度解析し、豊富なセマンティック情報を保持し、それを再利用することです。
各コマンドを別個のツールとして扱うのではなく、診断とコード生成に使用します。

以下の例では、`vize` npm パッケージがインストールされ、プロジェクト スクリプトから呼び出されることを前提としています。
はアプリケーションに推奨されるワークフローです。

## パイプライン

| レイヤー     | 何をするのか                                                                                                                       | 使用者                             |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| アーマチュア | Vue テンプレートと SFC 構造をトークン化して解析します                                                                              | コンパイラ、リンター、フォーマッタ |
| クロッキー   | スコープ、バインディングメタデータ、マクロ情報、ファイル間グラフを構築します。コンパイラ、lint、型認識チェック                     |
| 緑青         | Vue、スクリプト、CSS、a11y、SSR、Vapor、Musea、および型認識の lint ルールを実行します。 `vize lint`、エディタ診断、Oxlint ブリッジ |
| キヤノン     | 仮想 TypeScript を生成し、診断を Vue ファイルにマッピングします。 `vize check`、エディターの種類のチェック                         |
| マエストロ   | LSP を通じて診断機能とエディター機能を公開します                                                                                   | `vize lsp`、VS コード、ゼッド      |

これは、静的分析はリンティングだけではないことを意味します。テンプレートバインディング、コンパイラマクロ、コンポーネント
メタデータ、提供/注入関係、反応性フロー、生成された仮想 TypeScript、および
コンポーネント ギャラリーのメタデータはすべて、同じ下位レベルの分析作業に依存します。

具体的なルール名、デフォルト、発行できるファイル間診断コードについては、を参照してください。
[ルール](../rules/index.md)。

## 糸くず

デフォルトのプリセットから始めます。

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

正確性のみの CI には `essential` を使用し、デフォルトの推奨バンドルには `happy-path` を使用します。
より強力な規則が必要な場合は `opinionated`、Nuxt を意識した前提の場合は `nuxt`、
`incremental` 明示的に構成されたルールのみを実行したい場合。

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

基本的な lint パスが安定した後でのみ、ファイル間および型認識チェックをオプトインします。

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

ファイル間のリンティングは、一連のファイル間の提供/注入や反応性フローなどの関係を分析します。
Vue ファイル。 `--strict-reactivity` はネイティブ チェッカーを利用した反応性損失ルールを有効にするため、期待できます。
通常のテンプレートおよびスクリプトの lint ルールよりも遅くなります。

## 反応性オーバーレイ

Croquis は、分析された各 SFC の安定した反応性オーバーレイを公開します: 反応性ソース、`.value`
要件、反応性損失サイト、およびソースマッピングを使用した効果グラフエッジ。同じコンパクトでも
JSON モデルは、診断、レポート、エディター サーフェス、および Playground の**反応**タブにフィードを提供します。

## 緑青ルールモデル

Patina は lint ルール レイヤーです。ルールは、SFC ソース、テンプレート ルート、
テンプレート要素、ディレクティブ、`v-for`、`v-if`、補間。各ルールにはメタデータが含まれます。
ルール名、カテゴリ、デフォルトの重大度、ヘルプ テキスト、修正可能かどうか。プリセットはただ
どのルールを一緒に有効にするかを決定するレジストリ。

| エリア             | ルールの例                                                                                   | 内容                                                                 |
| ------------------ | -------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| Vue の正確性       | `vue/require-v-for-key`、`vue/valid-v-model`、`vue/no-use-v-if-with-v-for`                   | 1 つのコンポーネントにローカルなテンプレート セマンティクス          |
| Vue のセキュリティ | `vue/no-v-html`、`vue/no-unsafe-url`                                                         | XSS が発生しやすい HTML および URL シンク                            |
| Vue の構造         | `vue/sfc-element-order`、`vue/require-scoped-style`、`vue/no-unused-components`              | SFCの形状と部品の使い方とメンテナンス性                              |
| スクリプトの規則   | `script/no-options-api`、`script/no-get-current-instance`、`script/prefer-import-from-vue`   | Vue 構成 API とコンパイラ マクロの規則                               |
| CSS                | `css/no-important`、`css/no-hardcoded-values`、`css/prefer-logical-properties`               | スタイル ブロックとデザイン システムに適した CSS                     |
| アクセシビリティ   | `a11y/img-alt`、`a11y/anchor-has-content`、`a11y/label-has-for`                              | アクセシブルなマークアップとインタラクション パターン                |
| HTML               | `html/deprecated-element`、`html/id-duplication`、`html/no-empty-palpable-content`           | HTML の有効性とセマンティック マークアップ                           |
| SSR                | `ssr/no-browser-globals-in-ssr`、`ssr/no-hydration-mismatch`                                 | サーバー/クライアント レンダリングの危険                             |
| 蒸気               | `vapor/no-vue-lifecycle-events`、`vapor/no-inline-template`、`vapor/require-vapor-attribute` | 蒸気指向のテンプレート制約                                           |
| 美術館             | `musea/require-title`、`musea/valid-variant`、`musea/prefer-design-tokens`                   | コンポーネント ギャラリーとバリアント オーサリング                   |
| タイプ認識分析     | `type/require-typed-props`、`type/require-typed-emits`、`type/no-reactivity-loss`            | セマンティックまたはチェッカーに基づくコンテキストを必要とするルール |

組み込みのプリセットは、段階的な導入をサポートすることを目的としています。

| プリセット    | 形状                                                                                |
| ------------- | ----------------------------------------------------------------------------------- |
| `essential`   | エラーに重点を置いた Vue の正確性、セキュリティ、最小限の HTML チェック             |
| `happy-path`  | 正確性、セキュリティ、a11y、SSR、セマンティック チェックのためのデフォルト バンドル |
| `opinionated` | `happy-path` に加えて、より強力な規則、スクリプト ルール、および型ルール            |
| `nuxt`        | Nuxt の自動インポートの前提に合わせて調整された独自のルール                         |
| `incremental` | ホスト主導のルールごとの導入のための空の出発点                                      |

## 移行プラグマとカスタム ルール

Patina は、ルール名を一致させるための既存の ESLint 無効化プラグマを受け入れます。
`eslint-disable`、`eslint-enable`、`eslint-disable-next-line`、および `eslint-disable-line`。これにより、
プロジェクトは、すべての抑制コメントを書き換えることなく、`vue/require-v-for-key` などのルールを移行します。
前に。

プロジェクトローカル JavaScript ルール モジュールは、まだ安定した Vize ランタイム API ではありません。移行中は、
これらのルールを ESLint または Oxlint で実行し、`vize lint` の横で実行するか、`incremental` プリセットを使用して
すでにポリシーに一致する組み込みの Vize ルールのみを有効にします。 `rules` 構成オブジェクト コントロール
組み込みの Vize ルールの重大度を名前で表示します。

ランタイム環境グローバル (次のような典型的なサイドカー ESLint ルール) を禁止する一般的なケースの場合
`no-access-process`、`no-access-local-storage`、または `no-restricted-globals` 対 `localStorage` /
`sessionStorage`)、オプトインの組み込み `script/no-restricted-globals` ルールを維持する代わりに有効にします。
ESLint はそれら専用にインストールされています。デフォルトの拒否リストは `process`、`localStorage`、および
`sessionStorage`、それぞれの裸のリファレンスについて報告されています。

2 つのスクリプト ルールは、`linter.ruleOptions` (#1891) でプロジェクト ローカル構成も受け入れるため、チーム
`vize lint` を通じて独自のアーキテクチャ規則を強制できます。 `script/no-restricted-globals`
組み込みのデフォルトのリストを**置き換える**`globals` リストを受け取ります。 `script/no-restricted-members`は
設定され、`<object>.<property>` フラグが `members` リストからアクセスされるまでオフ。オプションが入力されています
(`name` / `object` / `property` とオプションの `message`、不明なキーは拒否されます);行方不明の
`message` は一般的な勧告に戻ります。

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

## クロスファイルルール

クロスファイル分析は Croquis 内に存在し、緑青診断を通じてリントにさらされます。それは
モジュール レジストリ、インポート グラフ、コンポーネント使用状況グラフなどを構築するため、オプトインします。
分析されたすべての Vue ファイルのインデックスを作成します。

現在、`vize lint --cross-file` により、マッチングの提供/挿入、一意の要素 ID チェックが可能になります。
反応性の追跡、および非同期競合状態の分析。 `--cross-file-tree` は、
これらの診断の上にツリーを提供/注入します。

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
```

下位レベルのクロスファイル エンジンは、現在の CLI サーフェスよりも広範です。

| ファイル間オプション      | 意図された診断または事実                                                               |
| ------------------------- | -------------------------------------------------------------------------------------- |
| `provide_inject`          | 一致しないインジェクト、未使用のプロバイダー、文字列キーの警告、非リアクティブなフロー |
| `unique_ids`              | 重複 ID と非固有 ID がループ内に導入される                                             |
| `reactivity_tracking`     | プロップの構造破壊、エイリアシング、およびコンポーネント間の反応性の損失               |
| `race_conditions`         | 提供された状態または共有された状態を介して競合できる非同期状態の更新                   |
| `fallthrough_attrs`       | `$attrs`、`inheritAttrs`、およびマルチルートフォールスルーの危険                       |
| `component_emits`         | 宣言されていないエミット、未使用のエミット、プロデューサーのないリスナー               |
| `event_bubbling`          | 処理されずにコンポーネントの境界を飛び越えるイベント                                   |
| `server_client_boundary`  | SSR/クライアント境界付近のブラウザ API の使用とハイドレーションのリスク                |
| `error_suspense_boundary` | 有用なサスペンスまたはエラー境界のない非同期コンポーネント                             |
| `circular_dependencies`   | 輸入サイクルと深い輸入チェーン                                                         |
| `component_resolution`    | 未登録または未解決のコンポーネントの使用法                                             |
| `props_validation`        | 必要なプロパティが欠落しており、子プロパティのタイプが一致しません。                   |

方向性は、デフォルトで単一ファイルのリンティングを高速に保ち、ファイル間のグループを明示的に公開することです。
これらは成熟し、信頼性の高いプロジェクトの事実を、
CLI、Oxlint ブリッジ、およびエディター サーバー。

## 型チェック

`vize check` は Vue SFC 用の仮想 TypeScript を生成し、Corsa プロジェクト セッションに
診断。 `.vue`、`.ts`、`.tsx`、および `.d.ts` 入力をチェックし、診断をマッピングし直します。
オリジナルのソースファイル。

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

パスが指定されていない場合、`vize check` は `tsconfig.json` `files`、`include`、および `exclude` を読み取ります。
プロジェクト構成が利用可能な場合はフィールド。生成されたコードをデバッグする場合は、`--show-virtual-ts` を使用します。
`--profile` `node_modules/.vize` でタイミングと仮想ファイルのアーティファクトが必要な場合。

```bash
vp run vize:check:virtual-ts
vp run vize:check:profile
vp run vize:check:single-server
```

宣言出力は、具体化されたチェッカー プロジェクトから入手できます。

```bash
vp run vize:check:declarations
```

プロジェクト全体のテンプレート値と生成された宣言ファイルは TypeScript を通じて表示される必要があります
プロジェクトの構成。 `tsconfig` に含まれるパスの下にアンビエント宣言を配置して渡します
必要に応じて、そのプロジェクト ファイルをチェッカーに送信します。

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

## npm パッケージ スクリプトと Rust CLI の比較

npm `vize` パッケージはパッケージ スクリプトを対象としており、パッケージ化された NAPI バインディングを使用します。

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

Rust CLI には現在、プロジェクトに裏付けられたより完全な型チェック サーフェイスがあります。

```bash
nix run github:ubugeeei-prod/vize#vize -- check --tsconfig tsconfig.app.json --profile src
vize check --tsconfig tsconfig.app.json --profile src
vize lsp
```

アプリケーションにインストール可能なワークフローが必要な場合は、npm パッケージ スクリプトを使用します。次の場合に Rust CLI を使用します。
`check-server`、LSP、IDE 管理、または Corsa がサポートするプロジェクト診断パスが必要です
Vue および TypeScript ファイル。

## オクスリント

チームがすでに Oxlint を実行していて、Vue 対応の診断を必要とする場合は、`oxlint-plugin-vize` を使用します。
同じコマンド:

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

## 導入パス

1. `vize lint --preset essential src` などの `vize:lint:ci` パッケージ スクリプトを CI に追加します。
2. 正確性診断が正常になったら、`happy-path` または `opinionated` に切り替えます。
3. `vize:check` パッケージ スクリプトをプロジェクト `tsconfig.json` に追加します。
4. 最初にエディターのリンティングを有効にし、CI 出力が安定したら型チェックを有効にします。
5. より深い分析の恩恵を受けるプロジェクトに対して、ファイル間および厳密な反応性チェックを追加します。

単一の品質ゲートの場合、`vize ready src` を実行する `vize:ready` パッケージ スクリプトは `fmt を実行します。

- -write`, `lint`, `check`, and `build` を順番に実行し、最初に失敗したステップで停止します。
