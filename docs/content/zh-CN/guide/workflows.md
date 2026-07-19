---
title: 用户工作流程
---

<!-- Generated translation; source: guide/workflows.md -->

# 用户工作流程

本指南简明扼要地介绍了常见的 Vize 工作流程：安装、连接配置，
格式化、lint、类型检查、编译，并在CI中运行相同的门。

## 安装

在拥有你 Vue 依赖的项目中安装 npm 包：

```bash
vp install -D vize
```

对于 monorepo，当包共享一个锁文件时，在工作区根安装。安装在
只有当该包拥有自己的锁文件和依赖图时才会被打包。

## 添加包脚本

优先使用命名脚本而非一次性命令，这样本地和CI运行共享相同的入口点：

```json
{
  "scripts": {
    "vize:fmt": "vize fmt --check src",
    "vize:fmt:fix": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path --max-warnings 0 src",
    "vize:check": "vize check src",
    "vize:build": "vize build src",
    "vize:ready": "vize ready src"
  }
}
```

`vize ready`是宽广的地方大门。在较大的仓库中，也要保留单独的命令
开发者可以隔离格式化、LINT、类型检查和编译器故障。

## 配置一次

当默认设置不足时，在项目根创建`vize.config.ts`：

```ts
import { defineConfig } from "vize";

export default defineConfig({
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

参见[配置](./configuration.md)中关于扁平单仓库条目、PKL、JSON、编译器选项和
Vue类型的分辨率细节。

## 节目形式

在CI中使用检查模式，本地写入模式：

```bash
vp run vize:fmt
vp run vize:fmt:fix
```

对于一次性迁移工作，`vize fmt --write`可以针对文件、目录或环状物。

## 绒毛

首先从`happy-path`开始，以保证正确性和低噪声的Vue诊断：

```bash
vize lint --preset happy-path --max-warnings 0 src
```

当CI输出应保持紧凑时使用`--help-level short`，换工具时`--format json`
会消耗诊断数据。完整规则请参见 [CLI](./cli.md) 和 [Rules](../rules/index.md)
浮出水面。

## 类型检查

从项目根节点运行`vize check`，比如活动`tsconfig`、Vue版本、框架包，
环境类型来自相同的依赖图：

```bash
vize check src
```

对于针对特定软件包的单仓库检查，可以从包目录运行或设置`typeChecker.tsconfig`
在一个有作用域的配置条目中。

## 编译

当你需要在 Vite 插件路径之外输出编译器时使用`vize build`：

```bash
vize build src --output dist/vize
```

对于 Vite 应用，优先选择 `@vizejs/vite-plugin`，让 Vite 拥有构建编排。参见
[Vite插件](./vite-plugin.md)。

## CI

在CI中使用相同的包脚本：

```yaml
- run: vp install --frozen-lockfile
- run: vp run vize:fmt
- run: vp run vize:lint
- run: vp run vize:check
```

只有当项目直接调用 Vize 编译器的输出时，才把`vize:build`留在门内。对于
Vite应用程序，普通的应用构建就是插件的运行。

## 调试失败

当故障不明确时：

- 带`--format json`重运行以检查稳定的诊断场;
- 在`check`、`lint`或`build`上使用`--profile`来寻找慢相位;
- 创建带有编译器不匹配的`vize inspector`检查器有效载荷;
- 请求修复时包含最小的`.vue`文件或项目切片。

[测试与反馈](./testing.md)和[故障排除](./troubleshooting.md)页面涵盖了内容
报告、真实世界的固定装置以及常见的环境问题。
