---
title: 配置
description: 通过现有的 Vite 配置和 TypeScript 项目共享 Vize 设置。
---

<!-- Reviewed translation; source: guide/configuration.md -->

# 配置

将 Vize 设置放在 `vite.config.ts` 中，将 TypeScript 项目设置保留在 `tsconfig.json` 中。使用默认设置时，无需额外的 Vize 配置文件。

> [!NOTE]
> 原生 CLI 和编辑器读取 Vite 配置，以及不创建专用配置的 init，正在为下一个版本准备。在该版本发布前，已发布的原生工具仍需使用现有的专用格式来自定义共享设置。[参考文档](./configuration-reference.md)介绍该格式。

## Vite+ 配置

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: { sourceMap: true },
  lint: { vize: { preset: "essential" } },
  fmt: { vize: { printWidth: 100 } },
  typecheck: { strict: true },
});
```

| 位置 | 用途 | 命令 |
| --- | --- | --- |
| `compiler` | Vue 编译 | `vp dev`, `vp build` |
| `lint.vize` | Vue lint 规则 | `vp run lint` |
| `fmt.vize` | Vue 格式化 | `vp run fmt:check` |
| `typecheck` | Vue 类型检查 | `vp run typecheck` |
| `pack.vize` | 库类型声明 | `vp run pack` |

使用 `vp run check` 运行组合的 Vize 任务。内置的 `vp check`、`vp lint` 和 `vp fmt` 保持 Vite+ 自身的行为。已有脚本可能使生成的任务改名为 `vize:<名称>`；请参阅[任务名称和覆盖设置](./vite-plus.md#tasks)。

## 修改一条规则

Vue 规则放在 `lint.vize.rules` 中，Oxlint 规则放在 `lint.rules` 中。[规则选项](../rules/options.md)介绍参数，[规则目录](../rules/all.md)提供完整示例。

```ts
export default defineConfig({
  lint: {
    vize: { rules: { "vue/no-v-html": "error" } },
    rules: { "no-debugger": "error" },
  },
});
```

## 选择启用的功能

将 `compiler`、`typecheck`、`lint.vize` 或 `fmt.vize` 设为 `false` 即可禁用对应功能。Vize 格式化 Vue 文件，Oxfmt 处理其他文件。请参阅[职责划分和冲突处理](./vite-plus.md#lint-and-formatter-ownership)。

## 普通 Vite

使用插件进行编译，并在顶层 `vize` 对象中配置 CLI 和编辑器共用的设置。导入插件也会添加 Vite 配置的类型定义。

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  vize: {
    linter: { preset: "essential" },
    formatter: { printWidth: 100 },
    typeChecker: { strict: true },
  },
});
```

<span id="standalone-cli"></span>

## 独立 CLI

安装 `vize` 后，从目标包的根目录运行命令。CLI 也会读取 `vite.config.*` 和 TypeScript 项目。Vite+ 的 `compiler`、`typecheck`、`lint.vize` 和 `fmt.vize` 设置也会转换为原生命令的配置。顶层 `vize` 中显式设置的原生选项优先于转换结果。

```bash
vp install -D vize
vp exec vize check
```

配置搜索会在最近的 `package.json`、`tsconfig.json` 或 `jsconfig.json` 所在目录停止。在 monorepo 中，请从目标包运行命令或使用 `vize.entries`，并显式选择编辑器的工作区文件夹。按文档自动发现嵌套 Vite 配置仍在开发中。

使用 Vite 配置时，不带输入参数的 `build`、`lint`、`fmt` 和 `check` 以选定的 Vite `root` 为目标。相对 `root` 从配置文件目录解析。Vite 中的 `typeChecker` 路径和作用域 `basePath` 以该 root 为基准；专用配置中的路径仍以配置文件目录为基准。显式传给 CLI 的文件、glob 和 `--tsconfig` 仍以执行命令的目录为基准。

共享的全局忽略模式会将文件排除在 CLI 文件搜索和编辑器 lint 之外。被忽略的文件在编辑器中打开后，仍可获得语法、类型和导航诊断。模式的顺序和含义，包括 `!` 否定模式，都会保留。

## 可选的专用配置

已有的 `vize.config.*` 仍受支持，且优先于同目录的 Vite 配置。CLI 的 `--config` 可显式选择文件。直接传给插件的选项和编辑器中显式设置的功能优先于共享设置；`config: false` 禁用插件的自动配置加载。

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

[共享配置参考](./configuration-reference.md)介绍文件发现、专用 TypeScript/JSON/PKL 格式、分范围配置、Vue 类型解析和 LSP/Musea 设置。编译选项和语法模式请参阅[编译器参考](./compiler-configuration-reference.md)，需显式启用的功能请参阅[实验性功能](./experimentals.md)。

