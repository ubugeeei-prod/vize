---
title: 入门指南
---

<!-- Generated translation; source: getting-started.md -->

# 入门指南

> **⚠️ 开发中：** Vize 正在积极开发，目前尚未准备好用于生产环境。
> API 和软件包边界可能会随时更改，恕不另行通知。

Vize (_/viːz/_) 是一个用 Rust 原生实现的 Vue.js 工具链。它将编译、代码检查、格式化、
类型检查、编辑器诊断和组件浏览整合在同一个工作区中，同时仍可通过专用的软件包和命令
单独使用每项功能。

| 需求                                     | 推荐入口                    |
| ---------------------------------------- | --------------------------- |
| 在 Vite 中编译 Vue SFC                   | `@vizejs/vite-plugin`       |
| 在 Nuxt 中编译 Vue SFC                   | `@vizejs/nuxt`              |
| 从项目脚本运行代码检查、格式化和类型检查 | `vize`                      |
| 将 Vize 诊断与 Oxlint 组合使用           | `oxlint-plugin-vize`        |
| 浏览和测试组件                           | `@vizejs/vite-plugin-musea` |
| 试用编辑器功能                           | VS Code、Zed 或 `vize lsp`  |

## 设置现有项目

在项目根目录运行交互式初始化命令：

```bash
vpx vize init
```

`vpx` 随 [Vite+](https://viteplus.dev/guide/install) 一起提供。如果当前 shell 中没有此命令，
请先安装 Vite+。

写入任何文件之前，`vize init` 会检测 Vite、Vite+ 或 Nuxt、软件包管理器、TypeScript、
当前使用的 lint 命令以及已有的 Vize 配置。你可以单独选择要配置的功能：

- Vite 插件或 Nuxt 模块
- Oxlint 插件，并写入当前 lint 命令实际读取的配置文件
- `vize fmt` 和 `vize check` 项目脚本
- 共享的 `vize.config.*` 设置
- VS Code 扩展推荐

不写入任何内容，预览所有计划中的文件和依赖项更改：

```bash
vpx vize init --dry-run
```

在 CI 或其他非交互环境中，请明确选择所需功能：

```bash
vpx vize init --yes --lint --bundler --fmt --typecheck --editor
```

有关检测规则、全部选项、幂等性保证，以及初始化程序会主动拒绝编辑文件的情况，
请参阅 [Project Setup（英文）](../guide/init.md)。

## 选择手动配置

如果需要保留现有配置，或希望逐步采用 Vize 的单项功能，请使用手动配置：

- [Vite 插件](./guide/vite-plugin.md) — 在 Vite 中原生编译 Vue SFC
- [Nuxt 集成](./integrations/nuxt.md) — 通过 Nuxt 自身 Vite 管线的受支持方式
- [软件包脚本和 CLI](./guide/cli.md) — `vize build`、`fmt`、`lint`、`check`、`ready`
  以及完整的 Rust CLI

Vite 是推荐的打包器集成。unplugin 和 Rspack 软件包仍处于实验阶段；请参阅
[其他打包器](./guide/unplugin.md)了解当前范围。

## 继续阅读专题指南

本页有意只提供入门导览。有关配置和集成的详细信息，请以以下专题指南为准：

- [配置](./guide/configuration.md) — `vize.config.*`、编译器选项、类型检查和 Musea 设置
- [静态分析](./guide/static-analysis.md) — 代码检查和类型检查模型
- [规则文档](./rules/index.md) — 具体诊断及示例
- [Oxlint 插件](./guide/oxlint.md) — 预设、设置以及每个命令实际读取的配置文件
- [VS Code 和其他编辑器](./integrations/vscode.md) — 选择启用的编辑器配置和 LSP 设置
- [JSX 与 TSX](./guide/jsx.md) — 在 `.vue` SFC 之外编写 Vue 组件
- [Musea](./guide/musea.md) — 组件示例、文档、设计令牌、a11y 和 VRT

在 Vize 编辑器集成仍处于实验阶段时，日常 Vue 开发请继续使用官方的
[`vuejs/language-tools`](https://github.com/vuejs/language-tools)。
