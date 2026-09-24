---
title: 源码分发 (vize lib)
---

<!-- Generated translation; source: guide/lib-pull.md -->

# 源码分发 (`vize lib`)

`vize lib` 以 shadcn/ui 的方式，把 `@vizejs/ui` 的组件和 `@vizejs/composable` 的组合式函数 **作为源码** 复制到你的项目中。
复制后的文件归你所有：可以随意修改，Vize 会记录每个文件来自哪个包版本，让升级保持安全。

```bash
vpx vize lib pull rating
```

这会把 `rating` 系列以及它导入的所有内容 (`controllable-state`、`id` 等) 写入 `src/components/vize/`，
并在 `vize-lib.lock.json` 中记录这次拉取。

## 源码从哪里来

每个已发布的 `@vizejs/ui` 和 `@vizejs/composable` tarball 都包含一个带版本的注册表：

```text
node_modules/@vizejs/ui/
  registry/
    registry.json          # 条目、文件、sha256 摘要、依赖
    files/families/...     # 原始 .vue / .ts / .css 源码（不含测试）
```

`vize lib` 按以下顺序解析注册表：

1. `--registry <path>`：`registry.json`、其所在目录或已解包的包目录。重复该参数即可同时传入 ui 和 composable
   注册表，此时自动发现会被关闭。
2. 项目中已安装的包 (`node_modules/@vizejs/ui/registry/registry.json`，与 Node 解析一样向上查找父目录)。
3. 当请求固定了与已安装版本不同的版本，或包尚未安装 (此时为 `@latest`) 时，执行
   `npm pack @vizejs/<pkg>@<version>` 到临时目录并用 `tar -xzf` 解包。使用 `--offline` 可禁止这一步。

不涉及任何注册表服务器或额外的 HTTP 客户端：注册表就是 npm 已经提供的 tarball，因此每个版本都不可变且可复现。

## 命令

| 命令                                     | 作用                                                                       |
| ---------------------------------------- | -------------------------------------------------------------------------- |
| `vize lib init [--dry-run]`              | 检测项目结构并写入 `lib` 配置部分。                                        |
| `vize lib list [--kind ui\|composable]`  | 列出可拉取的条目。                                                         |
| `vize lib search <words>`                | 按名称、标题、描述和别名搜索。                                             |
| `vize lib info <name>`                   | 显示文件、注册表依赖、npm peer 和包版本。                                  |
| `vize lib pull <item>... [--dir <dir>]`  | 复制条目及其注册表依赖；`--dry-run`、`--overwrite`。                       |
| `vize lib add <item>...`                 | 与 shadcn 兼容的 `pull` 别名 (`-p/--path`、`-o/--overwrite`、`-y/--yes`)。 |
| `vize lib status`                        | 将已拉取的文件与锁文件和已安装的注册表进行比较。                           |
| `vize lib diff <name> [--to <version>]`  | 显示从本地副本到注册表版本的 unified diff。                                |
| `vize lib update [<name>...] [--to <v>]` | 应用上游变更而不覆盖本地修改；`--dry-run`、`--force`。                     |
| `vize lib remove <name>...`              | 删除条目以及不再被需要的依赖；`--dry-run`、`--force`。                     |
| `vize lib outdated`                      | 将锁定的版本与已安装和最新的注册表进行比较。                               |

所有命令都支持 `--json` 输出机器可读结果，以及 `--root <dir>` 以针对其他项目运行。

### 指定条目

条目可以通过规范名称 (`rating`)、别名 (`star rating`、`useToggle`)、在两个包中同名时加种类前缀
(`ui:locale`、`composable:locale`)，以及精确的包版本 (`rating@0.427.0`、`composable:use-toggle@0.427.0`) 来指定。

### 目标目录

拉取的文件在每个种类各自的一个目录下保持注册表布局 (`families/form/rating/rating.vue`、
`foundations/id/deterministic-id.ts` 等)，因此条目之间的相对导入无需改写即可工作。目录按以下顺序确定：

1. `--dir <dir>` (必须位于项目内)
2. `vize.config.*` 中的 `lib` 部分
3. 注册表默认值：`src/components/vize` (ui) 和 `src/composables/vize` (composable)

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  lib: {
    uiDir: "src/ui/vendor",
    composableDir: "src/composables/vendor",
    // dir: "src/vendor",        // 两种类型共用的回退目录
    // lockfile: "vize-lib.lock.json",
  },
});
```

某个种类一旦拉取到某个目录，之后该种类的拉取都会复用它；会拆分依赖图的冲突 `--dir` 会被拒绝。

拉取的源码只导入相对路径和 `vue` 等 npm 包。`pull` 会报告 `package.json` 尚未声明的 npm 依赖，但不会替你安装。

## 快速开始：`init`

```bash
vize lib init --dry-run   # 显示检测到的结构和配置变更
vize lib init             # 写入
```

`init` 会检测源码目录 (`src/`，Nuxt 4 项目为 `app/`) 和 TypeScript，然后写入 `lib.uiDir` / `lib.composableDir`。
没有配置时创建 `vize.config.json`；已有 `vize.config.json` 时只追加 `lib` 部分而不改动其余内容；对于
`vize.config.ts` / `.pkl` 则打印代码片段而不编辑代码。已有的 `lib` 部分会被保留，除非使用 `--force`。
若 `tsconfig.json` 缺少 `allowImportingTsExtensions`，`init` 会给出提示：拉取的源码以 `./x.ts` 形式导入同级文件。

## 检查更新：`outdated`

`vize lib outdated` 列出注册表与锁文件不一致的已拉取条目：

| 列        | 含义                                                                                   |
| --------- | -------------------------------------------------------------------------------------- |
| `current` | `vize-lib.lock.json` 中记录的版本。                                                    |
| `wanted`  | `update` 将使用的注册表版本 (已安装的包，否则为最新)。                                 |
| `latest`  | npm 上发布的最新版本 (`npm view`；`--offline` 时跳过)。                                |
| `state`   | `update-available`、`newer-release`、`removed-upstream`、`unknown` (或 `up-to-date`)。 |

`update-available` 表示条目的 `contentHash` 不同；不影响条目文件的版本升级仍为 `up-to-date`。`--json` 包含所有条目。

## 第三方注册表

任何包或站点都可以发布相同格式的注册表，并通过命名空间使用：

```json
{
  "lib": {
    "registries": {
      "@acme": "npm:@acme/vue-kit",
      "@design": { "source": "https://design.example.com/r/registry.json", "dir": "src/design" },
      "@local": { "source": "./registry", "dir": "src/local" }
    }
  }
}
```

```bash
vize lib pull @acme/data-table @design/button@2.1.0
vize lib list --kind @acme
```

- `npm:<package>[@range]` 使用已安装包的 `registry/registry.json`，否则执行 `npm pack`。
- `https://…/registry.json` 通过 `curl` 获取，每个文件按需从同级的 `files/<path>` 下载 (仅 https)。
- 其他值是相对于配置文件的路径：`registry.json`、其所在目录或包目录。

第三方注册表使用与官方注册表相同的 JSON Schema 校验 (未知字段、格式错误的摘要、未知的角色或依赖、不完整的依赖闭包都会被拒绝)，
下载的每个字节在写入前都会用 SHA-256 校验。命名空间的条目放在其 `dir` (否则为注册表的 `defaultTargetDirectory`)，
以 `@namespace` 为键锁定，并且永远不会覆盖其他已拉取条目拥有的文件。

## 版本管理与安全更新

`vize-lib.lock.json` (请提交它) 为每个条目记录来源包及其精确版本、注册表 `contentHash`、是直接请求还是作为依赖引入，
以及拉取时每个文件的 SHA-256。这些摘要是 **你的文件**、**拉取时的文件** 与 **新的注册表文件** 三方比较的合并基准。

| 你的文件 vs 拉取时 | 注册表 vs 拉取时 | `update` / `pull` 的行为                              |
| ------------------ | ---------------- | ----------------------------------------------------- |
| 未改动             | 未改动           | 不做任何事 (`unchanged`)                              |
| 未改动             | 已改动           | 替换 (`update`)                                       |
| 已修改             | 未改动           | 保留你的修改 (`keep-local`)                           |
| 已修改             | 已改动           | 除非 `--force` / `--overwrite`，否则拒绝 (`conflict`) |
| 未改动             | 上游已删除       | 删除 (`delete`)                                       |
| 已修改             | 上游已删除       | 除非 `--force`，否则拒绝 (`conflict-delete`)          |
| 缺失               | 任意             | 恢复 (`create`)                                       |
| 存在但未锁定       | 任意             | 除非 `--overwrite`，否则拒绝 (`conflict`)             |

只要还有未解决的冲突就不会写入任何内容，因此被拒绝的更新不会改动文件和锁文件。用
`vize lib diff <name> --to <version>` 查看上游变更，手动合并后再运行 `update --force`。

典型的升级流程：

```bash
pnpm add @vizejs/ui@latest       # 或：vize lib update --to 0.428.0
vize lib status                  # 哪些条目有更新或本地修改
vize lib update --dry-run        # 预览文件操作
vize lib update                  # 应用；如有冲突会列出
```

`remove` 会删除条目以及仅为它拉取的所有依赖。只要其他已拉取的条目仍导入目标就会拒绝，并且除非使用 `--force`，
否则会保留本地修改过的文件。

## 注册表格式

注册表文档由
[`vize-lib-registry.schema.json`](https://github.com/ubugeeei-prod/vize/blob/main/npm/cli/schemas/vize-lib-registry.schema.json)
描述，锁文件由
[`vize-lib-lock.schema.json`](https://github.com/ubugeeei-prod/vize/blob/main/npm/cli/schemas/vize-lib-lock.schema.json)
描述。两者都随 `vize` npm 包发布在 `schemas/` 下。

- `registryDependencies` 是构建时根据相对导入图计算出的完整传递闭包；共享的基础模块是独立条目，
  因此拉取两个都需要 `id` 的组件时只会复制一次 `id`。
- 每个文件都带有 `sha256`；`vize lib` 会用它校验复制的每个字节。
- `contentHash` 只在条目的文件变化时改变，因此 `status` 只会对不同版本间源码确实不同的条目报告 "update available"。
