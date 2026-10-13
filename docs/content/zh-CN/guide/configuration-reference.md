---
title: 共享配置参考
---

<!-- Reviewed translation; source: guide/configuration-reference.md; scope: introduction and discovery -->

# 共享配置参考

优先使用现有的 `vite.config.*` 和 `tsconfig.json`；请参阅[配置指南](./configuration.md)。本页介绍原生设置和可选的专用配置格式。

## 配置文件

最近项目的同目录不存在专用配置时，会发现并读取 Vite 配置。CLI 也支持 `--config`；直接传给插件的选项和编辑器中显式设置的功能具有更高优先级。专用配置中的相对路径以配置文件目录为基准。Vite 中的 `typeChecker` 路径和作用域 `basePath` 以选定的 Vite `root` 为基准；显式 CLI 输入仍以执行目录为基准。使用 `vize.entries` 和目标包的 TypeScript 项目来设置包的范围。


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
