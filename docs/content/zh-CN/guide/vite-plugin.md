---
title: Vite 插件
---

<!-- Generated translation; source: guide/vite-plugin.md -->

# Vite 插件

> **⚠️ 正在开发中：**Vize正在积极开发中，尚未准备好投入生产使用。在采用非简单项目前，请彻底测试。

> **捆绑状态：**`@vizejs/vite-plugin`目前是最稳定的捆绑器集成。
> rollup/webpack/esbuild用`@vizejs/unplugin`，Rspack用`@vizejs/rspack-plugin`。
> 这些非Vite路径仍然不稳定，应被视为实验性。

`@vizejs/vite-plugin`为 Vite 项目提供原生速度的 Vue SFC 编译。它被设计为**直接替换**的，`@vitejs/plugin-vue`——你现有的Vue组件无需修改即可正常工作。

## 安装

从[Vite+安装指南](https://viteplus.dev/guide/install)中安装一次`vp`，然后添加这些软件包：

```bash
vp install -D @vizejs/vite-plugin
```

只有当你的项目从`"vize"`导入共享配置助手时，才`vize`直接依赖
或者暴露如`vize:lint`和`vize:check`等包脚本。

## 基本用法

```javascript
// vite.config.js
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

就这样。用 `@vizejs/vite-plugin` 替换`@vitejs/plugin-vue`，你的项目通过 Rust 编译。

## TypeScript Vue 导入

把插件包添加到`compilerOptions.types`，这样直接`.vue`导入问题可以通过以下方式解决。
未写本地 `env.d.ts` shim 的 TypeScript：

```json
{
  "compilerOptions": {
    "types": ["vite/client", "@vizejs/vite-plugin"]
  }
}
```

这不需要直接添加`vize`作为项目依赖。

对于 Vite Plus 项目，保留 Vite Plus 客户端类型，并附加插件包：

```json
{
  "compilerOptions": {
    "types": ["vite-plus/client", "@vizejs/vite-plugin"]
  }
}
```

对于大多数项目，保持直接插件选项较小，并在编译器设置中保持稳定
`vize.config.ts`。

## 共享配置

推荐的共享入口是`vize`。两个npm都会读取单个`vize.config.*`文件
打包命令和`@vizejs/vite-plugin`。

```bash
vp install -D vize
```

支持的配置文件：

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

TypeScript 配置：

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

PKL 配置：

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
}

vite {
  scanPatterns = new Listing {
    "src/**/*.vue"
  }
}
```

带模式的JSON配置：

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "vite": {
    "scanPatterns": ["src/**/*.vue"]
  }
}
```

从`@vizejs/vite-plugin`导入`defineConfig`仍然可以实现向后兼容，但`import { defineConfig } from "vize"`是未来的共享路径。

完整的共享配置形状请参见[配置](./configuration.md)。

Vite Plus-first项目还可以在`vite.config.ts`中保持仅启动时的设置：

```ts
import { defineConfig } from "vite-plus";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      config: {
        compiler: {
          sourceMap: true,
          vapor: false,
        },
        vite: {
          scanPatterns: ["src/**/*.vue"],
        },
        musea: {
          include: ["src/**/*.art.vue"],
        },
      },
    }),
  ],
});
```

Vite插件和插件商店在执行Vite Plus时可以使用内联配置。
使用 `vize.config.*` 来处理那些必须由 CLI 和 LSP 命令读取的设置。

## 编译器选项

直接选项被传递到`vize()`覆盖`vize.config.*`。
完整的优先级是直接插件选项，然后是内联`config`，然后是`vize.config.*`，然后
默认值。

```ts
vize({
  vueVersion: 3,
  sourceMap: true,
  ssr: false,
  vapor: false,
  customRenderer: false,
  templateSyntax: "standard",
  scanPatterns: ["src/**/*.vue"],
  ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
});
```

| 选项                   | 设置在哪里                                            | 描述                                                                                               |
| ---------------------- | ----------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `vueVersion`           | `vize({ vueVersion })`                                | 将`0.11`、`1`、`2`或`"legacy"`设置为非侵入式的遗留Vue兼容性模式运行，SFC编译则交由主机编译器完成。 |
| `sourceMap`            | `compiler.sourceMap`或`vize({ sourceMap })`           | 生成源图。默认是开启开发，关闭生产。                                                               |
| `ssr`                  | `compiler.ssr`或`vize({ ssr })`                       | 当Vite的SSR构建旗帜不够时，强制SSR汇编。                                                           |
| `vapor`                | `compiler.vapor`或`vize({ vapor })`                   | 通过Vapor后端编译模板。                                                                            |
| `jsxMode`              | `compiler.jsxMode`或`vize({ jsxMode })`               | `.jsx`/`.tsx`组件的默认输出后端（`"vdom"` / `"vapor"`）。每个组件的 `"use vue:*"` 指令覆盖了它。   |
| `customRenderer`       | `compiler.customRenderer`或`vize({ customRenderer })` | 把小写非HTML标签当作自定义渲染器元素。不匹配 `<TresMesh>` 这类 PascalCase 标签。                   |
| `customElements`       | `compiler.customElements`或`vize({ customElements })` | 作为自定义元素编译的标签模式。TresJS 的 PascalCase 渲染器标签使用 `["Tres*"]`。                    |
| `templateSyntax`       | `compiler.templateSyntax`或`vize({ templateSyntax })` | 选择`"standard"`、`"strict"`或`"quirks"`模板语法处理。                                             |
| `include`              | `vite.include`或`vize({ include })`                   | 插件应该编译的文件。                                                                               |
| `exclude`              | `vite.exclude`或`vize({ exclude })`                   | 这些文件是插件应该忽略的。                                                                         |
| `scanPatterns`         | `vite.scanPatterns`或`vize({ scanPatterns })`         | 用于启动预编译的球状模式。                                                                         |
| `ignorePatterns`       | `vite.ignorePatterns`或`vize({ ignorePatterns })`     | 启动前编译时，球状模式跳过。                                                                       |
| `configMode`           | `vize({ configMode })`                                | 使用`"root"`、`"auto"`或`false`来实现共享配置加载。                                                |
| `configFile`           | `vize({ configFile })`                                | 加载一个特定的配置文件。                                                                           |
| `config`               | `vize({ config })`                                    | Vite Plus 运行时设置的内联共享配置。                                                               |
| `handleNodeModulesVue` | `vize({ handleNodeModulesVue })`                      | 按需编译`.vue`从`node_modules`导入的文件。                                                         |
| `debug`                | `vize({ debug })`                                     | 打印插件调试日志。                                                                                 |

常见食谱：

```ts
// Vapor-oriented build
vize({ vapor: true });

// TresJS PascalCase 渲染器标签
vize({
  customRenderer: true,
  customElements: ["Tres*", "primitive"],
});

// Existing templates that rely on parser edge cases, such as
// v-for alias edge parens or `<div />` as a self-closing leaf
vize({ templateSyntax: "quirks" });

// Monorepo package with explicit scan roots
vize({
  root: import.meta.dirname,
  scanPatterns: ["src/**/*.vue", "examples/**/*.vue"],
});

// Legacy Vue / Nuxt 2 Bridge project with an existing host compiler plugin
vize({ vueVersion: 2 });
```

`vueVersion: 0.11`、`1`、`2`和`"legacy"`是主机-编译器的兼容模式。Vize没有
在这些模式下编译`.vue`文件，不会暴露 Vue 3 `vite:vue` API shim，也不会
注入Vue 3捆绑器的功能标志。保留现有的Vue编译器插件、`vue-loader`或Nuxt 2的插件
自己的编译器配置正常。

## 工作原理

该插件拦截`.vue`文件请求，并通过Vize的Rust原生流水线通过Node.js NAPI绑定进行编译：

1.**预编译**— 在`buildStart`时，插件会发现所有`.vue`文件并使用`compileBatch`批量编译。这会触发基于Rayon的Rust端并行编译，同时处理所有CPU核心上的所有文件。

2.**按需编译**— 开发过程中，如果请求的缓存中不存在的 `.vue` 文件（例如动态导入），则通过 `compileFile` 实时编译。

3.**HMR**— 当`.vue`文件发生变化时，只有该文件会被重新编译。插件检测该更改是否仅样式，并在可能的情况下应用仅样式的HMR更新，避免了整个组件的重新渲染。

4.**CSS 提取**— 在生产版本中，所有来自 Vue 组件的 Scopeed CSS 都会被提取并合并为 `assets/vize-components.css`，消除了每个组件的注入开销。

### 合辑流程

```
.vue file
  → Armature (Parser)          — Tokenizes and parses the SFC structure
  → Croquis (Semantic Analysis) — Analyzes template expressions and bindings
  → Atelier (Compilation)       — Generates optimized JavaScript output
  → Vitrine (NAPI Binding)      — Delivers the result to Node.js
  → Vite module graph            — Served as a virtual module
```

同一语义分析层通过linting和类型检查被重复使用。参见
[静态分析](./static-analysis.md)用于管道的诊断部分。

## 比较

| 特色       | @vitejs/plugin-vue | @vizejs/vite-plugin            |
| ---------- | ------------------ | ------------------------------ |
| 语言       | JavaScript         | 锈蚀（NAPI）                   |
| SFC合辑    | 是的               | 是的                           |
| 模板汇编   | 是的               | 是的                           |
| 脚本设置   | 是的               | 是的                           |
| CSS 范围   | 是的               | 是的                           |
| SSR支持    | 是的               | 是的                           |
| HMR        | 是的               | 是的（仅样式优化）             |
| 批次预编译 | 不                 | 是的（通过人造丝平行）         |
| CSS 提取   | 每个组件           | 合并单文件                     |
| 蒸汽模式   | 实验               | 一等舱（`vize_atelier_vapor`） |

## 高级功能

### 批次预编译

与`@vitejs/plugin-vue`在首次请求时编译每个`.vue`文件不同，Vize在构建开始时通过多线程批处理编译对发现的所有`.vue`文件进行预编译。这意味着：

- **开发服务器启动**— 所有组件在第一个页面加载前已准备好
- **生产构建**— 从一开始就实现最大并行性

### 静态资产重写

插件会自动重写模板中的静态资源URL。例如：

```vue
<template>
  <img src="./logo.png" />
</template>
```

`src`属性被提升到导入语句中，使 Vite 能够通过其资产流水线处理该资产（哈希、优化等）。

### 定义替代

Vite 通常跳过`import.meta.*`替换虚拟模块（前缀为 `\0`）。Vize 的插件手动应用定义替换，确保`import.meta.env.*`值在编译后的 Vue 组件中正确工作。

### 按环境隔离

为了与 Nuxt 兼容，插件会根据 Vite 环境（客户端与服务器/SSR）分离`define`值。这防止客户端环境值泄漏到SSR输出中。

## Nuxt 兼容性

该插件为探测`@vitejs/plugin-vue` API（如 Nuxt）的工具提供了兼容性垫片。这意味着 Vize 无需特殊配置即可使用 Nuxt 内置的 Vue 集成：

```ts
// nuxt.config.ts — using the dedicated Nuxt module
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

详情请参见[Nuxt Integration](../integrations/nuxt.md)。

## 注释

- 该插件需要`@vizejs/native` Node.js NAPI绑定（作为依赖自动安装）
- 蒸汽模式编译可通过`vize_atelier_vapor`（Vue 3.6+）获得
- VDOM编译使用`vize_atelier_dom`
- 插件支持导入所有编译后的CSS模块`virtual:vize-styles`
- `.jsx`/`.tsx` Vue 组件通过同一插件自动编译 — 详见 [JSX & TSX](./jsx.md) 指南
- 关于实验性汇总/webpack / esbuild / Rspack支持，请参见[实验性捆绑器集成](./unplugin.md)
