---
title: oxlint-plugin-vize Alpha
description: 新的 Oxlint JS 插件桥接将 Vize Patina 诊断整合到单一 Vue SFC 运行中。
---

<!-- Generated translation; source: blog/releases/2026-03-26-oxlint-plugin-vize-alpha.md -->

# `oxlint-plugin-vize`阿尔法

<div class="blog-post-meta">
  <span class="blog-meta-chip">
    <span>
      <span class="blog-meta-label">Published</span>
      <span class="blog-meta-value">2026-03-26</span>
    </span>
  </span>
  <a class="blog-author-card" href="https://github.com/ubugeeei">
    <img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
    <span class="blog-author-text">
      <span class="blog-meta-label">Author</span>
      <span class="blog-meta-value">ubugeeei</span>
    </span>
  </a>
</div>

今天我要打开`oxlint-plugin-vize`的第一个 alpha 版本，一个新的 Oxlint JS 插件桥接器，用于 Vize Patina。

目标很简单：保持 [Oxlint](https://oxc.rs/docs/guide/usage/linter) 作为 JavaScript 和 TypeScript 规则的主运行工具，同时让 Vize 在同一运行中贡献 Vue 特定的诊断。这个阿尔法不是在奥克斯林特和帕蒂娜之间做选择，而是让他们合作。

## 这是什么

`oxlint-plugin-vize`让Oxlint通过Vize的原生绑定执行Patina，同时仍使用Oxlint的JS插件模型和规则配置。

这意味着单`.oxlintrc.json`可以混合规则，比如：

- Oxlint 核心规则，如 `no-console`
- Oxlint 内置的 `vue` 插件
- Vize规则，如`vize/vue/require-v-for-key`
- 带包绿支持的Vue诊断，如`vize/vue/no-v-html`和`vize/vue/no-duplicate-attributes`

插件使用`vize`命名空间，并读取`settings.vize`设置。

## 为什么这个阿尔法很重要

Patina 已经很熟悉 Vue 模板，但许多团队希望 Oxlint 继续成为他们 lint 工作流程的核心。

这个阿尔法是朝向那个形状的第一步：

- 一个lint命令
- 一个配置文件
- 一个输出流
- Rust原生的JavaScript和TypeScript规则，以及Vue模板感知的诊断工具

对于Vue项目来说，这种组合很重要。模板规则如缺少`v-for`键或不安全的`v-html`使用，应能与通用的Oxlint规则并存，而不必要求单独的lint通行和独立的报告格式。

## 配置示例

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "locale": "en",
      "helpLevel": "none"
    }
  },
  "rules": {
    "no-console": "warn",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn",
    "vize/vue/no-duplicate-attributes": "error"
  }
}
```

该阿尔法目前支持：

- `settings.vize.locale` 用于诊断语言
- `settings.vize.helpLevel` `"full"`、`"short"`或`"none"`
- `showHelp` 用于向后兼容
- `settings.patina`作为兼容别名，而`settings.vize`成为典范密钥

## 工作原理

桥梁设计基于Oxlint的按规则执行模型，而非与之抗衡。

- 文件中第一个启用的 Vize 规则仅对该规则运行原生 Patina 通行。
- 如果同一文件启用了第二个 Vize 规则，插件升级为共享的全文件 Patina 通道，并将结果用于剩余的 Vize 规则。
- 文件内容和规则结果在 Oxlint 进程的生命周期内按文件和设置缓存。

这种设计既便宜地保留了第一条规则，又避免了多个Vice规则激活后重复的原生工作。

## 诊断与输出

这种集成中一个难点是位置报告。

Oxlint 的 JS 插件系统目前可从提取的 Vue 脚本程序运行，而许多 Patina 诊断则源自 `<template>` 或其他 SFC 模块。在这个 alpha 版本中，`oxlint-plugin-vize` 在诊断信息中保持真实的 Vue 块和 `line:column` 在线，这样输出仍然会指向 SFC 的正确位置。

该仓库还包含一个小型`examples/oxlint-vize`示例，用于展示来自以下混合输出：

- Oxlint 核心诊断
- Oxlint 内置的 Vue 支持
- 带铜绿的Vize诊断

## 当前限制

这仍处于alpha阶段，有几个限制需要明确指出：

- Oxlint JS 插件目前依赖提取后的 Vue 脚本程序，因此没有 `<script>` 或 `<script setup>` 的文件尚未调用该插件。
- 当 Oxlint 无法直接接受原始模板范围时，诊断锚仍然指向脚本程序。
- 初始的 alpha 封装针对节点 24+;当前版本支持 Node 22 和 Node 24+。
- Oxlint 的 JS 插件支持本身仍在不断发展，因此这里的一些粗糙边缘是上游约束，而非仅 Vize 的行为。

## 为什么是现在的阿尔法

我想尽早把这个整合交到大家手里，甚至在每个边缘情况都还没完善之前。

核心形状已经感觉很实用：

- Vize带来Vue特有的绒毛智能
- 奥克斯林特依然是顶级跑者
- 配置表面保持较小
- 性能模型保持原生优先

这足以让Vue用户获得真实反馈，他们希望更快的棉絮堆栈，同时又不放弃模板识别检查。

## 接下来会发生什么

接下来的步骤很简单：

- 随着 Oxlint 提供更多 Vue 识别插件钩子，改进模板位置映射
- 围绕平台原生绑定加固安装和发布流程
- 扩展文档和示例，用于真实项目设置
- 不断优化Oxlint格式化器中Patina帮助文本的呈现方式

这个阿尔法不是最终状态。它是Oxlint和Vize的Vue绒毛连接的第一个可用桥梁，我很期待它接下来的发展。
