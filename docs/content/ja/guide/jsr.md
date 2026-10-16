---
title: JSR から JavaScript API を使う
description: Node・Deno・Bun で公開済みの JSR パッケージを使い、ネイティブ依存の要件を確認する。
---

<!-- Reviewed translation; source: guide/jsr.md -->

# JSR から JavaScript API を使う

[`@vizejs/vize`](https://jsr.io/@vizejs/vize) は Vize の JavaScript API を提供します。各エントリーポイントは、同じ Vize バージョンの npm パッケージを再エクスポートします。コンパイラが使うネイティブバイナリは npm から取得します。

| import | API | npm パッケージ |
| --- | --- | --- |
| `@vizejs/vize` | `defineConfig` と設定の型 | `vize` |
| `@vizejs/vize/config` | 設定の読み込みと正規化 | `vize/config` |
| `@vizejs/vize/native` | コンパイラ・linter・formatter・解析 API | `@vizejs/native` |
| `@vizejs/vize/vite` | Vite プラグインとオプション | `@vizejs/vite-plugin` |

## Node.js と Bun

Node.js 22 または 24 以降のプロジェクトで、JSR パッケージと Vite 連携に必要な peer dependency を導入します。

```bash
npm install vite@8 vue@3
npx --yes jsr add @vizejs/vize@0.457.0
```

Bun の場合:

```bash
bun add vite@8 vue@3
bunx jsr add @vizejs/vize@0.457.0
```

インストールしたパッケージから API を読み込みます。

```ts
import { defineConfig } from "@vizejs/vize";
import { compileSfc } from "@vizejs/vize/native";
import vize from "@vizejs/vize/vite";

const config = defineConfig({ compiler: { vapor: false } });
const compiled = compileSfc("<template><h1>Hello</h1></template>", {
  filename: "Hello.vue",
});
const plugin = vize();
```

この例は公開済みの 0.457.0 を指定しています。依存する Vize の npm パッケージも同じリリースに固定されています。Vue アプリの設定は [Vite 連携](./vite-plugin.md)と[導入ガイド](../getting-started.md)を参照してください。

## Deno 2.9.7

次の確認コマンドは、Ubuntu 24.04 の Linux x64（glibc）と Deno 2.9.7 で検証しています。新しいディレクトリを作り、共通の consumer 設定と[公開 API の小さなサンプル](https://github.com/ubugeeei-prod/vize/blob/main/tools/support/release/jsr/consumer.ts)を取得します。

```bash
mkdir vize-jsr-example
cd vize-jsr-example
curl -fsSL https://raw.githubusercontent.com/ubugeeei-prod/vize/main/tools/support/release/jsr/consumer-deno.json -o deno.json
curl -fsSL https://raw.githubusercontent.com/ubugeeei-prod/vize/main/tools/support/release/jsr/consumer.ts -o consumer.ts
deno run --config deno.json --allow-all consumer.ts jsr:@vizejs/vize@0.457.0
```

成功時の出力:

```text
Verified jsr:@vizejs/vize@0.457.0 (no flags)
```

サンプルは設定・ネイティブ API を読み込み、Vue コンポーネントをコンパイルして結果を確認します。自分の Deno コードでは、`jsr:@vizejs/vize@0.457.0/native` のようにバージョン付きの指定を使えます。

[共通設定](https://github.com/ubugeeei-prod/vize/blob/main/tools/support/release/jsr/consumer-deno.json)は、外部依存に `P1D`（24 時間）の公開後待機期間を維持します。リリース直後に取得できる例外は、明示した 13 個の Vize 所有パッケージだけです。`@vizejs` 名前空間全体への例外ではありません。`nodeModulesDir: "none"` も指定し、検証済みの consumer と同じ解決方法を使います。これにより、`package.json` がある場合に未インストールの手動 `node_modules` を参照する問題を避けます。既存プロジェクトでは、他の設定を残したままこれらの項目を統合してください。

コマンドは検証済み consumer と同じ `--allow-all` を使います。Node-API のネイティブコードを読み込む [FFI 権限](https://docs.deno.com/runtime/fundamentals/ffi/)を含め、すべての権限を許可するため、信頼するビルド依存に対して使ってください。この確認だけで、より小さな権限の組み合わせや、すべてのネイティブ対象での Deno 動作を保証するものではありません。[読み取り専用の Actions 検証](https://github.com/ubugeeei-prod/vize/actions/runs/38904123567)では、空キャッシュから公開済みの 0.457.0 を読み込み、SFC をコンパイルしています。公開直後の外部パッケージが拒否されることは、別の制御ケースで確認しています。

## ランタイムとパッケージの範囲

- ネイティブパッケージの導入には `registry.npmjs.org` への接続と optional dependency が必要です。配布済みバイナリの対象は macOS x64/arm64、Windows x64/arm64、Linux x64/arm64（glibc・musl）です。バイナリの配布対象と、検証済みの Deno・ランタイムの組み合わせは別です。[ランタイムのサポート表](../stability.md)も確認してください。
- `/vite` には、利用するプロジェクトの Vite と Vue が必要です。上の Deno 確認コマンドは Vite プラグインを実行しません。Vite の連携には Node または Bun を使ってください。
- JSR が提供するのは JavaScript API です。`vize` コマンドを使う[単独 CLI](./cli.md)は、npm の [`vize` パッケージ](https://www.npmjs.com/package/vize)から導入します。
- ブラウザーでは npm の [`@vizejs/wasm` パッケージ](./wasm.md)を使います。JSR の native エントリーポイントには、JavaScript・ブラウザー用の代替実装はありません。
