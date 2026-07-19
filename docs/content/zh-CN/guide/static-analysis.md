---
title: 静态分析
---

<!-- Generated translation; source: guide/static-analysis.md -->

# 静态分析

Vize的分析栈由编译器、linter、类型检查器、编辑器服务器和Musea共享
工具。目标是解析一次Vue SFC，保留丰富的语义信息，并重复利用
用于诊断和代码生成，而不是把每个命令当作独立的工具。

以下示例假设`vize` npm包是通过项目脚本安装并调用的，这些脚本
是应用程序推荐的工作流程。

## 管道

| 图层     | 它的作用                                                         | 由                               |
| -------- | ---------------------------------------------------------------- | -------------------------------- |
| 骨架     | 分词化并解析Vue模板和SFC结构                                     | 编译器，LINTER，格式化器         |
| 克罗奎斯 | 构建作用域、绑定元数据、宏信息和跨文件图表                       | 编译器、lint、类型识别检查       |
| 铜绿     | 运行 Vue、脚本、CSS、a11y、SSR、Vapor、Musa 和类型识别 lint 规则 | `vize lint`，编辑诊断，Oxlint 桥 |
| 正史     | 生成虚拟 TypeScript，并将诊断映射回 Vue 文件                     | `vize check`，编辑器类型检查     |
| 指挥     | 通过LSP                                                          | `vize lsp`，VS Code，Zed         |

这意味着静态分析不仅仅是绒毛。模板绑定、编译器宏、组件
元数据、提供/注入关系、反应流、生成的虚拟 TypeScript，以及
组件画廊的元数据都依赖于相同的底层分析工作。

关于具体的规则名称、默认值和可以输出的跨文件诊断代码，请参见
[规则](../rules/index.md)。

## 绒毛

从默认预设开始：

```json
{
  "scripts": {
    "vize:lint": "vize lint src"
  }
}
```

```bash
vp run vize:lint
```

仅正确性CI使用`essential`，`happy-path`默认推荐的捆绑包，
`opinionated`当你想要更强的约定时，`nuxt` Nuxt感知假设，
`incremental`你只想让明确配置的规则运行。

```json
{
  "scripts": {
    "vize:lint:ci": "vize lint --preset essential --max-warnings 0 src",
    "vize:lint:opinionated": "vize lint --preset opinionated --help-level short src",
    "vize:lint:fix": "vize lint --fix src",
    "vize:lint:json": "vize lint --format json src"
  }
}
```

```bash
vp run vize:lint:ci
vp run vize:lint:opinionated
vp run vize:lint:fix
vp run vize:lint:json
```

只有在基本lint路径稳定后，才选择进行跨文件和类型识别检查：

```json
{
  "scripts": {
    "vize:lint:cross-file": "vize lint --cross-file src",
    "vize:lint:cross-file-tree": "vize lint --cross-file --cross-file-tree src",
    "vize:lint:strict-reactivity": "vize lint --strict-reactivity src"
  }
}
```

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
vp run vize:lint:strict-reactivity
```

跨文件线条分析了一系列
Vue文件。`--strict-reactivity`启用了本地的检测器支持的反应性损失规则，所以可以期待它
比普通模板和脚本 lint 规则慢。

## 反应性叠加

Croquis为每个分析的SFC暴露出稳定的反应性叠加：反应源、`.value`
需求、反应性损失位点和带源映射的效应图边。同一个紧凑
JSON 模型提供诊断、报告、编辑器表面以及 Playground 的**Reactivity**标签页。

## 铜绿规则模型

铜绿层是绒毛的规则层。规则是小访客通过SFC源代码模板根，
模板元素、指令、 `v-for`、`v-if` 和插值。每条规则都包含 的元数据
规则名称、类别、默认严重度、帮助文本以及是否可修复。预设只是
决定哪些规则可以一起启用的注册机构。

| 面积         | 示例规则                                                                                     | 内容涵盖                           |
| ------------ | -------------------------------------------------------------------------------------------- | ---------------------------------- |
| Vue 正确性   | `vue/require-v-for-key`，`vue/valid-v-model`，`vue/no-use-v-if-with-v-for`                   | 本地于一个组件的模板语义           |
| Vue 安全     | `vue/no-v-html`，`vue/no-unsafe-url`                                                         | XSS倾向的HTML和URL汇入             |
| Vue结构      | `vue/sfc-element-order`，`vue/require-scoped-style`，`vue/no-unused-components`              | SFC形状、组件使用及可维护性        |
| 文字惯例     | `script/no-options-api`，`script/no-get-current-instance`，`script/prefer-import-from-vue`   | Vue Composition API 和编译器宏约定 |
| CSS          | `css/no-important`，`css/no-hardcoded-values`，`css/prefer-logical-properties`               | 样式块和设计系统友好的CSS          |
| 无障碍       | `a11y/img-alt`，`a11y/anchor-has-content`，`a11y/label-has-for`                              | 可访问标记与交互模式               |
| HTML         | `html/deprecated-element`，`html/id-duplication`，`html/no-empty-palpable-content`           | HTML 有效性与语义标记              |
| SSR          | `ssr/no-browser-globals-in-ssr`，`ssr/no-hydration-mismatch`                                 | 服务器/客户端渲染危害              |
| 蒸汽         | `vapor/no-vue-lifecycle-events`，`vapor/no-inline-template`，`vapor/require-vapor-attribute` | 面向蒸汽的模板约束                 |
| 博物馆       | `musea/require-title`，`musea/valid-variant`，`musea/prefer-design-tokens`                   | 组件画廊与变体创作                 |
| 类型感知分析 | `type/require-typed-props`，`type/require-typed-emits`，`type/no-reactivity-loss`            | 需要语义或跳棋支持上下文的规则     |

内置预设旨在分阶段支持采用：

| 预设          | 形状                                              |
| ------------- | ------------------------------------------------- |
| `essential`   | 以错误为中心的Vue正确性、安全性及最小化的HTML检查 |
| `happy-path`  | 默认捆绑包：正确性、安全性、a11y、SSR、语义检查   |
| `opinionated` | `happy-path` 加上更强的约定、脚本规则和类型规则   |
| `nuxt`        | 针对Nuxt自动导入假设调整的观点规则                |
| `incremental` | 主机驱动、逐条规则采用的空白起点                  |

## 移民规范与习俗规则

Patina 接受现有的 ESLint 禁用语用来匹配规则名，包括
`eslint-disable`、`eslint-enable`、`eslint-disable-next-line`和`eslint-disable-line`。这使得
项目迁移规则如`vue/require-v-for-key`而无需重写每一条抑制评论
前面。

项目本地的JavaScript规则模块还不是稳定的Vize运行时API。迁徙期间，保持
这些规则在ESLint或Oxlint中运行，并与`vize lint`并列运行，或者使用`incremental`预设
只启用已经与你政策相匹配的内置 Vize 规则。`rules`配置对象控件
内置的 Vize 通过名称规则严重度。

对于禁止运行时环境全局（典型的sidecar-ESLint规则，如
`no-access-process`、`no-access-local-storage`或`no-restricted-globals`对`localStorage` /
`sessionStorage`），启用自愿加入的内置`script/no-restricted-globals`规则，而不是保留
只为这些人安装了ESLint。其默认拒绝列表为`process`、`localStorage`和
`sessionStorage`，在每一个裸露的参考文献中都有报道。

两个脚本规则也接受项目本地配置，`linter.ruleOptions`（#1891），因此团队
可以通过`vize lint`强制执行自己的架构规范。`script/no-restricted-globals`
接受一个`globals`列表，**替换**内置默认列表;`script/no-restricted-members`是
关闭直到配置，并标记`<object>.<property>` `members`列表中的访问。选项被输入
（`name` / `object` / `property` 加上一个可选的 `message`，拒绝未知键）;一个失踪的
`message`只能依赖通用的建议。

```json
{
  "linter": {
    "rules": {
      "script/no-restricted-globals": "error",
      "script/no-restricted-members": "error"
    },
    "ruleOptions": {
      "script/no-restricted-globals": {
        "globals": [
          { "name": "process", "message": "Read env via a typed helper." },
          { "name": "alert" }
        ]
      },
      "script/no-restricted-members": {
        "members": [
          { "object": "window", "property": "localStorage", "message": "Use authStorage." }
        ]
      }
    }
  }
}
```

## 跨档规则

跨文件分析者居住在克罗奎斯，通过铜绿诊断暴露于绒毛现象。确实如此
选择加入，因为它构建模块注册表、导入图、组件使用图等
所有分析的Vue文件的索引。

如今，`vize lint --cross-file`支持提供/注入匹配、唯一元素ID检查，
反应性追踪和异步竞赛-条件分析。`--cross-file-tree` 印刷
在这些诊断之上提供/注入树。

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
```

较低层级的跨文件引擎比当前的CLI表面更宽泛：

| 跨文件选项                | 预期诊断或事实                                   |
| ------------------------- | ------------------------------------------------ |
| `provide_inject`          | 未匹配注入、未使用供给、字符串键警告、非反应流   |
| `unique_ids`              | 循环中引入的重复ID和非唯一ID                     |
| `reactivity_tracking`     | 道具结构、混叠与跨组分反应性损失                 |
| `race_conditions`         | 异步状态更新可以快速切换已提供或共享状态         |
| `fallthrough_attrs`       | `$attrs`、`inheritAttrs`和多根倒塌危险           |
| `component_emits`         | 未申报的发射台、未使用的发射台和没有制作人的听众 |
| `event_bubbling`          | 在组件边界中冒泡而未被处理的事件                 |
| `server_client_boundary`  | 浏览器API使用及SSR/客户端边界下的水合风险        |
| `error_suspense_boundary` | 无实用的延迟或误差边界的异步分量                 |
| `circular_dependencies`   | 导入周期与深度导入链                             |
| `component_resolution`    | 未注册或未解决组件使用                           |
| `props_validation`        | 缺少必需道具和子道具类型不匹配                   |

其指导原则是默认保持单文件linting快速，明确暴露跨文件组，如下
它们成熟，并将高置信度项目事实导入与
CLI、Oxlint 桥接器和编辑器服务器。

## 类型检查

`vize check`为Vue SFC生成虚拟TypeScript，并请求Corsa项目会话
诊断。它检查`.vue`、`.ts`、`.tsx`和`.d.ts`输入，并将诊断映射回
原始源文件。

```json
{
  "scripts": {
    "vize:check": "vize check",
    "vize:check:src": "vize check src",
    "vize:check:app": "vize check --tsconfig tsconfig.app.json",
    "vize:check:json": "vize check --format json --quiet",
    "vize:check:virtual-ts": "vize check --show-virtual-ts src/components/App.vue",
    "vize:check:profile": "vize check --profile src",
    "vize:check:single-server": "vize check --servers 1 src",
    "vize:check:declarations": "vize check --declaration --declaration-dir dist/types"
  }
}
```

```bash
vp run vize:check
vp run vize:check:src
vp run vize:check:app
vp run vize:check:json
```

当没有路径时，`vize check`读取`tsconfig.json` `files`、`include`和`exclude`
如果有项目配置可用，则填写字段。调试生成代码时使用`--show-virtual-ts`
`--profile`需要时序和虚拟文件伪影时`node_modules/.vize`。

```bash
vp run vize:check:virtual-ts
vp run vize:check:profile
vp run vize:check:single-server
```

声明输出可从具体化检查器项目获得：

```bash
vp run vize:check:declarations
```

项目范围的模板值和生成的声明文件应通过TypeScript可见
项目配置。把环境声明放在你`tsconfig`的路径下，然后通过
需要时，该项目文件发送给检查器：

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
    $route: { path: string };
  }
}
```

```bash
vp run vize:check:app
```

## npm 包脚本 vs Rust CLI

npm `vize` 包用于包脚本，使用打包的 NAPI 绑定：

```json
{
  "scripts": {
    "vize:lint": "vize lint src",
    "vize:check": "vize check src --strict",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:lint
vp run vize:check
vp run vize:ready
```

Rust CLI 目前拥有更完整的项目支持类型检查表面：

```bash
nix run github:ubugeeei-prod/vize#vize -- check --tsconfig tsconfig.app.json --profile src
vize check --tsconfig tsconfig.app.json --profile src
vize lsp
```

当你想在应用中安装可安装的工作流程时，可以使用 npm 包脚本。使用 Rust CLI，当
你需要`check-server`、集成集成电路（LSP）、IDE管理，或者Corsa支持的项目诊断路径
Vue和TypeScript文件。

## 牛茚

当你的团队已经运行Oxlint并希望在`oxlint-plugin-vize`
同一条命令：

```bash
vp install -D oxlint oxlint-plugin-vize
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "preset": "essential",
      "helpLevel": "short"
    }
  },
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn"
  }
}
```

## 收养之路

1. 向CI添加`vize:lint:ci`包脚本，如`vize lint --preset essential src`。
2. 正确性诊断结果干净后切换到`happy-path`或`opinionated`。
3. 在你的项目`tsconfig.json`中添加一个`vize:check`包脚本。
4. 先启用编辑器线条，然后在CI输出稳定后进行类型检查。
5. 为需要深入分析的项目添加跨文件和严格的反应性检查。

对于单一质量门，运行 `vize ready src` 的 `vize:ready` 包脚本会依次执行 `fmt

- -write`、`lint`、`check`和`build`，并在第一个失败的步骤停止。
