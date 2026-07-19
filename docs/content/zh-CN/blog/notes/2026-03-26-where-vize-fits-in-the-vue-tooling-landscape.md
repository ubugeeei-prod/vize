---
title: Vue 工具地图
description: 这是一张显示 Vize 在当前 Vue 工具格局中的位置，以及它与邻近项目的不同之处的地图。
---

<!-- Generated translation; source: blog/notes/2026-03-26-where-vize-fits-in-the-vue-tooling-landscape.md -->

# Vue 工具地图

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

Vize 容易被误解的一个原因是它与人们已经熟悉的多个工具重叠，但并不总是在同一层。

其中一些项目是官方的。有些则是框架无关的。有些是编辑优先的。有些是编译器优先的。有些主要是类型检查。有些则试图成为完整的工具链。

所以最有用的问题不是“哪个更好？”问题是：**每种工具到底想解决什么问题？**

## 简短版

以下是最快的摆放方式：

| 项目            | 主要重心                                                | 它不是                             |
| --------------- | ------------------------------------------------------- | ---------------------------------- |
| **Vize**        | Rust 中独立的完整 Vue 工具链                            | 不是官方的Vue编辑器栈              |
| **Vue语言工具** | 官方Vue编辑器 + 类型检查工具                            | 不是完整的编译器/合成器/格式工具链 |
| **戈拉尔**      | 基于`typescript-go`的嵌入语言类型检查框架               | 不是Vue专用的完整工具链            |
| **维尔特**      | 替代完整 Vue 编译器 + LSP + 构建工具链                  | 不是官方的Vue工具链                |
| **Vite+**       | 跨运行时的统一网页开发入口，包管理，开发/构建/检查/测试 | 不是Vue专用的编译器或附录器        |
| **奥克斯林特**  | 高性能JS/TS衬垫                                         | 不是单独支持Vue模板的全绒毛堆栈    |

如果你把那张表格放在脑海里，大部分困惑都会消失。

## 维泽

Vize最好被理解为**独立的、完整的Vue工具链，在Rust中**。

其目标广泛：

- 编译Vue SFC
- lint Vue 特定图案
- 格式化Vue文件
- 类型检查Vue模板和脚本绑定
- 给LSP供电
- 提供组件画廊
- 将Vue支持的工具暴露给AI工作流程

这种广度正是 Vize 与本次比较中大多数项目的不同之处。它不仅仅是编辑器集成，不仅仅是一个类型检查器，也不仅仅是一个捆绑插件。它试图成为一个连贯的 Vue 原生工具链，拥有一个架构中心。

这也是为什么最近的字体检查方向很重要。Vize不仅仅是想“让事情更快”`vue-tsc`。目前的方向是将Vue支持的虚拟文件生成、诊断映射和面向编辑器的信息保留在`vize_canon`中，原生项目会话由[`corsa-bind`](https://github.com/ubugeeei/corsa-bind)驱动。

## 维兹正接近`tsgo`

最近的一则笔记[`corsa-bind: The Idea of Language Processor Orchestration`](https://wtrclred.io/posts/17)认为，有趣的部分不仅在于更快的执行，还在于“改变工作的形态，而不是编译器”。

这种框架非常接近Vize对`tsgo`的态度。

Vize并不打算把`tsgo`变成整个产品故事，也不会把它当作一次性的CLI来对待，每个功能都会重复运行。该方向更接近将TypeScript处理视为更广泛Vue工具链中的可重用原生服务：

- `vize check`实现一个支持Vue的虚拟TypeScript项目，打开Corsa项目会话，并批量请求诊断。
- `vize_maestro`可以保留 Corsa 桥，用于悬停、补全、定义、引用和重命名，当本地类型检查启用时。
- `vize_patina` 使用懒惰的原生 Corsa 会话来处理类型感知的 lint 规则，只探测所需的类型，而不是重建 JavaScript 托管栈中的所有类型。
- `vize_canon` 保留 Vue 专用虚拟文件生成和源映射的所有权，而 `corsa-bind` 和 `tsgo` 则回答 TypeScript 方面的问题。

所以Vize的《`tsgo`故事》不仅仅是“用更快的二进制替换`vue-tsc`”。它更接近于围绕常驻 TypeScript 处理器构建一个 Vue 原生控制层，然后在批处理检查、编辑器功能和类型识别线条间重复使用该层。

## Vize 与 Vue 语言工具

官方的[Vue语言工具](https://github.com/vuejs/language-tools)项目是生产准备的Vue编辑器和类型检查栈。包括：

- **Vue（官方）**VS Code 扩展
- `vue-tsc`
- `@vue/language-server`
- `@vue/language-core`

这个技术栈的核心是**语言工具**：编辑器支持、类型检查、虚拟代码生成以及集成，让Vue在集成开发环境中显得一流。

Vize和那个世界有重叠，因为Vize还有类型检查器和LSP。但Vize正试图覆盖更多内容：

- Vize包含自身的编译器目标
- Vize包含线条和格式化的目标
- Vize包含产品表面，如Musea和MCP工具
- Vize是以Rust为先，而非TypeScript

所以最简单的区别是：

- **Vue语言工具**是Vue的官方编辑器和类型检查基础
- **Vize**是一个独立尝试，旨在将更多 Vue 工具链统一到一个 Rust 架构下

如果你现在的首要任务是支持生产环境的编辑器，官方的Vue协议栈是基础。如果你感兴趣的是更广泛、实验性的、基于Rust原生的Vue工具链，那Vice就开始有意义了。

## 维泽对阵戈拉尔

[戈拉尔](https://github.com/auvred/golar)并不是真正意义上的“另一个Vue工具链”。

Golar 自称为基于`typescript-go`的嵌入式语言框架。对于 Vue 来说，它重用了官方的`@vue/language-core`机制，并专注于让基于扩展的语言如 `.vue`、`.astro` 和 `.svelte` 能够与 `tsgo` 兼容。

这意味着戈拉尔的重心为：

- CLI 类型检查
- 声明发射
- `tsgo` 嵌入语言的集成
- 虚拟代码生成插件基础设施

Vize在两个重要方面有所不同：

1.**范围**

Golar 主要是一个关于 `typescript-go` 的类型检查和虚拟代码故事。
Vize正试图拥有Vue工具链中更大的一块：编译器、linter、格式化器、类型检查器、LSP、画廊等等。

2.**Vue层的所有权**

Golar 有意重用官方 Vue 工具来生成 Vue 代码。
Vize正试图在Rust中构建更多Vue专属的技术栈。

实际执行层的差异也开始显现出来。Golar 与嵌入式语言的`typescript-go`集成密切相关。Vize目前的原生类型检查路径正围绕`vize_canon`加`corsa-bind`构建，这使得问题不再是“如何用更快的TS引擎重用官方技术栈？”，而是“Vue工具链有多少能存在于一个原生架构中？”

所以 Golar 更像是“让`tsgo`在嵌入式语言上运行良好”，而 Vize 更接近“构建一个端到端的原生 Vue 工具链”。

## 维泽 vs 维尔特

[维尔特](https://github.com/pikax/verter)可能是这份名单中最接近的哲学邻居。

和维兹一样，维尔特的目标很高。其公开愿景是一个混合 Rust + TypeScript Vue 编译器、LSP、构建工具、linter 以及更广泛的工具链。这让它与Vize属于同一类：雄心勃勃、全栈，并且愿意重新思考Vue工具链，而不是只修补一层。

这时差异更多体现在产品形态和架构上，而非类别：

- **Verter**自称为严格优先的 Vue 语言和编译器工具链，拥有强大的 VS Code 和 TS 提供者故事。
- **Vize**自詡为一个独立的高性能 Vue 工具链，拥有统一的 CLI、Vite 集成、Musea，以及更强的“一个解析器/一个 AST / 一个工具链”叙事。

在强调上也有差异：

- Verter 突出显示类型化的 TSX 生成、类型提供者后端（如 TSGO / tsserver）以及广泛的内置 lint 规则目录。
- Vize 展示了统一的 Rust 原生工具链，涵盖编译、lint、格式化、类型检查、编辑器工具、组件库和 AI 集成，同时明确定位为 Vite+ 和 Oxlint 等生态系统工具的补充。

所以我不会把维尔特描述为“同一个东西只是名字不同”。更准确地说，它是\*\*对“如果我们重新开始，下一代Vue工具链会是什么样子？”这个问题的又一个严肃回答。

## Vize vs Vite+

[Vite+](https://viteplus.dev/)处于另一层。

Vite+ 是更广泛的网页开发统一切入点。它的任务是在一个工作流程中管理运行时设置、包管理、开发、检查、测试、构建、打包和单仓库任务执行。它整合了 Vite、Vitest、Oxlint、Oxfmt、Rolldown、tsdown 及相关工具。

那就是Vite+：

- **框架无关性**
- 以工作流程为导向
- 比Vue更广泛

Vize 不同之处在于它是**Vue 特定**的。

Vite+ 并不试图成为 Vue 编译器或模板 linter。它为你提供了一个统一的网页工具链入口。
Vize可以接入那个世界。事实上，该仓库已经使用 Vite+ 进行工作区编排。

所以这其实不是真正的比赛：

- **Vite+**= 通用的网页工具链壳
- **Vize**= Vue专用引擎，可以存在于该壳体内

## 维泽 vs 奥克斯林特

[牛茚](https://oxc.rs/docs/guide/usage/linter)也处于不同的层次。

Oxlint 是 Oxc 生态系统中高性能的 JavaScript 和 TypeScript 附加工具。它在通用的 JS/TS 规则和日益注重类型感知的工作流程方面表现出色，但单独并不打算取代所有 Vue 模板识别诊断。

这正是维泽·帕蒂纳发挥作用的地方。

Patina 关注的是 Vue 特有的绒毛问题，例如：

- 模板指令
- SFC结构
- 组成约定
- Vue模板中的无障碍检查

所以区别很简单：

- **Oxlint**处理通用的 JS/TS 绒毛处理
- **Vize / Patina**处理 Vue 特有的绒毛

新的`oxlint-plugin-vize` α之所以存在，正是因为这两者是互补的，而非冗余的。

## 那么维泽坐哪儿？

维泽位于多个范畴的重叠中，但无法归约为其中任何一个范畴。

它如下：

- 比官方 Vue 语言工具更广泛
- 比 Golar `tsgo` 加速项目更广泛的项目
- 在雄心上最接近于Verter等其他全栈项目
- 补充通用工作流程工具如Vite+
- 与通用的JS/TS溶机如Oxlint的补充

如果要用一句话来总结：

> Vize是一个独立的Rust原生尝试，旨在统一Vue工具链中比官方语言工具覆盖更多的部分，同时仍与更广泛的生态系统工具合作，而非取代它们。

## 你应该抓住哪一个？

这取决于你想要什么：

- 如果你想要官方的、适合生产环境的编辑器和类型检查栈，请选择**Vue语言工具**。
- 如果你主要兴趣是基于`typescript-go`的类型检查嵌入语言，同时重复使用官方语言工具，可以参考**Golar**。
- 如果你想要另一个雄心勃勃的全栈Vue工具链，拥有严格的类型和逻辑性算法故事，可以看看**Verter**。
- 如果你想要一个统一的通用工作流程入口点用于网页开发，可以使用**Vite+**。
- 如果你需要高性能 JavaScript 和 TypeScript 打印，可以使用**Oxlint**。
- 如果你感兴趣的是建立一个更广泛的Rust原生Vue工具链，试图让编译器、打印、格式化、类型检查、编辑器工具、画廊工具和AI工具感觉像一个系统，可以使用**Vize**。

这才是真正的区别。
