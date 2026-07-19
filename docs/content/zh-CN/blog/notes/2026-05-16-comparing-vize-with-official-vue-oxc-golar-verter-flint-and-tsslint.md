---
title: 工具比较
description: Vise及附近项目在官方Vue工具、Oxc、Golar、Verter、Flint和TSSLint的实际比较。
---

<!-- Generated translation; source: blog/notes/2026-05-16-comparing-vize-with-official-vue-oxc-golar-verter-flint-and-tsslint.md -->

# 工具比较

<div class="blog-post-meta">
  <span class="blog-meta-chip">
    <span>
      <span class="blog-meta-label">Published</span>
      <span class="blog-meta-value">2026-05-16</span>
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

Vize与多个项目关系密切，因此不可避免地会被比较。

这种比较是有用的，但前提是轴线清晰。“更快”是不够的。“生锈”是不够的。“Vue支持”是不够的。

真正的问题是：**每个项目想要拥有哪个层？**

![显示 Vize 在附近模具景观中的关系图，包含仅参考组、相邻平台组、Vize 使用组和仅比较组](/blog/vize-toolchain-map.svg)

## 快速地图

| 项目         | 重心                                           | Vize与其的关系                                                |
| ------------ | ---------------------------------------------- | ------------------------------------------------------------- | ------------------------- |
| 官方Vue工具  | Vue编译器和语言工具的生产基线                  | Vize是独立且实验性的，因此必须将其视为参考点                  |
| Oxc / Oxlint | 通用JavaScript和TypeScript基础设施             | Vize可以在拥有Vue特有语义                                     | 的同时，重用并与Oxc协作。 |
| 戈拉尔       | 基于`typescript-go`的嵌入语言类型检查          | Vize 的工具链范围比单纯的类型检查                             |
| 维尔特       | 替代下一代Vue编译器和工具链                    | 雄心最接近，架构和产品形态各异                                |
| 弗林特       | 友好的、带有强默认值的 JS/TS 字样              | 补充一般TS线条，不是Vue SFC工具链                             |
| TSSLint      | 语言服务器内的TypeScript原生linting（linting） | 这是一个强烈的语义填充想法，但不是完整的Vue编译器/打印/画廊栈 |

## 官方Vue模具

官方堆叠最重要。

[Vue语言工具](https://github.com/vuejs/language-tools)、`vue-tsc`、Vue编译器包以及官方编辑器集成是生产基础。当Vize不同意官方行为时，这种分歧并不自动成为一个大胆的新想法。大多数时候，这是必要的修复、实现不完整，或者 Vize 需要更清晰的兼容性故事。

这并不意味着Vize毫无意义。

它定义了合同。

Vize可以尝试更统一的Rust原生架构，但它仍然需要关注真实Vue代码的形状、真实的编译器输出、真实的诊断和真实编辑器的期望。官方堆栈是让实验保持诚信的参考点。

## 奥克斯和奥克斯林特

[奥克森](https://oxc.rs/)是一个通用的JavaScript和TypeScript编译器基础设施项目。[牛茚](https://oxc.rs/docs/guide/usage/linter.html)是建立在那个世界之上的高性能衬板。

Vize不应在JavaScript和TypeScript层与Oxc竞争。那样太浪费了。Oxc 已经为生态系统提供了快速的解析器、语义基础设施、格式化方向、线条方向以及不断增长的共享原语。

Vize的问题更狭窄，也更针对Vue：

- `.vue`档案整体是什么？
- 模板作用域如何连接到脚本绑定？
- 指令、槽、道具、发射、样式块和编译器输出之间是如何关联的？
- 我们如何将诊断数据映射到人类编辑的确切来源？
- 这些语义如何为编译、lint、格式化、类型检查、LSP、Musea和AI工作流程提供信息？

OXC可以作为JS/TS的通用基础。Vize可以是Vue专用的工具链，基于这个基础，而不是把Vue扁平成“仅仅是脚本块”。

## 戈拉尔

[戈拉尔](https://github.com/auvred/golar)之所以有趣，是因为它对嵌入式语言`typescript-go`非常重视。

其核心是类型检查、虚拟代码和`tsgo`集成。对于Vue来说，这自然使其更接近官方语言核心模型。这是一个既实用又好的方式：重用Vue的虚拟代码机制，让TypeScript引擎更快或更灵活。

Vize试图解决更广泛的问题。

类型检查层很重要，但不是整个项目。Vize希望解析器、语义模型、编译器、linter、格式化器、原生类型检查路径、LSP、组件库和面向AI的表面都能共享更多相同的Vue感知核心。

所以区别不是“Golar是类型检查，Vize是更快的类型检查”。

区别在于：

- Golar 主要是一个嵌入语言 TypeScript 处理故事。
- Vize 是一个完整的 Vue 工具链故事，类型检查是 Vue 分析模型的一个使用者。

## 维尔特

[维尔特](https://github.com/pikax/verter)可能是哲学上最接近的比较。

它也提出了一个重大问题：如果我们愿意重新思考层次，下一代Vue工具链会是什么样子？

这和Vize的问题很接近。这两个项目都关注编译器行为、语言工具、诊断，以及比一堆无关插件更严格的体验。

区别在于重点：

- Verter从一开始就显得更为严格且以语言服务为导向。
- Vize强调在编译、lint、格式化、检查、LSP、Musea和AI工作流程中采用Rust原生共享核心。
- Vize还将组件库和设计系统工具视为前端环境的一流组成部分，而非单独的文档附带。

我不把维尔特当作敌人。这是又一次严肃的实验，空间值得多次尝试。

## 弗林特

[弗林特](https://www.flint.fyi/)是另一种比较。

它是一个基于JavaScript和TypeScript的linter，强调有用的默认值、缓存和类型化linting。这很有价值，因为JS/TS生态系统面临一个真实问题：仅语法的linting虽然快速但不完整，而语义linting则可能变得缓慢且操作成本高昂。

Vize同意语义反馈应当实用、快速且愉快的前提。

但Flint并不打算成为Vue SFC编译器、格式化器、模板分析器、组件库或Vue专用LSP。更准确地说，它是一种高质量的通用线条方向。

互补形状为：

- Flint 可以推动 JS/TS 的绒毛体验。
- Vize可以推动Vue专用分析。
- 良好的前端环境应当让这些层协作，而不是强迫每个工具都拥有所有关注点。

## TSSLint

[咂咂](https://marketplace.visualstudio.com/items?itemName=johnsoncodehk.vscode-tsslint)之所以重要，是因为它将TypeScript语义线条视为可以靠近TypeScript语言服务器的存在。

这个想法很有说服力：如果TypeScript检查器已经开启了一个项目，为什么还要在另一个linter流程中重建世界，仅仅是为了回答语义问题？

Vize也有类似的直觉，但他们把Vue看作一个多语言的产物。

对于Vize来说，问题不仅仅是“lint 规则能否重用 TypeScript 状态？”它如下：

- 模板分析能否重复使用与编译器相同的Vue语义模型？
- 类型感知的Vue lint规则能否在不支付全部重建费用的情况下提出有针对性的问题？
- 编辑器诊断、批次检查和AI修复循环能否一致使用相同的源映射？
- 系统能否让项目会话存活足够长的时间以摊销工作？

TSSLint 是一个强烈信号，表明语义线条正逐渐接近现有语言状态。Vize将这种本能扩展到Vue的专属结构中。

## 维兹想要拥有的东西

维兹不应该拥有一切。

它应拥有那些Vue特定知识必须连贯的领域：

- SFC 解析与块结构
- 模板语义学
- 指令与组成分析
- 编译器输出决策
- Vue感知绒毛检测
- 将生成的伪影映射回 `.vue`
- Musea 组件元数据
- 用于AI工作流程的机器可读诊断

它应在其他方面合作：

- 在可能的情况下使用 Oxc 进行 JavaScript 和 TypeScript 解析
- 将行为与官方 Vue 工具进行比较
- 向Golar、TSSLint和Flint学习类型感知反馈循环
- 关注Verter作为另一个完整工具链实验

## 产品立场

最干净的定位是：

> Vize 是一个独立的、实验性的、原生于 Rust 的 Vue 工具链，试图让编译器、合成器、格式化器、类型检查器、LSP、组件库和面向 AI 的诊断工具感觉像一个连贯的环境。

这意味着Vize并不是官方的答案。

这是一个高速实验的答案。

现在的工作是让这个答案在实际项目中有用，缩小与官方行为的差距，并保持架构足够锋利，使实验值得进行。
