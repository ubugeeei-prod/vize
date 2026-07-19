---
title: 稳定性
description: Vize v1 alpha 支持层级、兼容性承诺以及实验性表面。
---

<!-- Generated translation; source: stability.md -->

# 稳定

Vize正朝着v1 alpha迈进。alpha 合同故意比稳定的 v1 更窄
合同：它指定了早期采用者应能使用的表面，同时保留空间
快速更换内部结构和实验集成。整个项目尚未完全完成
生产准备工具链;发布决策应使用以下内容
[生产准备检查表](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/production-readiness.md)。
弃用窗口、SemVer规则和发布行支持在
[支持政策](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/support-policy.md)。

## 版本控制合同

在v1稳定版之前，任何预发布都可以包含破坏性变更。Vize仍然把破坏性变更当作
发布说明材料，尤其是影响包入口点、CLI 标志、配置字段时，
诊断代码，或生成输出。

v1 alpha 系列采用以下规则：

| 表面                   | 阿尔法期望值                                         |
| ---------------------- | ---------------------------------------------------- |
| 已发布的软件包名称     | 应保持可用或随迁移说明一起发货                       |
| 文档化的CLI命令和标志  | 应避免无声的行为改变                                 |
| 文档配置字段           | 除非发布说明提到变更，否则名称和数值形状应该保持稳定 |
| 文档中列出的诊断代码   | 应保持可识别性，以便抑制和修复报告有用               |
| 已发布的Rust箱子API    | 请按照下面的每个箱子等级和弃用合同                   |
| 未出口的Rust箱内部结构 | 在v1稳定之前，若不支持迁移，可能会有所变化           |
| 生成代码与虚拟TS输出   | 根据需要调整以保证正确性、兼容性、性能或诊断         |

## 运行时支持

公共 npm 运行时包的默认Node.js层是 Node 22，包括
`oxlint-plugin-vize`。Oxlint 插件声明 `^22 || >= 24`，所以 Node 22 和 Node 24 或更新版本是
而Node 23则不在测试的兼容性矩阵之外。

发布流程构建适用于 macOS、Linux 和 Windows 的原生包，覆盖 x64 和 arm64
其中包声明支持。CI兼容性作业涵盖声明的节点层和
当前项目节点版本。

全新安装的烟雾矩阵（`.github/workflows/native-smoke.yml`）每周运行一次
节奏和按需，而不是每次公关宣传都要这样。它对已发布的软件包安装路径进行演练
GitHub托管的Linux-x64-GNU、Linux-arm64-gnu、darwin-arm64和win32-x64-msvc运行程序;该
剩余的Darwin-X64和Win32-ARM64-MSVC目标仍保留在特定架构的托管运行器上。
矩阵运行在节点22和节点24之间。发布标签仍被发布工作流程阻挡
Tarball 在 NPM 包发布前安装烟雾。运行时烟雾检查`vize --version`，
`vize check`，`@vizejs/native` `require`和`import`，还有一个
`@vizejs/vite-plugin` `vite build`安装的沥青球。

目前，托管的新安装运行器并未执行两个已声明的Linuxmusl目标。
它们被每个平台的建造文件和`@vizejs/native-*`覆盖
可选依赖解析器，直到容器化的Alpine烟雾能够分阶段匹配的本地人
塔博尔：

| 目标             | 主持跑者空档                                              | 补偿覆盖                                             |
| ---------------- | --------------------------------------------------------- | ---------------------------------------------------- |
| Linux-x64-musl   | 没有 GitHub 托管的 Alpine/musl 虚拟机作为原生运行工具可用 | 建造工作会发出musl tarball;手动`node:alpine`烟雾。   |
| Linux-arm64-musl | Arm64托管运行器是Ubuntu GNU，不是Alpine/musl原生主机      | 建造工作发射arm64musl tarball;手动Alpine Arm64烟雾。 |

这些缺口的缩小与[#493](https://github.com/ubugeeei-prod/vize/issues/493)并列追踪。

该工作区的最低支持Rust版本（MSRV）在`Cargo.toml`下声明
`[workspace.package].rust-version`。`rust-toolchain.toml` 固定的开发工具链
可能是同一个版本，或者更新。在 v1 稳定之前，MSRV 可以在任何预发布中继续推进;
该动作在发布说明中会被提及。下游打包器应读
`rust-version`是从箱子的 `Cargo.toml` 中判断，而不是从工具链文件推断。

## 包支持层级

| 等级       | 包裹                                                                                          | 合同                                                                  |
| ---------- | --------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| Alpha支持  | `vize`，`@vizejs/native`，`@vizejs/vite-plugin`                                               | 旨在早期生产试验，配合发布说明支持的破坏性变更。                      |
| 兼容性预览 | `@vizejs/unplugin`，`@vizejs/rspack-plugin`，`@vizejs/nuxt`，`@vizejs/musea-nuxt`             | 预计适用于常见的主机配置，但主机与框架的兼容性可能会迅速变化。        |
| 实验       | `oxlint-plugin-vize`，`@vizejs/vite-plugin-musea`，`@vizejs/musea-mcp-server`，`@vizejs/wasm` | 公开包，但 API、命令、输出和工作流形状在 alpha 阶段可能会发生变化。   |
| 孵化       | `@vizejs/fresco`、`@vizejs/fresco-native`、编辑器扩展包                                       | 对开发和反馈很有用，但尚未成为v1 alpha Ready Production目标的一部分。 |

## 锈箱支撑等级

本表为 crates.io 消费者的规范兼容性契约。它覆盖了每个箱子
其货物元数据允许发布，包括因发布而暂时推迟的箱子
出版商在他们的首个发行准备 crates.io 期间。私有模块与实现
细节不是兼容性曲面。

<!-- rust-crate-support:start -->

| 箱子                 | 等级       | 目标观众                  | 公共入口                                        | 移除/废弃                 |
| -------------------- | ---------- | ------------------------- | ----------------------------------------------- | ------------------------- |
| `vize_carton`        | Alpha支持  | Vize 编译器和库作者       | `vize_carton::{Allocator, Bump, FxHashMap}`     | 一个辅修`#[deprecated]`   |
| `vize_relief`        | Alpha支持  | AST 与编译器集成作者      | `vize_relief::{RootNode, CompilerOptions}`      | 一个辅修`#[deprecated]`   |
| `vize_armature`      | Alpha支持  | 解析 Vue 模板的工具       | `vize_armature::{parse, Parser, Tokenizer}`     | 一个辅修`#[deprecated]`   |
| `vize_croquis`       | 兼容性预览 | 语义与类型感知工具作者    | `vize_croquis::{Croquis, Drawer}`               | 一个小调，`#[deprecated]` |
| `vize_croquis_cf`    | 实验       | 自愿参与的全项目分析实验  | `vize_croquis_cf::CrossFileAnalyzer`            | 没有最低要求;实用时会断音 |
| `vize_atelier_core`  | Alpha支持  | Custom Vue 编译器后端作者 | `vize_atelier_core::{transform, generate}`      | 一个辅修`#[deprecated]`   |
| `vize_atelier_dom`   | Alpha支持  | VDOM编译器和捆绑器集成    | `vize_atelier_dom::compile_template`            | 一个辅修`#[deprecated]`   |
| `vize_atelier_vapor` | 实验       | 选择加入的蒸汽编译器集成  | `vize_atelier_vapor::compile_vapor`             | 没有最低要求;实用时会断音 |
| `vize_atelier_ssr`   | 兼容性预览 | SSR与框架集成作者         | `vize_atelier_ssr::compile_ssr`                 | 一个辅修`#[deprecated]`   |
| `vize_atelier_sfc`   | Alpha支持  | SFC工具与捆绑器作者       | `vize_atelier_sfc::{parse_sfc, compile_sfc}`    | 一个辅修`#[deprecated]`   |
| `vize_atelier_jsx`   | 兼容性预览 | JSX/TSX 编译器及工具作者  | `vize_atelier_jsx::{compile_jsx, lower_source}` | 一个辅修`#[deprecated]`   |
| `vize_musea`         | 实验       | 博物馆画廊与文档工具      | `vize_musea::{parse_art, transform_to_csf}`     | 没有最低要求;实用时会断音 |
| `vize_fresco`        | 孵化       | TUI实验                   | `vize_fresco::{RenderTree, LayoutEngine}`       | 没有最低限度              |
| `vize_canon`         | 兼容性预览 | 类型检查与编辑器集成      | `vize_canon::{type_check_sfc, TypeChecker}`     | 一个辅修`#[deprecated]`   |
| `vize_patina`        | 兼容性预览 | Linter 和 Oxlint 积分     | `vize_patina::{lint, Linter}`                   | 一个辅修`#[deprecated]`   |

<!-- rust-crate-support:end -->

每个箱子还会在`package.metadata.vize.stability`上记录其等级。CI会比较那些货物
元数据值、此表以及完整的发布-发布者箱集合，因此添加、删除或
重新分类可发布的箱子不能默默更改合同。

### SemVer 门解读

`cargo-semver-checks`为发布商的箱子运行，这些箱子有可解析的注册表
基线。等待首次发布或被封锁的箱子，一旦其发布，就会加入该矩阵
基线数据已提供。在此之前，元数据/表/发布列表检查仍然适用。

| 等级                 | CI解释                                                              |
| -------------------- | ------------------------------------------------------------------- |
| Alpha支持/兼容性预览 | API 中断必须被修复，或遵循支持策略的弃用窗口，并带有常规中断标记。  |
| 实验                 | 门意外漂移;有意断裂可能会使用无废止窗口的断裂标记。                 |
| 孵化                 | 同样的检测效果适用，但整个API或箱子在任何版本中都可能被替换或移除。 |

CI识别的断开标记是常规变更标题中的`!`，或
`BREAKING CHANGE:`脚。通过任何一个标记的门槛都不会免除退役
用于支持Alpha或兼容预览箱的窗口。

## 什么算是足够稳定，适合阿尔法

当包或命令满足以下条件时，可以迁移到Alpha支持层：

- 文档化的安装和使用路径
- 对包构建、安装及支持Node运行时的CI覆盖
- 对已发布的入口点释放烟雾覆盖
- 为回归和兼容性报告提供明确的所有者
- 相关指南中记录的已知无支持行为

## 尚未承诺的事

alpha版本并不保证对所有Vue编译器边缘情况、每个包都完全兼容
管理器布局、所有编辑器功能，或者每个框架集成。当Vize不同意
官方Vue工具，除非是Vize指南，否则将官方输出视为兼容性基线
明确记录了不同的行为。发布阻断编译器、类型检查、运行时，
而 Vite 构建曲面在
[Vue 奇偶校验矩阵](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/vue-parity-matrix.md)。

有关安全处理，请参见仓库 `SECURITY.md`。关于贡献和修复的工作流程，请参见
`CONTRIBUTING.md`。
