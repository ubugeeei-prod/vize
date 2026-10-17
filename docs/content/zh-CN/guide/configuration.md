---
title: 配置
description: 通过现有的 Vite 配置和 TypeScript 项目共享 Vue 工具设置。
---

<!-- Reviewed translation; source: guide/configuration.md -->

# 配置

将 Vue 工具设置放在 `vite.config.*` 的 `vue:` 中，将 TypeScript 项目设置保留在 `tsconfig.json` 中。CLI、Vite 集成和编辑器扩展读取这些项目文件，无需专用的 Vize 配置。

## 保留现有的 Vite+ 配置

使用 Vize 的辅助函数包装现有配置：

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

const vpConfig = {
  server: { port: 3000 },
};

const vueConfig = {
  compiler: { sourceMap: true },
  linter: { preset: "essential" },
  formatter: { printWidth: 100 },
};

export default defineConfig(vpConfig).withVue(vueConfig);
```

`withVue` 保留现有的 Vite+ 选项、插件和钩子，支持配置对象、函数、Promise 和异步函数。也可以直接在配置中写 `vue: { ... }`。路径、作用域和配置组合详见[共享 Vue 配置](/guide/vue-configuration.md)（英文）。

## TypeScript 和 Vue 模板检查

保留现有的 TypeScript 项目，包括标准的 `extends` 和 `references`。Vue 模板编译选项放在 `vueCompilerOptions` 中：

```json
{
  "compilerOptions": {
    "strict": true,
    "noEmit": true,
    "jsx": "preserve"
  },
  "vueCompilerOptions": {
    "strictTemplates": true
  },
  "include": ["src/**/*"]
}
```

使用 `vue-tsc` 的现有项目可以保留原有布局。TypeScript 的严格性由 `compilerOptions` 决定，Vue 专用选项属于 `vueCompilerOptions`。支持的选项和显式 Vize 覆盖设置详见[配置参考](./configuration-reference.md)。

<span id="lint-rule-options"></span>

## 修改 lint 规则

Vue 规则放在 `vue.linter.rules` 中，Oxlint 规则保留在 `lint.rules` 中。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: { rules: { "no-debugger": "error" } },
}).withVue({
  linter: { rules: { "vue/no-v-html": "error" } },
});
```

[规则选项](/rules/options.md)介绍参数，[规则目录](../rules/all.md)提供问题和修正示例。不支持的选项字段会被拒绝。

## 普通 Vite

保留现有的 Vite 配置，并替换 Vue 编译插件：

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  vue: {
    linter: { preset: "essential" },
    formatter: { printWidth: 100 },
  },
});
```

<span id="standalone-cli"></span>

## CLI 和编辑器

从目标包的根目录运行命令。编辑器在选定工作区内解析每个文档所属的项目，并读取其 Vite 和 TypeScript 设置。Monorepo 中的各个包保留自己的 `tsconfig.json`、项目引用和 Vite 配置。使用 `--tsconfig` 可以显式选择其他 TypeScript 项目。

```bash
vp exec vize check
vp exec vize lint src
vp exec vize fmt --check src
```

Vite 配置中的原生相对路径以选定的 Vite `root` 为基准；相对 `root` 从配置文件目录解析。显式 CLI 文件参数和 `--tsconfig` 仍以执行命令的目录为基准。CLI 功能标志和编辑器中显式设置的功能开关优先于共享设置。

共享的全局忽略模式将文件排除在 CLI 文件搜索和编辑器 lint 之外。被忽略的文件在编辑器中打开后，仍可获得语法、类型和导航诊断。模式的顺序和含义，包括 `!` 否定模式，都会保留。

## 迁移已移除的配置

`vize.config.*` 和顶层 `vize:` 配置已不再受支持。将原生设置移到 `vite.config.*` 的 `vue:` 中，将 TypeScript 项目选项移到 `tsconfig.json` 中。迁移并检查结果后，再移除专用配置文件。与 Vite 配置并存的旧专用文件会被忽略；只有专用配置的项目会收到迁移错误。`vize init` 和 `vize setup` 会保留原文件并说明迁移方式。

不再提供 Vize 专用的 `extends`。要复用设置，请导入普通 JavaScript 对象，并传给 `defineConfig(vpConfig).withVue(vueConfig)`。标准 TypeScript `extends` 和项目 `references` 继续有效。

<span id="配置文件"></span>
<span id="typescript-配置"></span>
<span id="vue类型解析"></span>
<span id="实验平面作品"></span>
<span id="pkl-配置"></span>
<span id="json-配置"></span>
<span id="编译器选项"></span>
<span id="模板语法"></span>
<span id="jsx-和-tsx-输出模式"></span>
<span id="每个组件指令"></span>
<span id="优先权"></span>
<span id="诊断"></span>
<span id="vue方言"></span>
<span id="静态分析选项"></span>
<span id="博物馆选项"></span>

## 详细参考

[共享配置参考](./configuration-reference.md)介绍文件发现、作用域、Vue 类型解析以及 LSP/Musea 设置。编译选项和语法模式请参阅[编译器参考](./compiler-configuration-reference.md)，需显式启用的功能请参阅[实验性功能](/guide/experimentals.md)。

> [!NOTE]
> CLI 和编辑器读取现有的 Vite 与 TypeScript 项目文件。删除旧专用配置文件之前，请先迁移其中的设置。
