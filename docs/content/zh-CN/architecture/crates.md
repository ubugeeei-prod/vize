---
title: 板条箱
---

<!-- Generated translation; source: architecture/crates.md -->

# 板条箱参考

> **⚠️ 正在进行中：**Vize 正在积极开发中。请参阅规范
> [Rust crate 支持等级](../stability.md#rust-crate-support-tiers) 之前依赖于公共
> API。

Vize 的 Rust 工作区由 20 个主要 crate 组成。每个板条箱都拥有一条可重复使用的通道，因此
解析、语义分析、代码生成、linting、格式化、类型检查和
编辑器工具可以共享相同的语法模型。

## 基础

| 板条箱            | 角色                                                                 |
| ----------------- | -------------------------------------------------------------------- |
| `vize_carton`     | 共享分配器、字符串、哈希集合、标志、分析器、i18n 和 DOM/标签实用程序 |
| `vize_relief`     | 共享 Vue 模板 AST、编译器错误和编译器选项                            |
| `vize_armature`   | Vue 模板分词器和解析器                                               |
| `vize_croquis`    | 语义分析、范围跟踪、绑定元数据、反应性和虚拟 TS 助手                 |
| `vize_croquis_cf` | 选择加入跨文件语义分析和项目范围的诊断                               |

## 编译

| 板条箱               | 角色                                |
| -------------------- | ----------------------------------- |
| `vize_atelier_core`  | 共享转换通道和代码生成基础设施      |
| `vize_atelier_dom`   | 面向VDOM的模板编译                  |
| `vize_atelier_vapor` | Vapor-mode模板编译                  |
| `vize_atelier_ssr`   | 服务端渲染模板编译                  |
| `vize_atelier_sfc`   | `.vue` 解析加上脚本、模板和样式编排 |
| `vize_atelier_jsx`   | 共享 JSX/TSX 解析、降低和编译器集成 |

## 开发者工具

| 板条箱         | 角色                                                  |
| -------------- | ----------------------------------------------------- |
| `vize_patina`  | Vue SFC linter 和诊断格式化                           |
| `vize_glyph`   | Vue SFC 格式化程序                                    |
| `vize_canon`   | Vue 感知类型检查和虚拟 TypeScript 生成                |
| `vize_maestro` | 语言服务器协议实现                                    |
| `vize_musea`   | Musea 艺术解析、文档、调色板生成、autogen 和 VRT 核心 |
| `vize_curator` | 本地检查器有效负载、图形/差异元数据和配置文件报告     |
| `vize_fresco`  | 面向 TUI 的实验使用的终端 UI 原语                     |

## 分布层

| 板条箱         | 角色                                   |
| -------------- | -------------------------------------- |
| `vize_vitrine` | 为 JS 消费者提供共享 NAPI 和 WASM 绑定 |
| `vize`         | Rust 原生 CLI 加上 crate 重新导出文档  |

## 注释

- `vize_musea` 是 Musea 艺术工具的 Rust 核心。图库 UI 和开发服务器工作流程是
  由 `@vizejs/vite-plugin-musea` 提供。
- `vize_curator` 未发布。它拥有本地开发人员工件，例如检查器有效负载，
  代理报告、跨文件图形元数据和 CLI 配置文件报告呈现。低级
  探查器保留在 `vize_carton` 中，因为共享板条箱检测它们自己的热路径。
- `vize_vitrine` 是 Rust 到 JS 的桥梁。 `@vizejs/native` 等封装和
  `@vizejs/wasm` 发布其绑定。
- `vize` 是工作区中完整的 Rust CLI 包。对于 v1 alpha，其公共二进制通道是
  GitHub Releases 或 Nix，而 npm `vize` 包是受支持的包脚本入口点。

## 包映射

|包/命令|主 Rust 板条箱 |
| ------------------------ | | ---------------------------------------------------------------------------------------------------- |
| `vize build` | `vize`、`vize_atelier_sfc`、`vize_atelier_dom`、`vize_atelier_vapor`、`vize_atelier_ssr` |
| `vize fmt` | `vize`、`vize_glyph` |
| `vize lint` | `vize`，`vize_patina` |
| `vize check` | `vize`，`vize_canon` |
| `vize inspector` | `vize`，`vize_curator` |
| `vize lsp` | `vize`、`vize_maestro` |
| `@vizejs/vite-plugin` | `vize_vitrine`，`vize_atelier_sfc` |
| `@vizejs/native` | `vize_vitrine` |
| `@vizejs/wasm` | `vize_vitrine` |
| `@vizejs/vite-plugin-musea` | `vize_musea`、`vize_vitrine` |
| `@vizejs/musea-mcp-server` | `vize_musea`，`vize_vitrine` |
| `oxlint-plugin-vize` | `vize_patina`、`vize_vitrine` |
