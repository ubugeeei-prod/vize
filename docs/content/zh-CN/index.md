---
layout: entry
title: 维泽
description: Rust中的高性能Vue.js工具链。编译、去除、格式化、类型检查并探索Vue组件。
hero:
  name: Vize
  text: Rust 中的高性能Vue.js工具链
  tagline: /viːz/ —— 一个能看穿你代码的智慧工具。编译、剥离、格式化、类型检查和探索Vue组件——所有这些都由Rust驱动。⚠️ 还没准备好量产。
  image:
    src: logo.svg
    alt: Vize标志
  actions:
    - theme: brand
      text: 开始
      link: zh-CN/getting-started.md
    - theme: alt
      text: GitHub
      link: https://github.com/ubugeeei-prod/vize
    - theme: alt
      text: 游乐场
      link: https://vizejs.dev/play
features:
  - title: Vite 插件
    details: 从推荐的 Vue 应用集成开始：在 Vite 内部原生编译 SFC 并共享 Vize 配置。
    link: zh-CN/guide/vite-plugin.md
  - title: 静态分析流水线
    details: 解析器、语义分析、lint 规则、虚拟 TypeScript、跨文件检查和编辑器诊断共享相同的 Rust 原生分析层。
    link: zh-CN/guide/static-analysis.md
  - title: 规则文档
    details: 浏览concrete Vue、HTML、SSR、Vapor、Musea、类型感知和跨文件诊断，包含好坏的和好的例子。
    link: zh-CN/rules/index.md
  - title: 共享配置
    details: 配置编译器选项、Vite 扫描、lint 预设、类型检查、格式化、LSP 功能和 Musea from from `vize.config.*`。
    link: zh-CN/guide/configuration.md
  - title: 原生类型检查
    details: "`vize:check`包脚本通过`vize_canon`和 Corsa 项目会话运行，由 `corsa-bind` 支持，使 Vue 识别的诊断保持在本地路径上。"
    link: zh-CN/guide/static-analysis.md
  - title: 包脚本和CLI引用
    details: 应用工作流程使用项目脚本中的 npm 包，Rust CLI 文档用于 LSP、配置文件和直接二进制使用。
    link: zh-CN/guide/cli.md
  - title: 编译器检查器
    details: 检查Vue输出、Vice输出、虚拟TS、VIR和跨文件图表，然后分享永久链接的复制品或代理报告。
    link: zh-CN/guide/compiler-inspector.md
  - title: Oxlint 插件
    details: 在 Oxlint 中运行 Vue 的诊断，并一次性将 OXC 的 JS 和 TS 规则结合起来。
    link: zh-CN/guide/oxlint.md
  - title: 实验性捆绑器集成
    details: 存在rollup、webpack、esbuild以及专用的Rspack路径，但Vite仍然是推荐且最稳定的集成。
    link: zh-CN/guide/unplugin.md
  - title: 8.3倍快
    details: 多线程编译15,000个SFC文件（36.9MB），速度不到500毫秒。场地分配，人造丝并行性，零GC。
    link: zh-CN/architecture/performance.md
  - title: 组件画廊
    details: Musea — 艺术文件、文档、调色板生成、a11y 和 VRT 工具，画廊工作流程由 @vizejs/vite-plugin-musa 提供。
    link: zh-CN/guide/musea.md
  - title: WASM绑定
    details: 直接在浏览器中运行 Vue 编译器，配合 WebAssembly。动力游乐场、文档和教育工具。
    link: zh-CN/guide/wasm.md
  - title: 人工智能集成
    details: MCP服务器使AI助手能够通过Musea理解并操作你的Vue组件。
    link: zh-CN/integrations/mcp.md
  - title: 蒸汽模式
    details: 一流支持 Vue 3.6 Vapor 模式——无虚拟 DOM 的细粒度反应式编译。
    link: zh-CN/architecture/overview.md
  - title: 理念
    details: 艺术启发的建筑、氧化生态系统（OXC、oxlint、corsa-bind）以及统一的工具链愿景。
    link: zh-CN/philosophy.md
  - title: 博客
    details: 发布内容说明，包括已发布变更的说明，以及不定期的设计更新、开发日志和项目思考笔记。
    link: zh-CN/blog/index.md
---

<!-- Generated translation; source: index.md -->

## 当前方向

Vize最近最大的转变之一是原生类型检查。`vize check`命令由
NPM 包脚本和面向编辑器的类型检查流水线正在转向 `vize_canon` Plus
[`corsa-bind`](https://github.com/ubugeeei/corsa-bind)，这使得 Vize 能够保留虚拟文件和
TypeScript项目诊断在原生路径上运行更长时间。

这不仅仅影响纯粹的速度。它为 Vize 提供了模板分析、诊断、导航和未来编辑器功能之间的更紧密循环，同时减少了通过 JavaScript 托管编译流程回馈的工作量。忠诚的报道仍在追赶中，但这显然是工具链的发展方向。

同样的做法也适用于绒毛和棉花。静态分析从解析器和Croquis开始
语义模型，然后输入Patina lint 规则、Canon 虚拟TypeScript、编译器决策、编辑器
诊断和组件画廊元数据。实际工作流程有详细文档
[静态分析](./guide/static-analysis.md)，配置细节在
[配置](./guide/configuration.md)。具体规则和诊断目录已发布
[规则](./rules/index.md)。

## 作者

![乌布吉埃伊](https://github.com/ubugeeei.png)

- \*[ubugeeei](https://github.com/ubugeeei)\*\*是一位常驻东京的软件工程师，专注于Vue、Rust、设计和语言工具领域。

他是 [Vue.js 核心团队](https://vuejs.org/about/team.html)成员、[Vue.js 日本用户组](https://github.com/vuejs-jp)核心成员、[Vite+](https://github.com/voidzero-dev/vite-plus)核心贡献者，以及 [mates-dev](https://github.com/mates-dev)的首席工程师。

他还是[chibivue](https://github.com/chibivue-land/chibivue)、[Vize](https://github.com/ubugeeei-prod/vize)和[Ox Content](https://github.com/ubugeeei/ox-content)的创作者。

- GitHub：[github.com/ubugeeei](https://github.com/ubugeeei)
- X（推特）：[@ubugeeei](https://x.com/ubugeeei)
- 博客：[wtrclred.io](https://wtrclred.io)
- chibivue.land：[chibivue.land](https://chibivue.land)

## 赞助商

Vize 是一个免费且开源的项目，授权在 MIT 下。开发和维护完整的工具链——编译器、linter、格式化器、类型检查器、LSP、组件库和WASM绑定——是一项需要持续专注和奉献的重要工作。

如果 Vize 帮您节省时间、提升开发体验，或者您相信高性能Vue.js工具链的愿景，请考虑赞助该项目：

- CI/CD运行器基础设施由[Blacksmith](https://www.blacksmith.sh/)赞助。
- [GitHub 赞助商](https://github.com/sponsors/ubugeeei)

您的支持有助于资助持续的发展、基础设施费用，并确保Vize对所有人免费开放。每一份贡献——无论大小——都会产生真正的影响。
