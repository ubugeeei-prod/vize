---
title: WASM バインディング
---

<!-- Generated translation; source: guide/wasm.md -->

# WASM バインディング

> **⚠️ 進行中の作業:**Vize は積極的に開発中であり、まだ運用環境で使用する準備ができていません。 WASM API は予告なく変更される場合があります。

`@vizejs/wasm` は、ブラウザーで Vue コンパイラーを直接実行するための WebAssembly バインディングを提供します。これにより、サーバーなしでリアルタイムの SFC コンパイル、リンティング、フォーマットが可能になり、遊び場、ドキュメント、教育ツールに最適です。

WASM バインディングは、CLI および NAPI バインディング (`vize_vitrine`) と同じ Rust コードベースからコンパイルされ、すべてのプラットフォームで同一のコンパイル出力が保証されます。

## インストール

[Vite+ インストール ガイド](https://viteplus.dev/guide/install) から `vp` を一度インストールし、パッケージを追加します。

```bash
vp install @vizejs/wasm
```

## API

### コンパイラオプションの互換性

`CompilerOptions` タイプは、`compile`、`compileVapor`、
`parseTemplate`、および `compileSfc`。不明なオブジェクト キーは JavaScript 境界で無視され、
互換性を保証するものではありません。 `vueParserQuirks` は、非推奨のエイリアスとして残ります。
`templateSyntax: "quirks"`;明示的な `templateSyntax` が常に優先されます。共有されたRust
フィールド `experimentalServerScript` は予約されており、WASM コンパイラーの段階まで公開されません。
それを実装します。各ファサードは、コンパイラ ステージに適用されないサポートされているフィールドを無視します。
`bindingMetadata` は、テンプレートの直接コンパイルにのみ適用されます。ランタイム名は生成されたものに適用されます
VDOM モジュールと SFC クライアント出力 (VDOM または Vapor)。ソース マップは、以下を含む VDOM 出力に適用されます。
`compileSfc` によって返されたテンプレートの結果。 `outputMode` および `scriptExt` は SFC コンパイルにのみ適用されます。

### SFC をコンパイルする

Vue の単一ファイル コンポーネントを JavaScript にコンパイルします。

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

### リント SFC

SFC で Vue 固有の lint ルールを実行します。

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

`init()` 関数は、他の API を使用する前に 1 回呼び出す必要があります。 WebAssembly モジュールをロードしてインスタンス化します。

```javascript
import init from "@vizejs/wasm";

// Basic initialization
await init();

// With custom WASM URL (useful for CDN or bundler setups)
await init("https://cdn.example.com/vize_vitrine_bg.wasm");
```

## 使用例

### 遊び場

完全にブラウザ内で実行されるインタラクティブな Vue コンパイル プレイグラウンドを構築します。公式 [Vize Playground](https://vizejs.dev/play) は、リアルタイム コンパイルに WASM バインディングを使用します。

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

ライブで編集可能な Vue の例をドキュメントに埋め込みます。

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

リアルタイムでコンパイル出力を表示するインタラクティブなコンパイラ探索ツールを作成し、開発者が Vue テンプレートがどのように変換されるかを理解できるようにします。

### CI/CD

ネイティブバイナリが利用できない環境（Cloudflare Workers、Deno Deploy、ブラウザベースのCIなど）での軽量コンパイルにはWASMバインディングを使用します。

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

診断 (lint、コンパイル エラー) を生成するすべての WASM API は、ローカライズされたメッセージをサポートしています。

| コード | 言語              |
| ------ | ----------------- |
| `en`   | 英語 (デフォルト) |
| `ja`   | 日本語 (日本語)   |
| `zh`   | 中国語 (中文)     |

診断を生成する API に `locale` オプションを渡します。

```javascript
const result = lintSfc(source, {
  filename: "App.vue",
  locale: "ja", // Lint messages in Japanese
});

console.log(result.diagnostics);
```

## バンドルのサイズ

WASM モジュールには、WebAssembly にコンパイルされた完全な Vue コンパイラ パイプライン (パーサー、セマンティック アナライザー、コード ジェネレーター) が含まれています。 gzip 圧縮されたバンドルのサイズは約**1.5 MB**で、非クリティカル パスの読み込み (ページがインタラクティブになった後に読み込まれるなど) に適しています。

運用環境で使用する場合は、WASM モジュールの遅延ロードを検討してください。

```javascript
// Lazy-load the compiler only when needed
const compiler = await import("@vizejs/wasm");
await compiler.default(); // init()
const result = compiler.compileSfc(source, opts);
console.log(result.script.code, result.template?.code, result.css);
```
