---
title: 性能
---

<!-- Generated translation; source: architecture/performance.md -->

# 表演

> **⚠️ 正在开发中：**Vize正在积极开发中，尚未准备好投入生产使用。基准数据来自开发版本，可能会有所变化。

Vize通过利用Rust的零成本抽象和原生多线程，相比标准基于JavaScript的Vue编译器实现了显著的性能提升。速度不是可有可无的——它是开发者经验的前提条件。

<!-- benchmark:environment:start -->

## 基准环境

README 与所有语言页面均由同一个已提交的产物生成。文件数量按行标明。

| 运行环境                                 | 测量时间                 | 测量次数 | 预热次数 |
| ---------------------------------------- | ------------------------ | -------- | -------- |
| `blacksmith-32vcpu-ubuntu-2404` (32 CPU) | 2026-09-16T08:19:25.262Z | 3        | 1        |

[Actions](https://github.com/ubugeeei-prod/vize/actions/runs/35072615669) · [f9cc50ee2e2f](https://github.com/ubugeeei-prod/vize/commit/f9cc50ee2e2ff14cd120241cd87a0162446df65b) · [JSON](https://github.com/ubugeeei-prod/vize/blob/main/tools/benchmarks/results/tool-benchmark-latest.json)

版本: vize: `vize 0.424.11` · tsgo: `Version 7.0.2` · vueTsc: `Version 6.0.3` · verterTsc: `verter-tsc 0.0.1-beta.3` · vue: `3.6.0-beta.10` · node: `v24.14.0`.

下方本地 Linter、Formatter 和实现分析属于历史测量，并非此快照的结果。记录的环境为 MacBook Pro M2 Max（12 核、96 GB RAM）、macOS 15.3.2、Node v24.14.0、Vite v8.0.0 和 Vue v3.6.0-beta.10。

## 基准：SFC 编译

文件数: **3,000** (12,302,100 bytes).

|               | @vue/compiler-sfc | Vize    | 速度比    |
| ------------- | ----------------- | ------- | --------- |
| 1T            | 3.57s             | 835.3ms | **4.3x**  |
| workers / max | 1.61s             | 65.0ms  | **24.7x** |
| 1T / max      | 3.57s             | 65.0ms  | **54.8x** |

[完整方法与验证结果](./performance-blacksmith)

<!-- benchmark:environment:end -->

## 为什么是 Rust？

### 零成本抽象

Rust 的所有权模型消除了垃圾回收暂停。模板 AST 节点存放在每次编译一个的 arena（`vize_carton`）中，并从模板源码借用文本，因此节点是纯数据，自身不持有任何堆分配（`crates/vize_relief/src/relief/elements.rs`）。这意味着：

- **无 GC 暂停** — 在基于 V8 的编译器中，垃圾回收可能导致不可预测的延迟尖峰。Vize 没有 GC 开销。
- **无 JIT 预热** — V8 的 JIT 编译器需要时间来优化热路径。Vize 从第一条指令开始就全速运行。
- **可预测的性能** — Rust 的提前编译意味着性能在不同运行间保持一致，不依赖于 V8 的优化启发式。

### 原生多线程

Vize 使用 [Rayon](https://docs.rs/rayon) 进行数据并行编译。每个 SFC 文件都独立编译，使工作负载天然并行；`crates/vize/src/commands/build/runner.rs` 中的批处理驱动器将输入分发到线程池：

```rust
// crates/vize/src/commands/build/runner.rs — 批处理驱动器
planned_inputs
    .par_iter()
    .map(|input| compile_file_with_profile(&input.source, compile_settings, &stats))
    .collect()
```

arena 并不在这里创建。它在诞生之处获取 —— `vize_atelier_sfc` 内部的模板、脚本和样式入口 —— 来自每个工作线程各自的池：

```rust
// 例如 crates/vize_atelier_sfc/src/compile.rs
let allocator = vize_carton::pool::acquire();
```

工作窃取方式意味着，如果某个文件明显大于其他文件，空闲线程会从繁忙线程的队列中窃取工作，保持近乎完美的负载均衡。

### 高效内存布局

Rust 的结构体布局和枚举判别值都很紧凑。`vize_relief` 中的 AST 表示对缓存友好，减少了内存带宽瓶颈：

- **单字节判别值** — `NodeType` 是带 27 个变体的 `#[repr(u8)]`（`crates/vize_relief/src/relief/core.rs`），因此节点的类别只占一个字节，而非堆分配的字符串。
- **固定的节点大小** — 每个模板节点都带有 `const` 大小断言，因此让节点变大的字段会让构建失败，而不是让预算失守。`ElementNode` 为 104 字节，`SimpleExpressionNode` 为 88，`AttributeNode` 为 56，`TextNode` 为 24，`SourceLocation` 为 8（`crates/vize_relief/src/relief/{elements,expressions,control_flow,nodes}.rs`）。
- **无对象头** — 与携带原型链、属性映射和隐藏类指针的 JavaScript 对象不同，Rust 结构体是零开销的纯数据。

### 无运行时开销

与运行在 V8 上的基于 JavaScript 的编译器不同，Vize 直接编译为原生代码。没有 JIT 预热，没有垃圾回收器，也没有事件循环争用。CLI 按平台以自包含的原生可执行文件分发：在 musl Linux 目标上是完全静态链接的，并由 CI 验证（`tools/commands/ci/github/verify-musl-cli-binary.rs`）；在 glibc、macOS 和 Windows 目标上则动态链接到系统 C 库。Vite 插件把同一个编译器作为原生 Node 插件（`@vizejs/native`）加载，而不是作为独立进程。

## 性能架构选择

### Arena 分配

`vize_carton::Allocator` 是用于 AST 节点的 bump 分配器，它封装了 [`oxc_allocator`](https://docs.rs/oxc_allocator)，使模板节点与保留的 JavaScript 表达式共享同一个 arena 和同一个生命周期（`crates/vize_carton/src/allocator.rs`）。这意味着：

- **分配是 O(1)** — 只需把指针向前推进。没有空闲链表遍历，没有碎片管理。
- **回收是 O(1) 并且被复用** — 编译结束时 arena 是被 `reset()` 而不是被丢弃：bump 指针回到块的起点，arena 回到每个工作线程各自的空闲列表（`crates/vize_carton/src/pool.rs`，每个工作线程最多保留 4 个空闲 arena）。下一个文件复用同一块内存，而不是向操作系统再要一份。
- **内存局部性极佳** — 节点在内存中连续排布，最大化树遍历时的 L1/L2 缓存命中率。

arena 中的值不得比它所属的那次编译活得更久。该契约由编译器强制（`reset` 接受 `&mut self`，池的守卫拥有自己的 arena），并且在调试构建中还有一个代际标记：若某个值在其 arena 被回收后仍被读取，就会 panic（`crates/vize_carton/src/allocator/generation.rs`）。

AST 中没有任何类型实现 `Drop` —— arena 的容器类型会拒绝需要析构的载荷，因此这是编译错误，而不是一条约定。

### 单遍分词器

`vize_armature` 的分词器是作用于 `&[u8]` 的面向字节的状态机（`crates/vize_armature/src/tokenizer.rs`）。它从不实体化 token：整个编译器里既没有 `Token` 类型，也没有 token 向量。相反，`tokenize()` 一次扫描到输入末尾，并把事件推送给由解析器实现的 `Callbacks` 接收器 —— 每个事件在产生时同步处理，两阶段设计所需的中间数组根本不存在。

请注意这是推送式的，而非惰性拉取：解析器不会请求 token，也无法中途停止该循环。

### 字符串内联

在一次编译中反复出现的名称 —— 规范化的指令名、资源名、驼峰化的参数名 —— 会被 `vize_carton::interner` 内联为 arena 支撑的原子；一个编译期 [`phf`](https://docs.rs/phf) 集合收录了 181 个常见名称（HTML/SVG/MathML 标签、Vue 内置组件、指令名，以及转换特殊处理的属性），它们会解析为 `'static` 字面量，完全不触及 arena。这意味着：

- 重复出现的计算名称共享一次 arena 分配
- 对已知名称的查找是编译期完美哈希，不涉及分配

内联是回退路径，而不是常见情况。大多数名称根本不会被复制：标签名、属性名以及大部分表达式内容都是直接从模板源码借用的 `&'a str` 切片，因此常见路径不做任何分配（逐字段的策略记录在 `crates/vize_carton/src/interner.rs`）。

原子就是普通的 `&'a str`，因此名称比较是内容比较，而非指针同一性比较。内联换来的是分配的节省和缓存局部性，它并不是 `==` 的快速路径。

### 增量编译

Vite 插件（`@vizejs/vite-plugin`）按文件级缓存，分为键不同的两层：

- **内存中，用于开发和 HMR** — 以解析后的文件路径为键（`npm/builder/vite/src/plugin/compiled-module-cache.ts`）。条目在热更新时被显式逐出，而不是重新计算键，因此被改动的文件会重新编译，而它的邻居不会。
- **预编译变更检测** — 以 `mtime` 加大小为键，比较在 Rust 中进行（`crates/vize_atelier_sfc/src/vite_plugin/precompile.rs`）。决定一个批次重新编译哪些文件的正是这道闸门。
- **磁盘上，跨进程** — 位于 `node_modules/.vize/vite-precompile`，以源码的 SHA-256 哈希，加上涵盖编译器二进制标识与解析后选项的清单键为键（`npm/builder/vite/src/plugin/precompile-cache-key.ts`）。这里使用内容哈希，正是因为 `mtime` 跨机器和跨检出并不可信。

## 实测：Arena 与表达式工作

上述编译器内部工作由按 crate 的微基准框架（`cargo bench --bench davinci`）在固定的六个夹具阶梯
`tools/benchmarks/crates/davinci_harness/fixtures/{small,medium,large,stress-deep,stress-wide,stress-interp}.vue` 上测量。

**如何阅读这些数字。** 分配次数是确定性的且与机器无关，因此是精确事实，并被用作回归棘轮。墙钟时间是在共享的开发机上以 `--quick` 采样测得的，**仅具方向性** —— 参考运行器（Blacksmith）的记录仍待完成，这也是 `docs/davinci/plan/budgets.toml` 中每一项 `wall_p50_ns` 和 `allocs` 仍为 `0`（意为“尚未记录，仅供参考”）的原因。每次运行的结果文件落在 `tools/benchmarks/results/davinci/`，属于本地产物，而非已提交的基线。

字符串与 arena 工作前后，每次编译的分配调用次数（精确值，同一批夹具）：

| 夹具            | 解析      | DOM 编译    | SSR 编译      | Vapor 编译    |
| --------------- | --------- | ----------- | ------------- | ------------- |
| `small`         | 21 → 9    | 52 → 39     | 73 → 60       | 90 → 73       |
| `medium`        | 171 → 107 | 329 → 264   | 1,099 → 1,030 | 588 → 515     |
| `large`         | 350 → 272 | 656 → 573   | 1,106 → 983   | 1,136 → 1,003 |
| `stress-deep`   | 397 → 155 | 669 → 426   | 612 → 369     | 764 → 514     |
| `stress-wide`   | 213 → 204 | 255 → 245   | 416 → 405     | 280 → 261     |
| `stress-interp` | 616 → 105 | 1,048 → 536 | 3,149 → 2,637 | 1,495 → 974   |

节点大小随之缩小，新的大小由 `const` 断言固定：`RootNode` 296 → 224 字节，`DirectiveNode` 208 → 176，`ElementNode` 128 → 104，`SimpleExpressionNode` 120 → 88，`AttributeNode` 80 → 56，`TextNode` 32 → 24。

**峰值常驻内存。** 跨文件复用 arena 是单项最大的收益，而且是内存结果而非速度结果。编译已提交语料库中全部 36,541 个 SFC（`vize build "tests/_fixtures/_git/**/*.vue" --format stats`，`ci-opt` 二进制，取自 `/usr/bin/time -l` 的最大常驻集大小，前后同一台机器）：

| 工作线程 | 之前     | 之后     | 变化       | 各运行次数 |
| -------- | -------- | -------- | ---------- | ---------- |
| 12       | 766.5 MB | 171.1 MB | **−77.7%** | 5          |
| 1        | 717.0 MB | 88.2 MB  | **−87.7%** | 3          |

单工作线程的数字是累积信号：它不受调度影响，因此表明旧的峰值来自逐文件的泄漏，而不是每个工作线程的 arena。墙钟时间在噪声范围内没有变化，输出的 36,541 个文件全部逐字节相同（比对 SHA-256 清单）。

**表达式重复解析。** 模板表达式现在只在模板解析阶段解析一次，并保留在节点上。消费方读取保留的 AST，而不是重新解析文本。在 SSR 通道上，`stress-interp` 夹具每次编译的冗余表达式重解析从 500 次降到零，该融合通道相对于引入保留之前的代码树净减 **−13.6%** 墙钟时间（346.8µs → 299.8µs）—— 解析本身变贵了，而消费方便宜得多。DOM 和 Vapor 通道在该夹具上没有可删除的重解析，因此仍然承担新增的解析成本；解决这一点作为剩余的阶段工作被跟踪，而不是已交付的收益。

## 基准测试：Linter — patina 与 eslint-plugin-vue 的比较

对 **15,000 个 Vue SFC 文件**进行 lint（本地工作站）：

|          | eslint-plugin-vue （ST） | 维泽包浆（ST） | 加速      | eslint-plugin-vue （MT） | Vize包锈（MT） | 加速      | **eslint ST vs Vize MT** |
| -------- | ------------------------ | -------------- | --------- | ------------------------ | -------------- | --------- | ------------------------ |
| **时间** | 45.08秒                  | 4.02秒         | **11.2x** | 16.38秒                  | 784毫秒        | **20.9x** | **57.5x**                |

跑`vp run --workspace-root bench:lint`来繁殖。

### 类型感知绒毛配置文件

类型感知的附加处理在成本趋于聚集的阶段被有意描绘：SFC 解析，
Croquis 分析、虚拟 TypeScript 生成、模板查询收集和 Corsa 探针。当
启用了多个模板支持的类型感知规则，Patina 收集模板表达式和
模板 Promise 查询在 Corsa 探测阶段前的一次 AST 走动中完成。查询集合也共享
OXC表达式解析用于unsafe模板和浮动承诺检查，因此一个模板表达式
当两个规则都启用时，不支付重复解析成本。

跑`vize lint --profile --preset opinionated src`去本地项目看看这些排。该
Profile Report还包括严格的审计部分，检查工作时间的累积覆盖情况
工作时间、慢阈值点击，以及在列出热文件和内部文件前捕获的内部时段
行动。热文件行显示每阶段的共享和吞吐量，操作行显示主导
跨度或最大/平均峰值。

## 基准测试：Formatter — 字形与更漂亮

格式化 **15,000 个 Vue SFC 文件**（本地工作站）：

|          | Prettier （CLI） | 维泽字形（ST） | 加速      | 维泽字形（MT） | **Prettier CLI vs Vize MT** |
| -------- | ---------------- | -------------- | --------- | -------------- | --------------------------- |
| **时间** | 101.20秒         | 2.97秒         | **34.1x** | 835毫秒        | **121.2x**                  |

跑`vp run --workspace-root bench:fmt`来繁殖。

<!-- benchmark:typecheck:start -->

## 基准：类型检查

README 与所有语言页面均由同一个已提交的产物生成。文件数量按行标明。

| 运行环境                                 | 测量时间                 | 测量次数 | 预热次数 |
| ---------------------------------------- | ------------------------ | -------- | -------- |
| `blacksmith-32vcpu-ubuntu-2404` (32 CPU) | 2026-09-16T08:19:25.262Z | 3        | 1        |

[Actions](https://github.com/ubugeeei-prod/vize/actions/runs/35072615669) · [f9cc50ee2e2f](https://github.com/ubugeeei-prod/vize/commit/f9cc50ee2e2ff14cd120241cd87a0162446df65b) · [JSON](https://github.com/ubugeeei-prod/vize/blob/main/tools/benchmarks/results/tool-benchmark-latest.json)

版本: vize: `vize 0.424.11` · tsgo: `Version 7.0.2` · vueTsc: `Version 6.0.3` · verterTsc: `verter-tsc 0.0.1-beta.3` · vue: `3.6.0-beta.10` · node: `v24.14.0`.

| 项目              | 文件数 | 比较工具 | 比较工具中位数 | Vize 1T | Vize max | 速度比   |
| ----------------- | ------ | -------- | -------------- | ------- | -------- | -------- |
| 类型检查          | 500    | vue-tsc  | 6.75s          | 2.03s   | 1.37s    | **4.9x** |
| 大型 SFC 类型检查 | 1      | vue-tsc  | 1.34s          | 186.4ms | 190.1ms  | **7.0x** |

类型检查速度比比较 Vize 与 vue-tsc，即 Vue 项目当前实际使用的类型检查器。vue-tsc 运行 JavaScript 版 TypeScript 编译器，而 Vize 运行原生 tsgo，因此该比值涵盖整条工具链，而非仅 Vue 层；完整快照中按引擎类别分组的表格会在同一引擎内部排名，用于单独衡量 Vue 层。每次计时都通过诊断工作验证，但不同工具的诊断覆盖范围并不相同。这些比值不代表准确性相同，也不代表相对于旧版 Vize 的性能提升。未通过验证的比较工具不发布耗时或排名。

[完整方法与验证结果](./performance-blacksmith)

以下分析为历史本地测量，不属于本次参考环境比较。

<!-- benchmark:typecheck:end -->

### 类型检查员配置文件

500-SFC 配置文件灯具将大部分墙时存储在 Corsa CLI 命令中，而导入重写快速路径则消除了之前未使用 Vue 指定符文件的 OXC 解析成本：

| 公制                       | 在        | 现状      |
| -------------------------- | --------- | --------- |
| `canon.import.rewrite.vue` | 26.77毫秒 | 2.45毫秒  |
| 生成的最大虚拟TS           | 15,401B   | 14,414B   |
| 总轮廓壁时间               | 1.88秒    | 668毫秒   |
| 科萨诊断阶段               | 1.67秒    | 482毫秒   |
| Corsa CLI parse            | 无        | 10.41毫秒 |

Rust侧的 `virtual project` 阶段——每文件的 SFC 解析，Croquis 分析，
虚拟 TS 生成和导入重写——在 rayon 的话题中被扇动
`VirtualProject::register_paths`里的泳池。每个`.vue`文件都是独立的
一旦工作区选项解析完成，一个批次就能并行化
干净利落。在1000 SFC的灯具上，相位从~~71毫秒降至~~25毫秒之前
甚至还提到了科萨。

### 重诊断的 e2e 灯具

当灯具存在时，`tools/benchmarks/scripts/check.ts`还会测量`tests/_fixtures/_git/npmx.dev`应用。这会捕捉真实应用夹具上的诊断映射路径：

| 固定装置      | 来源SFC文件 | 虚拟文件 | 诊断  | 维兹正史 |
| ------------- | ----------- | -------- | ----- | -------- |
| npmx.dev 应用 | 134         | 226      | 1,053 | 1.94秒   |

该灯具当前配置文件保持CLI诊断解析在~7毫秒。大部分时间现在都集中在 Corsa CLI 命令本身。将框架自动导入存根提升到一个环境文件中，也使生成的最大虚拟TS文件从约275KB减少到144KB。

<!-- benchmark:vite:start -->

## 基准：Vite 插件

README 与所有语言页面均由同一个已提交的产物生成。文件数量按行标明。

| 运行环境                                 | 测量时间                 | 测量次数 | 预热次数 |
| ---------------------------------------- | ------------------------ | -------- | -------- |
| `blacksmith-32vcpu-ubuntu-2404` (32 CPU) | 2026-09-16T08:19:25.262Z | 3        | 1        |

[Actions](https://github.com/ubugeeei-prod/vize/actions/runs/35072615669) · [f9cc50ee2e2f](https://github.com/ubugeeei-prod/vize/commit/f9cc50ee2e2ff14cd120241cd87a0162446df65b) · [JSON](https://github.com/ubugeeei-prod/vize/blob/main/tools/benchmarks/results/tool-benchmark-latest.json)

版本: vize: `vize 0.424.11` · tsgo: `Version 7.0.2` · vueTsc: `Version 6.0.3` · verterTsc: `verter-tsc 0.0.1-beta.3` · vue: `3.6.0-beta.10` · node: `v24.14.0`.

文件数: **1,000**.

|           | @vitejs/plugin-vue | @vizejs/vite-plugin | 速度比   |
| --------- | ------------------ | ------------------- | -------- |
| Vite 构建 | 1.66s              | 889.2ms             | **1.9x** |

[完整方法与验证结果](./performance-blacksmith)

<!-- benchmark:vite:end -->

在此之前发布的数字 —— `957ms` / `479ms` / `2.0x` —— 来自 #3392 之前的 `tools/benchmarks/scripts/vite.ts`：它让 Vize 带着自身预热留下的持久预编译缓存运行，而 `@vitejs/plugin-vue` 从零开始编译。该测试工具现在会在其运行的机器上分别报告冷启动和热启动两行，因此它的输出是本地诊断值，而不是可发布的加速比。请使用 `vp run --workspace-root bench:vite` 来比较改动前后的自身表现。
