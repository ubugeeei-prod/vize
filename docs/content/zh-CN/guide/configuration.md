---
title: 配置
---

<!-- Generated translation; source: guide/configuration.md -->

# 配置

Vize 使用`vize.config.*`来实现共享的 npm 包命令、Vite 插件和 Rust CLI 设置。

## 配置文件

npm 包命令和`@vizejs/vite-plugin`从项目根加载这些文件
优先顺序：

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

Rust CLI按上述顺序读取与命令原生设置相同的配置文件名，例如：
`check`、`lint`、`lsp`和`fmt`。

## TypeScript 配置

```ts
import { defineConfig } from "vize";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  compiler: {
    sourceMap: mode !== "production",
    ssr: isSsrBuild,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: {
    include: [/\.vue$/],
    exclude: [/node_modules/],
    scanPatterns: ["src/**/*.vue"],
    ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
  },
  linter: {
    enabled: command !== "build",
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
  },
  formatter: {
    printWidth: 100,
    singleQuote: false,
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
  musea: {
    include: ["src/**/*.art.vue"],
    basePath: "/__musea__",
  },
}));
```

## Vue类型解析

Vize 不会将 Vue 的类型表面从已发布的 `vize` 包中钉出：`vize check`，语言
服务器和包命令解析`vue`、`@vue/compiler-sfc`及相关环境类型，这些类型来自
分析项目，因此 Vue 3 的补丁、次要和预发布选项仍由该项目控制
而非用于构建Vize的版本。为了获得可预测的结果，声明支持的Vue
用户项目中的版本（非通过 Vize 内部），保持 `vue`、`@vue/compiler-sfc` 和
像Nuxt这样的集成就在那里对齐了，并`vize check`从项目根点或点开始运行
`typeChecker.tsconfig`目标包裹;只用`typeChecker.corsaPath`来选棋子
二进制，绝不覆盖Vue类型的版本。当一个项目支持多个Vue系列时，测试每个范围
因此 Vize 遵循活跃的依赖图，而非硬编码的类型路径。

## 实验平面作品

Monorepos 可以用 `entries` 描述根默认和包范围覆盖。普通对象
配置内部规范化为一个条目，数组导出被`defineConfig`接受
ESLint-flat-config风格的创作。

```ts
export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  entries: [
    {
      name: "web app",
      basePath: "apps/web",
      files: ["src/**/*.vue"],
      typeChecker: {
        tsconfig: "tsconfig.app.json",
      },
    },
    {
      name: "ui package",
      basePath: "packages/ui",
      files: ["src/**/*.vue"],
      formatter: {
        singleQuote: true,
      },
    },
  ],
});
```

## PKL 配置

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
  vapor = false
  customRenderer = false
  templateSyntax = "standard"
}

vite {
  scanPatterns = new Listing {
    "src/**/*.vue"
  }
}

linter {
  preset = "happy-path"
}

typeChecker {
  enabled = true
  strict = true
}

entries = new Listing {
  new ConfigEntry {
    name = "web app"
    basePath = "apps/web"
    files = new Listing { "src/**/*.vue" }
    typeChecker {
      tsconfig = "tsconfig.app.json"
    }
  }
}

lsp {
  lint = true
  typecheck = false
  editor = false
  formatting = false
}
```

## JSON 配置

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "compiler": {
    "sourceMap": true,
    "vapor": false,
    "customRenderer": false,
    "templateSyntax": "standard"
  },
  "vite": {
    "scanPatterns": ["src/**/*.vue"]
  },
  "linter": {
    "preset": "happy-path"
  },
  "typeChecker": {
    "enabled": true,
    "strict": true
  },
  "musea": {
    "include": ["src/**/*.art.vue"],
    "basePath": "/__musea__"
  }
}
```

## 编译器选项

这些选项属于`compiler`。它们是基于模式支持的，并通过`defineConfig`共享;不是
每一次积分都会消耗所有领域。

| 选项                | 价值观                               | 常见用途                                |
| ------------------- | ------------------------------------ | --------------------------------------- | ------------ |
| `sourceMap`         | `boolean`                            | 在 Vite 插件                            | 中启用源映射 |
| `ssr`               | `boolean`                            | 在不依赖Vite的SSR构建标志时编译为SSR    |
| `vapor`             | `boolean`                            | 启用蒸汽模式编译                        |
| `jsxMode`           | `"vdom"`或`"vapor"`                  | `.jsx`/`.tsx`组件的默认输出后端         |
| `customRenderer`    | `boolean`                            | 将小写非HTML标签视为自定义渲染器元素    |
| `customElements`    | `string[]`                           | 作为自定义元素编译的标签模式（TresJS 用 `Tres*`） |
| `templateSyntax`    | `"standard"`、`"strict"`或`"quirks"` | 模板语法选择警告、错误或Vue-quirk处理   |
| `scriptExt`         | `"ts"`或`"js"`                       | 保留TS输出或在npm build命令中下编译为JS |
| `mode`              | `"module"`或`"function"`             | 低级别编译器输出模式                    |
| `prefixIdentifiers` | `boolean`                            | 模板标识符前缀为`_ctx`                  |
| `hoistStatic`       | `boolean`                            | 控制静态节点起重                        |
| `cacheHandlers`     | `boolean`                            | 控制事件处理缓存                        |
| `isTs`              | `boolean`                            | 解析脚本块作为TypeScript                |
| `runtimeModuleName` | `string`                             | 覆盖运行时导入模块                      |
| `runtimeGlobalName` | `string`                             | 覆盖函数/IIFE风格输出的全局运行时       |

对于 Vite 项目，直接插件选项覆盖共享配置：

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      vapor: true,
      sourceMap: true,
      customRenderer: true,
      templateSyntax: "standard",
    }),
  ],
});
```

## 模板语法

`compiler.templateSyntax`默认是`"standard"`。

- `"standard"`接受可恢复的无效语法，发出警告，并重写为有效输出。
- `"strict"` 报告无效语法为编译错误。
- `"quirks"` 保留模板语法兼容性的怪异问题，无需额外警告。

已知的案例有：

- `v-for`带有不匹配边缘括号的别名。Vue 剥离前导`(`或后`)`
  在分`value`、`key`和`index`之前的别名;标准模式与严格模式报告
  这些别名是畸形的，而个性模式则是Vue的。
- 非空的HTML元素，采用自闭语法编写，如`<div />`或`<span />`。
  标准模式会警告并重写它们为空元素、严格模式错误和个性模式保留
  它们作为自闭叶片。

```text
<template>
  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="(item in items">{{ item }}</div>

  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="item) in items">{{ item }}</div>

  <!-- Standard warns and rewrites this as `<div></div>`. Strict errors. Quirk keeps it as a leaf. -->
  <div />
</template>
```

Vue上游实现：

- [`forAliasRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/utils.ts#L571)
- [`parseForExpression`中的`stripParensRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/parser.ts#L493-L530)

关于 HTML 严格模式行为的 invalid 后面，请参见 [Troubleshooting](./troubleshooting.md)
自动关闭标签。

## JSX 和 TSX 输出模式

> 关于完整的创作API、作用域样式、类型检查、编辑器支持及限制，请参见
> [JSX 和 TSX 指南](./jsx.md)。本节仅涵盖输出模式配置键。

Vize 将 `.jsx`/`.tsx` Vue 组件编译为 Virtual DOM 或
[蒸汽](https://blog.vuejs.org/posts/vue-vapor)输出。`compiler.jsxMode` 选择**全局
默认**用于未明确选择加入的组件;默认是`"vdom"`。

```ts
// vize.config.ts
import { defineConfig } from "@vizejs/vite-plugin";

export default defineConfig({
  compiler: {
    // Default every .jsx/.tsx component to Vapor output.
    jsxMode: "vapor",
  },
});
```

`jsxMode`独立于`compiler.vapor`：`vapor`切换蒸汽以控制`.vue` SFC，同时`jsxMode`
控制JSX/TSX的默认后端。项目可以将SFC保留在VDOM上，同时默认JSX为
蒸汽，或者反过来。Vite 插件也直接接受 `jsxMode` 作为插件选项，这
覆盖共享配置。

### 每个组件指令

单个组件通过指令序言覆盖默认，镜像`"use strict"`：

```tsx
// Compiled to Vapor regardless of the configured default.
const Fast = () => {
  "use vue:vapor";
  return <div class="fast" />;
};

// Compiled to Virtual DOM regardless of the configured default.
const Classic = () => {
  "use vue:vdom";
  return <div class="classic" />;
};
```

由于每个组件独立路由，**单个模块可以混合两个后端**：

```tsx
// vize.config: { compiler: { jsxMode: "vapor" } }

// No directive -> takes the configured default (Vapor here).
export const Dashboard = () => <main>{/* ... */}</main>;

// Opts back into Virtual DOM just for this component.
export const LegacyWidget = () => {
  "use vue:vdom";
  return <aside>{/* ... */}</aside>;
};
```

### 优先权

组件的输出模式按以下顺序解析：

1. 每个组件的`"use vue:vapor"`/`"use vue:vdom"`指令。
2. `compiler.jsxMode`默认设置（或插件的`jsxMode`选项）。
3. 内置的备选方案，`"vdom"`。

### 诊断

一个以`"use vue:"`开头但未命名已知模式的指令（例如打字错误
`"use vue:vdomx"`）被报告为编译错误，而非无声忽略，且有两个冲突
一个组件中的模式指令（`"use vue:vapor"` 跟随 `"use vue:vdom"`）也是
被诊断出来。像`"use strict"`这样的无关序章则保持原样。

## Vue方言

`dialect` 选择 Vue 方言配置文件用于独立的 HTML 文档（`.html`/`.htm`）：

```json
{
  "dialect": "petite-vue"
}
```

- `"vue"`将独立HTML文档视为普通的Vue从CDN翻译的文档。
- `"petite-vue"` 将独立的 HTML 文档选入
  [小巧的](https://github.com/vuejs/petite-vue)方言（`v-scope`/`v-effect`
  完成和针对小范围的IDE特性）。

当缺少密钥时，会根据文档结构性地检测方言：一个`<script src>`
解析为petite-vue包、`petite-vue`的内联ES导入，或`PetiteVue.createApp`
叫。评论或散文中提到小维特从不切换方言，且单排
组件始终使用标准的Vue方言。

## 静态分析选项

使用`linter`来实现npm绒毛路径：

```ts
export default defineConfig({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
});
```

使用`typeChecker`作为NPM检查路径：

```ts
export default defineConfig({
  typeChecker: {
    enabled: true,
    strict: true,
    checkProps: true,
    checkEmits: true,
    checkTemplateBindings: true,
    // Vue 3 Options API template bindings; default-on (matches vue-tsc).
    optionsApi: true,
  },
});
```

`typeChecker.optionsApi` resolves Vue 3 Options API template bindings
（在普通`<script> export default { ... }`上`data`/`computed`/`methods`/`inject`/`setup`/`props`）。
它以标准配置（不是 `legacy` 功能）发售，默认开启（匹配`vue-tsc`），
并且仅对非 `<script setup>` 组件运行，因此该公共路径保持零成本;场景
`optionsApi: false`选择退出。Legacy Vue 2.7 / Nuxt 2 支持（`typeChecker.legacyVue2`，增加了
Nuxt 2模板全局）是一个独立的`legacy`构建选择加入。

`typeChecker.tsconfig` 和 `typeChecker.corsaPath` 是共享模式的一部分，但
项目支持的Corsa路径如今是Rust CLI的表面。`corsaPath`与`vize check`共享，
类型感知`vize lint`和`vize lsp`（`typeChecker.tsgoPath`是已弃用的别名）;运行时间
栈是 TypeScript 7 native platform package（`typescript` / `@typescript/typescript-*`）
和 Corsa/corsa-bind API 层。除非需要将 Vize 指向特定已安装的 `lib/tsc` 可执行文件，
否则请保持 `corsaPath` 未设置。保留环境声明、生成的自动导入文件、路径别名和Vue
`ComponentCustomProperties`在你的项目`tsconfig.json`中设置声明，并使用包脚本
例如`vize:check:app` `--tsconfig`或`--corsa-path`覆盖。

```json
{
  "typeChecker": {
    "servers": 1
  }
}
```

`typeChecker.servers`为未来的Corsa工人池保留。直接项目会话执行者
目前仅支持`1`;更大的值会很快失效，而不是假装调谐并发。

## 博物馆选项

共享配置目前涵盖画廊文件集和路由：

```ts
export default defineConfig({
  musea: {
    include: ["src/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
    basePath: "/__musea__",
    storybookCompat: false,
    inlineArt: false,
  },
});
```

放弃以演示为重点的选项，如`previewCss`、`previewSetup`、`tokensPath`、`theme`和
`storybookOutDir`直接`vite.config.ts`的`musea()`。
