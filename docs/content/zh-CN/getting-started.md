---
title: 入门指南
description: 通过 Vite 配置和 TypeScript 项目将 Vize 加入 Vue。
---

<!-- Reviewed translation; source: getting-started.md -->

# 入门指南

通过 [Vite+](https://viteplus.dev/guide/install) 将 Vize 加入 Vue 3 应用。工具设置放在 `vite.config.ts` 中，TypeScript 项目设置保留在 `tsconfig.json` 中。Vize 仍在开发；采用前请查看[支持状态](./stability.md)。

## 1. 安装集成

在已安装 Vite+ 的现有 Vue 项目中运行：

```bash
vp install -D @vizejs/vite-plugin
```

集成使用项目的 Vite+ 版本（0.2.3 或更高）。普通 Vite 请参阅[插件迁移](/guide/migration.md#vite-plugin)，Nuxt 请参阅 [Nuxt 集成](./integrations/nuxt.md)。

## 2. 更新 `vite.config.ts`

该函数添加 Vize 编译器和原生检查任务。删除旧 Vue 插件的 import 以及 `plugins` 中的 `vue()`，保留 alias、server、test 和其他插件。[迁移指南](/guide/migration.md)提供完整的修改前后示例。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

## 3. 构建并检查应用

```bash
vp dev
vp build
vp run check
```

`vp run check` 将 Vue 类型检查、lint 和格式化与 Oxlint、Oxfmt 一起执行。开发时也可以运行单独的任务：

```bash
vp run typecheck
vp run lint
vp run fmt:check
```

使用 `vp run check -- --fix` 应用修复。生成的 Vize 任务应通过 `vp run` 执行：内置的 `vp check`、`vp lint` 和 `vp fmt` 运行 Vite+ 自身的工具。Vize 保留已有脚本，名称冲突时生成 `vize:<名称>`；请参阅[任务名称和覆盖设置](/guide/vite-plus.md#tasks)。

## 选择下一步

- [修改规则或编译器选项](./guide/configuration.md)。
- [迁移现有工具](/guide/migration.md)，查看完整的修改前后示例。
- [理解 lint 诊断](./rules/all.md)，对照 Vue 的问题示例和修正示例。
- [浏览组件](./guide/ui/index.md)，查看用法和 API。
- [使用 Musea 预览组件](./guide/musea.md)。
- [设置编辑器](/guide/vite-plus-editor.md)，共享原生工具配置。

尚未翻译的指南链接指向英文版。

> [!NOTE]
> 原生 CLI 和编辑器读取 Vite 配置，以及不创建专用配置的 init，正在为下一个版本准备。在该版本发布前，已发布的原生工具仍需使用现有的专用格式来自定义共享设置。[参考文档](./guide/configuration-reference.md)介绍该格式。

## 不使用 Vite+ 的场景

[独立 CLI](./guide/cli.md)提供 `vize lint`、`vize fmt` 和 `vize check`。从目标包的根目录运行，并将 TypeScript 项目保留在 `tsconfig.json` 中。原生命令也可以共享 `vite.config.*` 顶层的 `vize` 设置；搜索规则和当前限制请参阅 [CLI 配置](./guide/configuration.md#standalone-cli)。

先用 `vpx vize init --dry-run` 预览交互式设置，再运行 `vpx vize init` 选择功能。[项目设置](/guide/init.md)介绍检测和编辑限制。init 使用项目设置和默认值，不会创建专用的 Vize 配置文件。

<span id="设置现有项目"></span>
<span id="选择手动配置"></span>
<span id="继续阅读专题指南"></span>
