---
title: WASM バインディング
---

<!-- Reviewed translation; source: guide/wasm.md -->

# WASM バインディング

> **⚠️ 開発中:** Vize は現在開発中で、本番利用に向けた準備はまだ完了していません。WASM API は予告なく変更される場合があります。

`@vizejs/wasm` は、ブラウザーで Vue コンパイラを実行する WebAssembly バインディングです。
サーバーを使わずに SFC のコンパイル、lint、フォーマットをリアルタイムで実行でき、プレイグラウンド、ドキュメント、学習ツールに利用できます。

WASM バインディングは、CLI と NAPI バインディング（`vize_vitrine`）と同じ Rust コードからビルドされます。
そのため、プラットフォームが異なっても同じコンパイル結果を得られます。

## インストール

[Vite+ のインストールガイド](https://viteplus.dev/guide/install)に従って `vp` を一度インストールし、パッケージを追加します。

```bash
vp install @vizejs/wasm
```

## API

### コンパイラオプションの互換性

`CompilerOptions` 型は、`compile`、`compileVapor`、`parseTemplate`、`compileSfc` が対応する設定項目の一覧です。
未知のオブジェクトキーは JavaScript からの入力時に無視され、互換性の保証には含まれません。
`vueParserQuirks` は `templateSyntax: "quirks"` の非推奨の別名として残っています。
`templateSyntax` を明示した場合は、常にそちらが優先されます。
共有の Rust 型にある `experimentalServerScript` は予約済みの項目で、WASM のコンパイラ段階で実装されるまでは公開しません。

各 API は、対応する設定項目であっても、自身のコンパイラ段階に関係しないものを無視します。
`bindingMetadata` が適用されるのは、テンプレートの直接コンパイルだけです。
ランタイム名は、生成する VDOM モジュールと SFC のクライアント出力（VDOM または Vapor）に適用されます。
ソースマップは、`compileSfc` が返すテンプレートの結果を含め、VDOM 出力に適用されます。
`outputMode` と `scriptExt` は、SFC のコンパイルにだけ適用されます。

### SFC をコンパイルする

Vue の単一ファイルコンポーネントを JavaScript にコンパイルします。

```javascript
import init, { compileSfc } from "@vizejs/wasm";

await init();

const result = compileSfc(
  `<template>
    <div>{{ msg }}</div>
  </template>

  <script setup lang="ts">
  const msg = ref('Hello Vize!')
  </script>`,
  { filename: "App.vue" },
);

console.log(result.script.code); // compiled <script> / <script setup>
console.log(result.template?.code); // compiled render function, when a template exists
console.log(result.css); // compiled styles, when styles exist
```

<span id="リント-sfc"></span>

### SFC の lint

SFC に対して Vue 固有の lint ルールを実行します。

```javascript
import init, { lintSfc } from "@vizejs/wasm";

await init();

const result = lintSfc(source, {
  filename: "App.vue",
  locale: "en", // 'en' | 'ja' | 'zh'
});

for (const diagnostic of result.diagnostics) {
  console.log(
    `${diagnostic.severity}: ${diagnostic.message} (line ${diagnostic.location.start.line})`,
  );
}
```

### SFC のフォーマット

Vue SFC をフォーマットします。

```javascript
import init, { formatSfc } from "@vizejs/wasm";

await init();

const formatted = formatSfc(source, { printWidth: 80 });

console.log(formatted.code);
```

## 初期化

他の API を使う前に、`init()` を一度呼び出してください。
WebAssembly モジュールを読み込み、インスタンスを生成します。

```javascript
import init from "@vizejs/wasm";

// Basic initialization
await init();

// With custom WASM URL (useful for CDN or bundler setups)
await init("https://cdn.example.com/vize_vitrine_bg.wasm");
```

## 使用例

<span id="遊び場"></span>

### プレイグラウンド

ブラウザー内で動く、インタラクティブな Vue コンパイルのプレイグラウンドを作れます。
公式の [Vize Playground](https://vizejs.dev/play) も、リアルタイムのコンパイルに WASM バインディングを使っています。

```javascript
// React to editor changes and compile in real-time
editor.onChange((source) => {
  const result = compileSfc(source, {
    filename: "Playground.vue",
  });

  if (result.errors.length === 0) {
    preview.update({
      script: result.script.code,
      template: result.template?.code,
      css: result.css,
    });
  } else {
    diagnostics.show(result.errors);
  }
});
```

### ドキュメント

その場で編集・実行できる Vue の例を、ドキュメントに埋め込めます。

```javascript
// Compile documentation examples on the fly
const examples = document.querySelectorAll("[data-vue-example]");
for (const el of examples) {
  const result = compileSfc(el.textContent, {
    filename: `example-${el.id}.vue`,
  });
  // Use result.script.code, result.template?.code, and result.css to mount it.
}
```

### 教育

コンパイル結果をリアルタイムで表示する、インタラクティブなコンパイラの学習ツールを作れます。
Vue のテンプレートがどのように変換されるかを理解する助けになります。

### CI/CD

Cloudflare Workers、Deno Deploy、ブラウザー上の CI など、ネイティブバイナリを利用できない環境でも、
WASM バインディングで軽量なコンパイルを実行できます。

## ソースからのビルド

```bash
# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli

# Build WASM
cargo build --release -p vize_vitrine \
  --no-default-features \
  --features wasm \
  --target wasm32-unknown-unknown

# Generate JS bindings
wasm-bindgen \
  target/wasm32-unknown-unknown/release/vize_vitrine.wasm \
  --out-dir npm/wasm \
  --target web
```

## 国際化

診断を生成する WASM API（lint とコンパイルエラー）は、メッセージの言語を選択できます。

| コード | 言語           |
| ------ | -------------- |
| `en`   | 英語（既定値） |
| `ja`   | 日本語         |
| `zh`   | 中国語（中文） |

診断を生成する API に、`locale` オプションを渡します。

```javascript
const result = lintSfc(source, {
  filename: "App.vue",
  locale: "ja", // Lint messages in Japanese
});

console.log(result.diagnostics);
```

<span id="バンドルのサイズ"></span>

## バンドルサイズ

WASM モジュールには、Vue コンパイラのパーサー、意味解析、コード生成を含むパイプライン全体が入っています。
gzip 圧縮後のサイズは約 **1.5 MB** です。ページの操作が可能になった後など、初期表示を妨げないタイミングで読み込む使い方に適しています。

本番環境で利用する場合は、WASM モジュールの遅延読み込みを検討してください。

```javascript
// Lazy-load the compiler only when needed
const compiler = await import("@vizejs/wasm");
await compiler.default(); // init()
const result = compiler.compileSfc(source, opts);
console.log(result.script.code, result.template?.code, result.css);
```
