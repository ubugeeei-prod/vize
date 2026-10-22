---
title: 共享配置参考
---

<!-- Reviewed translation; source: guide/configuration-reference.md -->

<span id="standalone-configuration-reference"></span>

# 共享配置参考

在现有 Vite 配置中使用 `vue:`，在 TypeScript 项目中使用 `vueCompilerOptions`。最小配置见[配置指南](./configuration.md)。

## 配置文件

CLI 和编辑器发现最近项目中的 `vite.config.ts`、`.mts`、`.cts`、`.js`、`.mjs` 和 `.cjs`。`package.json`、`tsconfig.json` 和 `jsconfig.json` 构成项目边界。命令可以通过 `--config` 选择 Vite 配置。

专用的 `vize.config.*` 文件已不再受支持。与 Vite 配置并存的旧专用文件会报错，和只有专用配置的项目一样收到迁移错误。将旧 TypeScript、JSON 或 Pkl 配置中的原生设置迁入 `vue:`，不要继续创建专用文件。

Vite 配置中的原生相对路径以选定的 Vite `root` 为基准，`root` 自身从配置文件目录解析。显式 CLI 输入和 `--tsconfig` 以执行命令的目录为基准。直接传给插件的选项和编辑器中显式设置的功能开关优先于共享设置。

## TypeScript 配置

```ts
// vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  server: { port: 3000 },
}).withVue({
  compiler: {
    sourceMap: true,
    vapor: false,
    templateSyntax: "standard",
  },
  linter: { preset: "happy-path" },
  formatter: { printWidth: 100, singleQuote: false },
  languageServer: { hover: true },
});
```

`vue` 接受 `version`、`compiler`、`typeChecker`、`linter`、`formatter`、`languageServer`、`globalTypes`、`files`、`ignores`、`basePath`、`lib`、`vite`、`musea`、`experimentals`、`dialect`、`name` 和 `entries`。请使用原生选项名；未知成员会产生配置错误。编译模式见[编译器参考](./compiler-configuration-reference.md)，带类型的 lint 选项见[规则选项](/rules/options.md)。

## Vue 类型解析

`vize check` 和语言服务器从被分析的项目解析 Vue 类型。请在用户项目中声明需要的 Vue 版本，并让 `vue`、`@vue/compiler-sfc` 和 Nuxt 等框架集成保持一致。Vize 自身构建时的依赖不会决定用户项目使用的 Vue 版本。支持多个 Vue 系列的项目应分别验证各个依赖范围。

环境声明、生成的自动导入文件、路径别名和 Vue `ComponentCustomProperties` 声明保留在项目的 `tsconfig.json` 中。标准 TypeScript `extends`（包括包路径和数组形式）及项目 `references` 保持原有含义。

```json
{
  "extends": "./tsconfig.base.json",
  "compilerOptions": { "strict": true, "noEmit": true },
  "vueCompilerOptions": {
    "target": 3.5,
    "strictTemplates": true,
    "checkUnknownComponents": true
  },
  "include": ["src/**/*"]
}
```

支持的 Vue 选项包括 `target`、`strictTemplates`、`checkUnknownProps`、`checkUnknownComponents`、`checkUnknownEvents`、`checkUnknownDirectives`、`strictVModel`、`strictCssModules`、`jsxSlots`、`inferComponentDollarEl`、`inferComponentDollarRefs`、`inferTemplateDollarEl`、`inferTemplateDollarRefs`、`inferTemplateDollarSlots`、`inferTemplateDollarAttrs`、`fallthroughAttributes`、`checkRequiredFallthroughAttributes`、`resolveStyleImports`、`resolveStyleClassNames` 和 `vapor`。这不代表已支持全部 `vue-tsc` 选项。

`target` 接受 Vue 2.x（最高 2.7）、Vue 3.x、`99` 和 `"auto"`。自动选择使用项目安装的 Vue 包；Vite 配置中显式设置的 `vue.version` 优先。不支持的目标值会产生配置错误。TypeScript 严格性保留在 `compilerOptions` 中；显式 Vize 类型检查选项覆盖对应的项目默认值。

### TypeScript 项目归属与支持边界

这些设置来自源文件所属的 TypeScript 项目，包括继承的 JSONC、标准包路径或数组形式的 `extends` 以及项目 `references`。`target: "auto"` 解析该项目安装的 Vue 包；找不到或版本不受支持时会产生配置错误。小于 2.7 的数值 Vue 2 目标选择 Vue 2，恰好 2.7 选择 Vue 2.7，Vue 3.x 或 `99` 选择现代 Vue 3 系列。Vite 中显式设置的 `vue.version`（包括 `"3"`）优先于 `target`。

还支持 `strictComponentAttrs`，但需要显式启用；`strictTemplates` 不会自动开启它。`plugins` 仅识别标识符 `vue-router/volar/sfc-typed-router`，不会加载任意 Vue 语言插件。

`optionsWrapper` 尚不支持。它在 Vue 工具中表示代码前缀和后缀包装，并不是 Options API 开关。Vize 已有的 Options API 检查通过独立的 `vue.typeChecker.optionsApi` 设置。以上是有限的选项映射，不代表与全部 `vue-tsc` 选项完全等价。

## 实验性作用域条目

Monorepo 类型检查使用目标包的 TypeScript 项目。Vue lint 的作用域设置可以使用 `vue.entries`：

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  root: "apps/web",
  vue: {
    entries: [
      {
        files: ["src/trusted/**/*.vue"],
        linter: { rules: { "vue/no-v-html": "off" } },
      },
    ],
  },
});
```

条目接受 `basePath`、`files`、`ignores` 和 `linter`；其他部分保持项目级设置。不要将 TypeScript 项目引用放入这些条目。

## 复用配置

不再提供 Vize 专用的 `extends`。使用普通 JavaScript 对象复用设置：

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import vpConfig from "./build-options.ts";
import vueConfig from "./vue-options.ts";

export default defineConfig(vpConfig).withVue(vueConfig);
```

这不会改变标准 TypeScript `extends` 或 Vite 配置函数的行为。

## 静态分析选项

原生 Vue lint 选项放在 `vue.linter` 中，语言服务器选项放在 `vue.languageServer` 中。Vue 规则使用 `vue.linter.rules`，Oxlint 规则保留在 `lint.rules` 中。跨文件和类型感知检查见 [lint 执行](/guide/lint-execution.md)（英文），完整选项对象见[规则选项](/rules/options.md)。未知规则选项字段会被拒绝。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
  typeChecker: {
    enabled: true,
    checkProps: true,
    checkEmits: true,
    checkTemplateBindings: true,
    optionsApi: true,
  },
});
```

`vue.typeChecker.optionsApi` 默认启用，解析 Vue 3 Options API 的模板绑定，包括普通 `<script>` 中的 `data`、`computed`、`methods`、`inject`、`setup` 和 `props`。它仅对非 `<script setup>` 组件运行，可设为 `false` 关闭。Legacy Vue 2.7 / Nuxt 2 支持是独立的 `legacy` 构建功能。

`vue.typeChecker.tsconfig` 选择其他 TypeScript 项目。只有需要指定已安装的检查器可执行文件时才设置 `vue.typeChecker.corsaPath`；它不会选择 Vue 类型版本。`vue.typeChecker.servers` 限制并行检查器进程数（`1`–`32`），省略时 Vize 会根据项目和可用 CPU 自动调整并发数。

<span id="博物馆选项"></span>

## Musea 共享配置

共享的文件发现、路由和 VRT 设置放在 `vue.musea` 中。直接传给 `musea()` 的选项覆盖对应的共享值。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [musea({ storybookCompat: false })],
  vue: {
    musea: {
      include: ["src/**/*.art.vue"],
      exclude: ["node_modules/**", "dist/**"],
      basePath: "/__musea__",
      inlineArt: false,
    },
  },
});
```

`previewCss`、`previewSetup`、`tokensPath`、`theme` 和 `storybookOutDir` 等展示选项属于 `musea()` 插件选项。

`musea-vrt` 从同一 Vite 配置中读取共享的 `include`、`exclude` 和 `vrt` 选项。`vize musea new` 创建示例故事，不会创建专用配置文件。

## 迁移示例

专用 JSON/PKL 配置和 Vize 自定义 `extends` 已移除。请通过导入普通 JavaScript 对象复用配置。以下示例保留迁移前支持的设置。

### 动态 Vite 配置

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  vue: {
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
    languageServer: {
      lint: true,
      typecheck: false,
      editor: false,
      formatting: false,
    },
    musea: {
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
    },
  },
}));
```

### 迁移已移除的 JSON 和 PKL 设置

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: { scanPatterns: ["src/**/*.vue"] },
  linter: { preset: "happy-path" },
  typeChecker: { enabled: true, strict: true },
  languageServer: { lint: true, typecheck: false, editor: false, formatting: false },
  musea: { include: ["src/**/*.art.vue"], basePath: "/__musea__" },
});
```

### 按包划分项目

`entries` 只划分 lint 设置的作用域。每个包的检查器与格式化设置应放在该包自己的 Vite 和 TypeScript 项目中。下面两个文件属于不同的包项目。

```ts
// apps/web/vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  name: "web app",
  files: ["src/**/*.vue"],
  formatter: { printWidth: 100 },
  typeChecker: { tsconfig: "tsconfig.app.json" },
});
```

```ts
// packages/ui/vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
export default defineConfig({}).withVue({
  name: "ui package",
  files: ["src/**/*.vue"],
  formatter: { printWidth: 100, singleQuote: true },
});
```

### 静态分析与检查进程

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
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

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  typeChecker: { servers: 1 },
});
```

### 现有工作流设置

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: {
    printWidth: 100,
  },
  linter: {
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    tsconfig: "tsconfig.json",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```
