---
title: WASM绑定
---

<!-- Generated translation; source: guide/wasm.md -->

# WASM绑定

> **⚠️ 正在开发中：**Vize正在积极开发中，尚未准备好投入生产使用。WASM API 可能会在没有预告的情况下发生更改。

`@vizejs/wasm` 提供 WebAssembly 绑定，用于直接在浏览器中运行 Vue 编译器。这使得无需服务器的实时SFC编译、打印和格式化成为可能——非常适合游乐场、文档和教育工具。

WASM 绑定基于与 CLI 和 NAPI 绑定（`vize_vitrine`）相同的 Rust 代码库编译，确保所有平台的编译输出一致。

## 安装

从[Vite+安装指南](https://viteplus.dev/guide/install)中安装一次`vp`，然后添加以下包：

```bash
vp install @vizejs/wasm
```

## API

### 编译器选项兼容性

`CompilerOptions`类型是支持的期权库存，`compile` `compileVapor`，
`parseTemplate`，`compileSfc`。未知对象键在 JavaScript 边界被忽略，
这不是兼容性的承诺。`vueParserQuirks` 作为已废弃的别名
`templateSyntax: "quirks"`;明确的`templateSyntax`总是优先。共享的锈蚀
字段`experimentalServerScript`被保留，直到WASM编译器阶段才会被暴露
实现它。每个门面都忽略了不适用于其编译阶段的支持字段：
`bindingMetadata`只适用于直接模板编译。运行时名称适用于生成的
VDOM模块和SFC客户端输出（VDOM或Vapor）;源映射适用于 VDOM 输出，包括
模板结果由`compileSfc`返回。`outputMode`和`scriptExt`只适用于SFC编译。

### 编译SFC

将 Vue 单文件组件编译成 JavaScript：

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

### 绒毛中队

在SFC上运行Vue专用的绒毛规则：

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

### 赛制 SFC

格式化 Vue SFC：

```javascript
import init, { formatSfc } from "@vizejs/wasm";

await init();

const formatted = formatSfc(source, { printWidth: 80 });

console.log(formatted.code);
```

## 初始化

`init()`函数必须被调用一次，才能使用任何其他 API。它加载并实例化 WebAssembly 模块：

```javascript
import init from "@vizejs/wasm";

// Basic initialization
await init();

// With custom WASM URL (useful for CDN or bundler setups)
await init("https://cdn.example.com/vize_vitrine_bg.wasm");
```

## 使用场景

### 游乐场

构建完全在浏览器中运行的交互式Vue编译游乐场。官方的[Vize游乐场](https://vizejs.dev/play)使用WASM绑定进行实时编译：

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

### 文档

在您的文档中嵌入实时且可编辑的Vue示例：

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

创建交互式编译器探索工具，实时显示编译输出，帮助开发者理解Vue模板的转换过程。

### CI/CD

在没有原生二进制文件的环境中（例如 Cloudflare Workers、Deno Deploy、基于浏览器的 CI）使用WASM 绑定进行轻量级编译。

## 源头建设

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

## 国际化

所有产生诊断（lint、编译错误）的 WASM API 都支持本地化消息：

| 代码 | 语言           |
| ---- | -------------- |
| `en` | 英语（默认）   |
| `ja` | 日语（日本語） |
| `zh` | 中文（中文）   |

将`locale`选项传递给任何产生诊断的API：

```javascript
const result = lintSfc(source, {
  filename: "App.vue",
  locale: "ja", // Lint messages in Japanese
});

console.log(result.diagnostics);
```

## 捆绑大小

WASM 模块包含完整的 Vue 编译器流水线（解析器、语义分析器、代码生成器），并已编译成 WebAssembly。gzip化的捆绑包大小约为**1.5 MB**，适合非关键路径加载（例如，页面交互后加载）。

在生产环境中，考虑对WASM模块进行懒加载：

```javascript
// Lazy-load the compiler only when needed
const compiler = await import("@vizejs/wasm");
await compiler.default(); // init()
const result = compiler.compileSfc(source, opts);
console.log(result.script.code, result.template?.code, result.css);
```
