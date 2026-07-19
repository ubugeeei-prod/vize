---
title: 入门指南
---

<!-- Generated translation; source: getting-started.md -->

# 开始

> **⚠️ 正在开发中：**Vize正在积极开发中，尚未准备好投入生产使用。API 和包边界可能会在没有预告的情况下发生变化。

## 什么是Vize？

Vize（_/viːz/_）是用Rust编写的Vue.js工具链。工作区包含共享
以下构件：

| 面积          | 主锈箱                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | 面向用户的入口                               |
| ------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| 合辑          | [`vize_atelier_core`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_core)、[`vize_atelier_dom`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_dom)、[`vize_atelier_vapor`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_vapor)、[`vize_atelier_ssr`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_ssr)、[`vize_atelier_sfc`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_sfc) | `@vizejs/vite-plugin`，NPM `vize:build` 脚本 |
| 绒毛          | [`vize_patina`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_patina)                                                                                                                                                                                                                                                                                                                                                                                                             | NPM `vize:lint`脚本，`oxlint-plugin-vize`    |
| 格式          | [`vize_glyph`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_glyph)                                                                                                                                                                                                                                                                                                                                                                                                               | NPM `vize:fmt`脚本                           |
| 类型检查      | [`vize_canon`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_canon)                                                                                                                                                                                                                                                                                                                                                                                                               | NPM `vize:check`脚本                         |
| 编辑支持      | [`vize_maestro`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_maestro)                                                                                                                                                                                                                                                                                                                                                                                                           | VS Code、Zed、Rust `vize lsp`                |
| Musea艺术工具 | [`vize_musea`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_musea)                                                                                                                                                                                                                                                                                                                                                                                                               | `@vizejs/vite-plugin-musea`                  |
| 装订          | [`vize_vitrine`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_vitrine)                                                                                                                                                                                                                                                                                                                                                                                                           | `@vizejs/native`，`@vizejs/wasm`             |

本指南推荐使用 [Vite+](https://viteplus.dev/)（`vp`）来管理 JavaScript 包和项目命令。它保持了安装和执行流程在不同包管理器之间的一致性，同时仍使用工作区底层工具。

如果你还没有`vp`，先安装一次并重新开启一个外壳：

```bash
curl -fsSL https://vite.plus | bash
```

详情请参见[Vite+文档](https://viteplus.dev/)和[安装依赖指南](https://viteplus.dev/guide/install)。

## 维兹的行为

从大层面来看，Vice被划分为几个可重复使用的通道：

| 莱恩     | 包或脚本                                 | 你得到了什么                                                      |
| -------- | ---------------------------------------- | ----------------------------------------------------------------- |
| 编译     | `@vizejs/vite-plugin`，`vize:build`      | Rust原生的Vue SFC编译，SSR输出，Vapor模式，Scoped CSS处理         |
| 静态分析 | `vize:lint`，`oxlint-plugin-vize`        | Vue模板、脚本、CSS、a11y、SSR、Vapor、Musea、跨文件和类型感知诊断 |
| 类型检查 | `vize:check`                             | 虚拟TypeScript生成、项目诊断、Vue到源的诊断映射                   |
| 格式     | `vize:fmt`                               | Vue SFC 格式化及项目和 CLI 选项                                   |
| 组件画廊 | `@vizejs/vite-plugin-musea`，`musea-vrt` | 艺术文件、组件变体、预览设置、设计标记、a11y、VRT                 |
| 编辑支持 | VS Code、Zed、Rust `vize lsp`            | 选择加入的诊断和编辑器功能                                        |

关于lint和类型检查模型，请参见[静态分析](./guide/static-analysis.md)
[规则](./rules/index.md) 用于具体规则输出，且
[配置](./guide/configuration.md)用于共享配置和编译器选项。

在JSX/TSX中编写组件而不是`.vue` SFC吗？请参阅[JSX与多伦多](./guide/jsx.md)指南——
`.jsx`/`.tsx` Vue组件通过同一条Rust通道编译。

## 选择你的入口

### 1.Vite项目

如果你想在现有的 Vite 项目中实现原生 Vue 编译，可以使用 Vite 插件。

```bash
vp install -D @vizejs/vite-plugin
```

只有当你想导入共享配置助手时，才会直接安装`vize`依赖
`"vize"`或添加 Vize 的包脚本，如 `vize:lint` 和 `vize:check`。

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

当你想打包使用相同的设置时，可以在`vize.config.ts`添加编译器选项
脚本和插件：

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

### 2.Nuxt 项目

当你想让 Vize 在 Nuxt 自己的 Vite 流水线内运行时，可以使用 Nuxt 模块。

```bash
vp install @vizejs/nuxt
```

将模块添加到`nuxt.config.ts`：

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

像平时一样运行你的Nuxt开发服务器。该模块为 Vue SFC 注册 `@vizejs/vite-plugin`
编译过程中保留了 Nuxt 自动导入、组件、中间件和 SSR 转换。

请参阅 [Nuxt 集成](./integrations/nuxt.md)指南，了解 Musea 的设置和 Nuxt 专用笔记。

### 3.npm Package Scripts + Shared Config

当你想获得共享配置工具和原生命令时，可以使用`vize` npm包
项目脚本。

```bash
vp install -D vize
```

推荐的包脚本：

```json
{
  "scripts": {
    "vize:build": "vize build src",
    "vize:fmt": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path src",
    "vize:check": "vize check src",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:fmt
vp run vize:lint
vp run vize:check
vp run vize:build
vp run vize:ready
```

npm包的`vize check`命令使用打包的NAPI检查器，并可以输出Vue组件
与`--declaration --declaration-dir dist/types`的宣言。需要时用Rust CLI吧
`check-server`、LSP、IDE管理，或跨Vue、TS、TSX和`.d.ts`输入的项目诊断。

### 4.完整的Rust CLI系统

大多数应用工作流程应该使用上述的 npm 包脚本。使用 Rust 二进制文件
今天需要完整的原生CLI：LSP、IDE管理、配置文件或`check-server`。对于v1 alpha，
支持的公开渠道包括GitHub发布二进制文件和Nix入口;Rust CLI 则不是
已通过 crates.io 出版。

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

```bash
vize build src/**/*.vue
vize fmt --check src
vize lint --profile src
vize check --profile src
vize ready src
vize lsp
```

## 本地类型检查

`vize check` 由 `vize_canon` 驱动，现在 依赖于 [`corsa-bind`](https://github.com/ubugeeei/corsa-bind) 项目会话来实现原生 TypeScript 诊断。Vize为Vue SFC生成虚拟TypeScript，向Corsa请求项目感知诊断，然后将结果映射到原始的`.vue`、`.ts`、`.tsx`和`.d.ts`文件上。

这条路径仍在成熟中，因此编辑器类型检查目前仍是选择加入的功能。该
运行时栈是`@typescript/native-preview`包，Corsa/corsa-bind 是 API 层 Vize。
与 对话，而 TypeScript 原生预览版安装的可执行文件仍然通常被命名为
`tsgo`。用`typeChecker.corsaPath`，或者运行的包脚本
`vize check --corsa-path /path/to/tsgo`，当你想固定那个运行时。
`typeChecker.tsgoPath`仍是一个已废弃的兼容性别名。

有用的包脚本目标：

```json
{
  "scripts": {
    "vize:check": "vize check",
    "vize:check:app": "vize check --tsconfig tsconfig.app.json",
    "vize:check:virtual-ts": "vize check --show-virtual-ts src/components/App.vue",
    "vize:check:declarations": "vize check --declaration --declaration-dir dist/types"
  }
}
```

```bash
vp run vize:check
vp run vize:check:app
vp run vize:check:virtual-ts
vp run vize:check:declarations
```

## 共享的`vize.config.*`

npm 包命令和`@vizejs/vite-plugin` 共享配置发现：

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

TypeScript 配置：

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
  },
  linter: {
    preset: "opinionated",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    corsaPath: "./node_modules/.bin/tsgo",
  },
  formatter: {
    printWidth: 100,
  },
  musea: {
    include: ["src/**/*.art.vue"],
    basePath: "/__musea__",
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
});
```

PKL 配置：

```pkl
amends "node_modules/vize/pkl/vize.pkl"

linter {
  preset = "opinionated"
}

typeChecker {
  enabled = true
  strict = true
}

lsp {
  lint = true
  typecheck = false
  editor = false
  formatting = false
}
```

带模式的JSON配置：

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "linter": {
    "preset": "opinionated"
  }
}
```

## 包裹

```bash
vp install -D @vizejs/vite-plugin
vp install @vizejs/native
vp install @vizejs/wasm
vp install @vizejs/unplugin
vp install @vizejs/rspack-plugin @rspack/core
vp install @vizejs/nuxt
vp install @vizejs/vite-plugin-musea
vp install @vizejs/musea-mcp-server
vp install -D oxlint oxlint-plugin-vize
```

注释：

- `@vizejs/vite-plugin` 是目前推荐的捆绑器集成。
- `@vizejs/unplugin`和`@vizejs/rspack-plugin`还在实验阶段。
- `@vizejs/native`和`@vizejs/wasm`直接暴露Rust装帧。
- `@vizejs/vite-plugin-musea` 为 Musa 提供画廊和开发服务器的工作流程。

## 博物馆组件画廊

当你想要Vue原生组件示例、文档、令牌、VRT和a11y检查时，可以使用Musea：

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
    }),
  ],
});
```

运行你的Vite开发服务器并打开`/__musea__`。有关艺术文件，请参见[Musea](./guide/musea.md)
预览设置、设计标记、VRT和生成变体。

## Oxlint 整合

在Oxlint内部运行Vize的Vue诊断：

```bash
vp install -D oxlint oxlint-plugin-vize
```

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn"
  },
  "settings": {
    "vize": {
      "preset": "general-recommended",
      "helpLevel": "short"
    }
  }
}
```

对于终端优先使用，优先使用：

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

## 编辑支持

日常编辑Vue时，暂时继续用`vuejs/language-tools`。
Vize编辑器的功能设计支持增量选择。

VS Code 起点：

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

Zed起始点：

```json
{
  "languages": {
    "Vue": {
      "language_servers": ["vize", "..."]
    }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true
      }
    }
  }
}
```

## 地方发展

本地任务保持本地;[CI平权](./contributing.md#common-checks)用`nix develop .#testbox`。

```bash
nix develop
vp install --frozen-lockfile
vp check
vp fmt
vp dev
vp build
```
