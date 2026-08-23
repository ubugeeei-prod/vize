---
title: CLI
---

<!-- Generated translation; source: guide/cli.md -->

# CLI 参考

> **⚠️ 正在进行中：**Vize 正在积极开发中，CLI 表面仍在不断发展。

大多数应用流程都应该安装`vize` npm包并通过`package.json`
剧本。本页介绍了 LSP 的底层 Rust 原生 `vize` 二进制，IDE 管理，
`check-server`、分析以及其他直接的CLI工作流程。npm 包会暴露共享配置
辅助工具以及NAPI支持的`build`、`fmt`、`lint`、`check`、`clean`、`ready`和`upgrade`命令。

关于分析流程的更高级解释，请参见[静态分析](./static-analysis.md)。

## 应用包脚本

对于应用，从npm安装并用稳定命令连接到项目脚本：

```bash
vp install -D vize
```

```json
{
  "scripts": {
    "vize:build": "vize build src",
    "vize:fmt": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path src",
    "vize:check": "vize check src",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:lint
vp run vize:check
vp run vize:ready
```

用于一次性本地调试`vp exec vize ...`，但文档化时更倾向于使用命名脚本
工作流程和配置项。

## 锈蚀二进制安装

对于v1 alpha，可以使用预构建的GitHub发布二进制文件或Nix入口。Rust CLI 不是
crates.io 安装渠道支持。

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

你也可以从以下平台下载特定平台的二进制文件
[GitHub发布](https://github.com/ubugeeei-prod/vize/releases)。

在本仓库内进行本地开发时，请安装工作区构建：

```bash
cargo install --path crates/vize --force --locked
```

## npm 包脚本 vs Rust CLI

| 需要                                               | 推荐入门点                  |
| -------------------------------------------------- | --------------------------- |
| 构建、格式化、lint、检查、准备和升级的脚本打包     | `vp run vize:*` 来自 NPM 包 |
| 跨`.vue`、`.ts`、`.tsx`和`.d.ts`的项目支持类型检查 | 锈蚀`vize check`            |
| LSP、IDE设置、`check-server`和配置文件伪影         | 锈`vize`二进制              |
| 共享 Vite 插件、npm 包命令和 Rust CLI 设置         | `vize.config.*`             |

## 命令

```bash
vize [COMMAND]
```

当无命令调用时，`vize`默认为`build`。

| 指挥           | 描述                                      |
| -------------- | ----------------------------------------- |
| `build`        | 编译Vue SFC文件                           |
| `fmt`          | 格式化Vue SFC文件                         |
| `lint`         | Lint Vue SFC 文件                         |
| `check`        | 类型检查 Vue SFC、TS、TSX 和 `.d.ts` 输入 |
| `inspector`    | 创建游乐场编译器检查器负载                |
| `clean`        | 移除Vize生成的缓存伪影                    |
| `ready`        | 运行`fmt`、`lint`、`check`和`build`       |
| `upgrade`      | 更新已安装的CLI                           |
| `check-server` | 启动Unix JSON-RPC类型检查服务器           |
| `musea`        | Musea子指令与支架                         |
| `lsp`          | 启动语言服务器                            |
| `ide`          | 安装或管理编辑器集成                      |

所有`--profile`终端报告均由本地`vize_curator`箱处理。该
Instrumentation Hooks仍`vize_carton`，而Curator则拥有CLI报告表格
面向探员和特工的文物。

## 建造

```bash
vize build src/**/*.vue
vize build --ssr
vize build --profile src
```

关键选项：

| 选项                          | 描述                                         |
| ----------------------------- | -------------------------------------------- |
| `-o, --output`                | 低于公共输入根的源相对输出;拒绝碰撞          |
| `-f, --format`                | 输出格式：`js`、`json`、`stats`              |
| `--ssr`                       | 启用SSR编译                                  |
| `--custom-renderer`           | 将小写非HTML标签视为自定义渲染器元素         |
| `--custom-elements <PATTERN>` | 作为自定义元素编译的标签模式；可重复指定     |
| `--script-ext`                | `preserve`或`downcompile`                    |
| `--declaration`               | 为构建的SFCs（别名：`--dts`）发布`.d.ts`文件 |
| `--declaration-dir`           | 声明输出目录（默认：构建输出目录）           |
| `-j, --threads`               | 线数覆盖                                     |
| `--profile`                   | 打印时序配置文件                             |
| `--continue-on-error`         | 继续编译并在最后报告失败                     |

## 节目形式

```bash
vize fmt --check src
vize fmt --write src
```

关键选项：

| 选项                               | 描述                                      |
| ---------------------------------- | ----------------------------------------- |
| `--check`                          | 会变更的报告文件                          |
| `-w, --write`                      | 写格式化输出                              |
| `--single-quote`                   | 切换字符串引用样式                        |
| `--print-width`                    | 最大线宽                                  |
| `--tab-width`                      | 缩进宽度                                  |
| `--use-tabs`                       | 切换制表符与空格                          |
| `--no-semi`                        | 省略分号                                  |
| `--sort-attributes`                | 排序模板属性                              |
| `--single-attribute-per-line`      | 每行放置一个属性                          |
| `--max-attributes-per-line`        | 在给定属性计数                            |
| `--normalize-directive-shorthands` | 规范化`v-bind:` / `v-on:` / `v-slot:`速记 |
| `--profile`                        | 打印时序配置文件                          |

## 绒毛

```bash
vize lint src
vize lint --preset opinionated src
vize lint --help-level short src
```

关键选项：

| 选项                  | 描述                                                                              |
| --------------------- | --------------------------------------------------------------------------------- |
| `--fix`               | 从允许文本编辑的规则中应用安全的自动修复，然后报告剩余的诊断                      |
| `-f, --format`        | 输出格式：`text`、`ansi`、`plain`、`json`、`stylish`、`markdown`、`html`或`agent` |
| `--max-warnings`      | 警告超过限制时失败                                                                |
| `-q, --quiet`         | 仅节目摘要                                                                        |
| `--help-level`        | `full`、`short`或`none`                                                           |
| `--preset`            | `happy-path`、`opinionated`、`essential`、`incremental`或`nuxt`                   |
| `--cross-file`        | 启用选择加入的跨文件检查                                                          |
| `--cross-file-tree`   | 当启用跨文件线条时打印提供/注入树                                                 |
| `--strict-reactivity` | 启用本地检查器支持的反应性损失线条                                                |
| `--profile`           | 打印时序配置文件                                                                  |
| `--slow-threshold`    | 配置文件输出的慢文件阈值                                                          |

预设旨在分阶段采用：

| 预设          | 在                                       |
| ------------- | ---------------------------------------- |
| `essential`   | 你需要CI                                 |
| `happy-path`  | 你想要默认推荐的捆绑包                   |
| `opinionated` | 你需要更强的约定、脚本规则和类型感知候选 |
| `incremental` | 你只需要显式配置的规则                   |
| `nuxt`        | 你需要带有Nuxt组件假设的主观规则         |

示例：

```bash
vize lint --preset essential --max-warnings 0 src
vize lint --preset opinionated --help-level short src
vize lint --cross-file --cross-file-tree src
vize lint --strict-reactivity src
vize lint --format ansi src
vize lint --format plain src
vize lint --format agent src
vize lint --format markdown src
```

## 检查

```bash
vize check
vize check src
vize check --tsconfig tsconfig.app.json
vize check --profile src
```

`vize check`有通过[`corsa-bind`](https://github.com/ubugeeei/corsa-bind)展示的`vize_canon`和Corsa项目会议支持。Vize为Vue SFC生成虚拟TypeScript，在本地路径上运行项目诊断，并将结果映射回原始源位置。

当没有明确路径时，`vize check`使用`tsconfig.json` `files` / `include` /
如果有的话`exclude`。显式输入可以是文件、目录或块状，可以包括`.vue`，
`.ts`、`.tsx`和`.d.ts`。

关键选项：

| 选项                | 描述                                      |
| ------------------- | ----------------------------------------- |
| `-s, --socket`      | 连接运行中的`check-server`                |
| `--tsconfig`        | 覆盖`tsconfig.json`                       |
| `-f, --format`      | 输出格式：`text`或`json`                  |
| `--show-virtual-ts` | 打印生成的虚拟TypeScript                  |
| `-q, --quiet`       | 仅节目摘要                                |
| `--profile`         | 将配置文件伪影写入`node_modules/.vize`    |
| `--corsa-path`      | 覆盖 Corsa 可执行路径                     |
| `--servers`         | 保留的Corsa服务器数量;仅支持`1`           |
| `--declaration`     | 输出`.d.ts`                               |
| `--declaration-dir` | Output directory for emitted declarations |

当你想在开发 Vize 或测试 Corsa 时钉住自定义 Corsa 可执行文件时，可以使用 `--corsa-path`
本地`corsa-bind`结账。共享配置密钥为`typeChecker.corsaPath`;`typeChecker.tsgoPath`
仅作为兼容性别名保留。

实用的模式：

```bash
vize check --tsconfig tsconfig.app.json src
vize check --show-virtual-ts src/components/App.vue
vize check --profile src
vize check --declaration --declaration-dir dist/types
```

项目范围的模板值和 Vue 环境类型应通过 TypeScript 项目可见
配置。包含生成文件，如`auto-imports.d.ts`、`components.d.ts`或您自己的文件
Vue声明在`tsconfig.json`中，然后在需要时选择该项目，`--tsconfig`：

```json
{
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue", "src/**/*.d.ts"]
}
```

```ts
// src/types/vue-app.d.ts
declare module "vue" {
  interface ComponentCustomProperties {
    $t: (key: string) => string;
  }
}
```

## 警官

```bash
vize inspector src/App.vue
vize inspector "src/**/*.vue" --target ssr
vize inspector src --format json --output inspector-payload.json
vize inspector src --format agent --output inspector-agent.json
```

`vize inspector`将一个或多个`.vue`文件打包到游乐场消耗的有效载荷中
编译器检查员。浏览器随后检查Vue输出、Vize输出、虚拟TS、VIR以及
跨文件图，然后生成一个永久链接和一个预填充的拉取请求链接。

当其他本地工具或 AI 代理需要相同的复制品时，可以使用 `--format agent`
浏览器。报告包含具体的有效载荷、游乐场网址、汇总指标和导入图表。
有效载荷、图和线差元数据由本地专用`vize_curator`箱构建，因此CLI和
游乐场检查保持对齐。

关键选项：

| 选项                | 描述                               |
| ------------------- | ---------------------------------- |
| `-f, --format`      | 输出格式：`url`、`json`或`agent`   |
| `--target`          | 编译器目标：`dom` 或 `ssr`         |
| `--playground-url`  | Playground生成链接的基础链接       |
| `--max-files`       | 批处理有效载荷中包含的限制文件     |
| `--custom-renderer` | 启用自定义渲染器比较               |
| `--template-syntax` | 选择`standard`、`strict`或`quirks` |
| `-o, --output`      | 将URL或JSON负载写入文件            |

关于贡献者的工作流程，请参见[编译器检查器](./compiler-inspector.md)。

## 干净

```bash
vize clean
vize clean --dry-run
vize clean --scope node-modules
vize clean --scope project
vize clean --force
vize clean path/to/project
```

`vize clean` 移除已知的 Vize 拥有的本地文件，然后移除所选项目根节点
空荡荡的`.vize`和`node_modules/.vize`的父母。管理工件列表涵盖配置文件输出，
Musea报告/快照/令牌、Patina会话、配置模式、LSP日志、套接字剩余、OXC
转储、Oxlint 绕过文件以及实体化的 Corsa 项目文件。未知条目`.vize`
默认保留;只有在选定的工件根应被移除时才使用`--force`
批发。`--dry-run`打印出将被移除的遗迹路径。用`--scope node-modules`
或者`--scope project`只清理一个神器根部。

## 准备好了

```bash
vize ready src
vize ready --output dist src
```

`vize ready`按顺序运行`fmt --write`、`lint`、`check`和`build`。命令停止于
第一步失败。

关键选项：

| 选项           | 描述                      |
| -------------- | ------------------------- |
| `-o, --output` | 构建步骤的输出目录        |
| `--ssr`        | 启用SSR编译以构建         |
| `--script-ext` | `preserve`或`downcompile` |

## 升级

```bash
vize upgrade
vize upgrade --dry-run
```

默认情况下，`vize upgrade`通过Vite+更新npm包：

```bash
vp install -D vize@latest
```

只用`--source cargo`来做明确的本地货运安装。

## 博物馆

```bash
vize musea --help
vize musea serve --port 6006
vize musea new
```

`musea`分指挥部目前专注于脚手架和实验性进入点。
对于日常画廊开发，目前推荐的工作流程是
`@vizejs/vite-plugin-musea`。

npm 包还提供了一个方便`vize musea`命令，可以运行 Vite 和 Musea 一起运行
项目中安装的插件：

```bash
vp exec vize musea
vp exec vize musea --build
```

## LSP和IDE

```bash
vize lsp
vize lsp --port 9527
vize ide vscode
vize ide zed
```

`vize lsp`直接启动语言服务器。
`vize ide` 为 VS Code 和 Zed 添加了编辑器专用的安装和管理命令
整合。

## 全球期权

```bash
vize --help
vize --version
vize <command> --help
```
