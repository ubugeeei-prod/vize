---
title: VS Code
---

<!-- Generated translation; source: integrations/vscode.md -->

# VS 代码集成

> **⚠️ 正在进行中：**Vize 的编辑支持仍处于实验阶段。

> **重要：**日常的Vue编辑器支持请继续使用官方的Vue语言工具
> （`vuejs/language-tools`）暂时。Vize专为增量选择加入评估设计。

该仓库包含两个实验性的 VS Code 扩展：

- **Vize**— 由`vize lsp`支持的Vue语言支持
- **Vize Art**— Musea `*.art.vue` 文件的语法高亮

从VS Code市场安装：

```bash
code --install-extension ubugeeei.vize
code --install-extension vize.vize-art
```

如果你想`*.art.vue`接收 Vize 悬停、完成、访问定义和
除了语法高亮外，还支持引用。

## 维兹扩展

Vize扩展从`vize lsp`开始，可以选择加入特定的能力包。
当你打开一个Vue文件时，扩展名仍然被禁用或未启用任何功能，扩展包现在会提供一键推荐的工作区设置，这样悬停、跳转和诊断功能就不会无声地关闭。
该配置为当前工作区写入`vize.enable`、`vize.lint.enable`、`vize.typecheck.enable`和`vize.editor.enable`。
如果你手动只设置`vize.enable: true`，Vize也会使用推荐的诊断方法，
编辑器配置文件代替启动一个空的语言服务器。
Vize状态栏条项会`Vize: Show Status`打开，显示配置文件切换器和服务器
二进制选择器、重启操作、设置和日志都能在一个地方实现。

### 推荐起点

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

这使得绒毛检测优先，导航、完成和格式化则交由您
现有的Vue工具。

### 常见设定

| 背景                         | 目的                       |
| ---------------------------- | -------------------------- |
| `vize.enable`                | 启用扩展和语言服务器       |
| `vize.serverPath`            | 覆盖`vize`可执行路径       |
| `vize.lint.enable`           | 启用绒毛诊断               |
| `vize.typecheck.enable`      | 启用类型感知诊断和后端功能 |
| `vize.editor.enable`         | 启用编辑器辅助包           |
| `vize.completion.enable`     | 启用完备化                 |
| `vize.formatting.enable`     | 启用文档格式化             |
| `vize.definition.enable`     | 启用访问定义               |
| `vize.references.enable`     | 启用参考文献               |
| `vize.hover.enable`          | 启用悬停                   |
| `vize.codeActions.enable`    | 启用绒毛快速修复           |
| `vize.semanticTokens.enable` | 启用语义标记               |
| `vize.trace.server`          | 跟踪 LSP 通信              |

### 有用的指令

| 指挥                                      | 目的                                |
| ----------------------------------------- | ----------------------------------- |
| `Vize: Show Status`                       | 打开状态与设置操作中心              |
| `Vize: Enable Recommended Profile`        | 启用除絮、字体检查和编辑协助        |
| `Vize: Enable Lint-Only Profile`          | 在保持其他工具使用的同时启用诊断    |
| `Vize: Select Language Server Executable` | 从文件选择器中设置`vize.serverPath` |
| `Vize: Disable Language Server`           | 当前配置目标的停止Viceze            |
| `Vize: Restart Language Server`           | 重启语言服务器                      |
| `Vize: Show Output Channel`               | 显示扩展和 LSP 日志                 |

### 扩展使用情况

```text
VS Code
  ↕ Language Server Protocol
vize lsp (vize_maestro)
  → vize_armature
  → vize_croquis
  → vize_patina
  → vize_canon
  → vize_glyph
```

### 从源代码或VSIX安装

从[Vite+安装指南](https://viteplus.dev/guide/install)中安装一次`vp`，然后：

```bash
git clone https://github.com/ubugeeei-prod/vize.git
cd vize
cd editors/vscode
vp install -- --ignore-workspace
vp pack
vp exec vsce package --no-dependencies --out dist/vize.vsix
code --install-extension dist/vize.vsix
```

## Vize艺术扩展

`Vize Art`为 Musea `*.art.vue` 文件提供语法高亮。
其市场分机编号为`vize.vize-art`。

它认可：

- `<art>` 元数据块
- `<variant>`块
- 标准的Vue `<template>`、`<script>`和`<style>`部分

## 其他编辑

`vize lsp`遵循语言服务器协议，并可被 Neovim、Helix 等编辑器使用，
Zed和Emacs。

Neovim 设置示例：

```lua
require("lspconfig").vize.setup({
  cmd = { "vize", "lsp" },
  filetypes = { "vue" },
  init_options = {
    lint = true,
    typecheck = true,
    editor = true,
  },
})
```

`editor = true`是测试悬停、完成、跳跃、引用和符号的最简单方法
一起。当其他TypeScript服务器如tsgo拥有项目诊断时，保持
`typecheck = false`并只开启你想评估的Vue专属功能。
