---
title: 语言工程实践
---

<!-- Generated translation; source: architecture/language-engineering-practices.md -->

# 语言工程实践

Vize 是一个 Vue 工具链，但它具有与编译器相同的故障模式：微小的语法更改可以
同时移动诊断、代码生成、编辑器行为、包输出和性能
时间。本页记录了Vize采用的成熟编译器和类型的语言处理实践
检查器存储库，然后将它们映射到 Vize 自己的装置、快照、奇偶测试、基准测试，
并释放门。

## 源信号

| 来源                                                                                                                                 | 实践观察                                                                                                                                               | 维兹翻译                                                                                                                                                         |
| ------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`rust-lang/rust`](https://github.com/rust-lang/rust) 和 [`rustc-dev-guide`](https://rustc-dev-guide.rust-lang.org/tests/intro.html) | `compiletest` 按套件对 UI 测试进行分组，存储接近源案例的预期输出，使用 `tidy` 作为存储库不变量，并分别跟踪生态系统和性能回归。                         | 首先将面向编译器的更改视为固定装置更改。将解析器/编译器期望保留在 `tests/fixtures` 和 `tests/expected` 中，并将存储库不变性保留在 `tests/tooling/*.test.ts` 中。 |
| [`rustc` 生态系统和性能测试](https://rustc-dev-guide.rust-lang.org/tests/ecosystem.html)                                             | Crater、cargotest、大型项目构建器和 rustc-perf 在合并编译器更改之前或之后明确了广泛的兼容性和性能风险。                                                | 升级广泛的 Vue 语义、生成的代码形状或对现实世界固定装置、Vue 奇偶校验矩阵和 PR 基准预算的热路径更改，而不是仅依赖于单位固定装置。                                |
| [`rust-fuzz/cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) 和 libFuzzer                                                       | 覆盖引导的模糊目标运行任意字节输入，保留语料库，并在将崩溃重现器转变为确定性回归之前将其最小化。                                                       | 在将崩溃修复视为完成之前，从 `tests/fuzz` 到 `cargo +nightly fuzz run <target>` 进行模糊解析器、词法分析器、CSS、表达式和模板编译边界。                          |
| [Linux内核测试](https://www.kernel.org/doc/html/next/dev-tools/testing-overview.html)                                                | KUnit 涵盖小型白盒单元，kselftest 涵盖用户可见的系统界面，KCOV 提供覆盖引导的模糊测试，`perf stat` 捕获可重复的计数器和计时状态。                      | 将微小的板条箱级检查与 CLI/工作区集成检查分开，对任意输入使用覆盖率/模糊测试，并在热路径移动时附加配置文件或基准测试状态。                                       |
| [Chromium测试和CQ](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/testing/testing_in_chromium.md)                        | Chromium 层密封单元、浏览器、网络、遥测和模糊器测试； CQ/trybots 使昂贵或不稳定的通道变得明确，ClusterFuzz 大规模运行发现的模糊目标。                  | 默认情况下保持 Vize 检查密封，将浏览器/应用程序行为升级到现实世界的固定装置，使用 PR 基准预算实现类似遥测的状态，并保留模糊重现器进行分类。                      |
| [V8测试](https://v8.dev/docs/test)和[功能发布](https://v8.dev/docs/feature-launch-process)                                           | V8 运行 `mjsunit` 和 Test262 等引擎套件，仅在审核后重新生成预期文件，使用 `tools/run_perf.py` 和基准比较流程，并且需要在发布语言功能之前进行模糊测试。 | 将 Vue/TS 兼容性更改视为语言功能：引用源行为、添加场景测试、比较相关性能，以及在升级之前运行或安排模糊测试。                                                     |
| [`microsoft/TypeScript`](https://github.com/microsoft/TypeScript)                                                                    | Hereby 任务图将构建、格式化、lint、测试和基线任务分开。编译器输出通过 `tests/baselines/reference` 与 `baseline-accept` 之前本地生成的输出进行比较。    | 将快照保留为已审核的合同。更改的 `tests/snapshots/*` 或 Rust `insta` 快照必须由 PR 进行解释，并仅限于更改的行为。                                                |
| [`TypeScript tests/cases/fourslash`](https://github.com/microsoft/TypeScript/tree/main/tests/cases/fourslash)                        | 面向编辑器的语言服务行为被捕获为数千个场景文件，而不是仅从编译器测试中推断出来。                                                                       | LSP、快速修复、完成、悬停和增量编辑器更改应该具有场景级烟雾或集成覆盖，而不仅仅是解析器/编译器固定装置。                                                         |
| [`microsoft/typescript-go`](https://github.com/microsoft/typescript-go)                                                              | 本机端口保留 TypeScript 子模块作为参考实现，添加最少的编译器测试，将生成的输出写入 `testdata/baselines/local`，并将减少的 `.diff` 基线视为收敛证据。   | 在引入 Vize 特定规则之前，将 Vize 输出与官方 Vue 和 TypeScript 行为进行比较。如果 Vize 故意出现偏差，请记录原因和兼容性层。                                      |
| [`facebook/flow`](https://github.com/facebook/flow)                                                                                  | Flow 使用 `.exp` 预期输出保持目录状集成测试，支持重新记录有意的输出更改，并为编辑器和服务器流使用操作/断言样式 `newtests`。                            | 更喜欢使用小型场景装置进行诊断和编辑器工作流程。仅在检查差异并将生成的噪声保持在基线之外之后，重新记录的快照才是可接受的。                                       |

## Vize 更改类

每个语言处理 PR 都应该命名其变更类并包含来自匹配的证据
行。在开发过程中使用最窄的命令，然后在更改涉及共享时扩大
行为。

| 更改班级                    | 所需证据                                                                                     | 常用命令                                                                                                                                                   |
| --------------------------- | -------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 解析器或 AST                | 最小的解析器固定装置、预期的 AST 或错误输出，并且没有广泛的快照刷新。                        | `cargo test -p vize_armature`、`cargo test -p vize_test_runner`、`node tests/tooling/support/generate-expected.ts <fixture>`                               |
| 编译器和代码生成器          | 最小源装置、DOM/Vapor/SSR 预期输出以及发出的运行时形状发生变化时的真实世界奇偶校验。         | `cargo test -p vize_atelier_dom`、`cargo test -p vize_atelier_vapor`、`vp run --filter './tests' test:build`                                               |
| 语义分析、lint 和跨文件分析 | 规则或分析器装置、JSON 或代理输出快照以及更改诊断的文档。                                    | `cargo test -p vize_patina`、`vp run --filter './tests' test:lint`、`node --test tests/tooling/snapshot-baselines.test.ts`                                 |
| 虚拟 TypeScript 和类型检查  | 最小的 SFC 夹具、映射的诊断快照、生成的虚拟 TS 审查以及官方 Vue 或 TypeScript 奇偶校验说明。 | `vp run --filter './tests' test:check:fixtures`、`cargo test -p vize_canon`、`vize check --show-virtual-ts <file>`                                         |
| 格式化程序和 LSP            | 黄金格式输出或协议烟雾覆盖，加上用户可见行为时的集中编辑器集成检查。                         | `cargo test -p vize_glyph`、`cargo test -p vize_maestro`、`node --test tests/tooling/lsp-smoke.test.ts`                                                    |
| 运行时打包、发布或文档      | 生产状况发生变化时进行治理测试、烟雾安装或工作流程覆盖以及发布/准备文档。                    | `node --test tests/tooling/*.test.ts`、`rust-script tools/commands/release/npm/smoke-release-install.rs --prepare-manifests --runtime-checks`、`vp run --workspace-root check:ci` |

## 保障通道

除了更改类之外，某些更改还需要第二个镜头。这些车道保持安全状态，
绩效状态，并模糊 PR 中明确的证据，而不是让他们作为审阅者
记忆。

| 车道     | 当变化触及                                                                               | 时使用记录证据                                                                                                                                                                                                                                |
| -------- | ---------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 安全     | URL 处理、HTML 或 SSR 输出、文件系统/配置加载、本机加载、包发布、CI 或凭据。             | `security-audit`（`.github/workflows/check.yml`、`vp exec pnpm audit --prod --audit-level moderate`、`cargo audit --deny warnings`）、烟雾安装运行时检查、固定 GitHub Actions 检查以及涵盖风险输入或边界的任何集中回归。                      |
| 性能     | 解析器、编译器、linter、格式化程序、类型检查器、缓存、项目图遍历、生成的输出或 CLI I/O。 | 回归需要时输出 `.github/workflows/benchmark.yml`、`bench/compare-pr.mjs`、`bench/enforce-pr-budget.mjs`、`pr-benchmark-budget` 状态、本地 `bench:*` 任务以及 `vize lint --profile`、`vize check --profile` 或 `vize fmt --profile` 输出归因。 |
| 模糊测试 | 面向字节的解析、语法恢复、CSS 解析、JS/TS 表达式解析、模板词法分析或代码生成恢复。       | `.github/workflows/fuzz.yml`、`tests/fuzz/Cargo.toml`、`tools/commands/ci/fuzz/seed_corpus.rs`、`cargo +nightly fuzz run <target>`、上传的 `fuzz-reproducers-*` 工件以及崩溃、超时或 OOM 后的最小化确定性回归已被了解。                                  |

## 基线政策

- 从最小的失败或说明性案例开始，然后仅在更广泛的情况下才接受更广泛的固定装置
  证明交叉行为。
- 快照和基线文件是用户可见的合同。如果差异更改了诊断，则生成
  代码、公共 CLI 输出或编辑器行为，PR 应该说明为什么新输出是正确的。
- 在不稳定数据达到基线之前对其进行标准化。路径、时间、哈希值和环境
  细节不应造成重复的快照流失。
- 保持奇偶校验工件明确。 `tests/snapshots/check`、`tests/snapshots/lint`，真实世界
  Fixture 快照和 Vue 奇偶校验矩阵是兼容性记录。
- 不要刷新大型快照基线，除非 PR 与这些输出有关。当许多文件移动时
  在一起，包括对共同原因的简短解释。

## 升级触发器

当变更具有以下形式之一时，寻求更广泛的证据：

- 语法、转换或虚拟 TypeScript 行为可能会影响普通 Vue 应用程序：
  添加或更新现实世界的装置并解释与官方 Vue 工具的奇偶性。
- 生成的代码形状、缓存、项目图遍历或类型感知分析可以移动
  吞吐量：运行与表面匹配的本地基准并依赖于 PR 基准预算。
- URL 处理、HTML/SSR 输出、配置加载、包发布、本机加载、CI 或
  凭证相邻代码更改：记录安全审核状态并添加重点回归
  证明边界仍然被守卫。
- 解析器恢复、任意字节输入、CSS/模板/表达式解析或崩溃修复：运行或
  安排匹配的模糊目标，保留再现器，并实现最小化的确定性
  关闭修复请求之前的回归。
- LSP、编辑器、快速修复、完成、悬停或增量行为更改：添加场景级别
  执行用户可见序列的覆盖范围，而不仅仅是最终诊断。
- 快照会因路径、哈希值、顺序、计时、环境或主机平台而发生更改：
  首先进行归一化，然后仅在剩余差异有意义时才接受基线。

## 操作护栏

Vize 使这些实践保持可执行，而不是依赖于内存：

- `CONTRIBUTING.md` 为贡献者命名变更类规则。
- `.github/PULL_REQUEST_TEMPLATE.md` 要求提供行为参考、风险和验证证据。
- `bench/test-inventory.mjs` 报告 PR CI 中当前的测试资产库存。
- `.github/workflows/benchmark.yml` 比较基础和头部 CLI 性能并执行 PR 预算。
- `.github/workflows/check.yml` 为生产 npm 和 Rust 运行 `security-audit` 作业
  依赖性咨询。
- `.github/workflows/fuzz.yml` 运行 `tests/fuzz` 货物模糊工作区并上传崩溃
  用于解析器/编译器分类的再现器。
- `docs/release/production-readiness.md` 和 `docs/release/vue-parity-matrix.md` 定义何时
  行为可以称为生产就绪或兼容。
- `tests/tooling/language-engineering-practices.test.ts` 保留此页面，贡献指南，
  和 PR 模板连接在一起。
