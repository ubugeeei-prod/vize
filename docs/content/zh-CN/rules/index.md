---
title: 规则
description: 用预设启用 Vize 的 lint 规则，再逐条调整。
---

<!-- Generated translation; source: rules/index.md -->

# 规则

Vize 的 linter 检查 Vue 单文件组件：模板、脚本、样式以及文件之间的关系。先选择一个预设，再按需调高、调低或关闭单条规则。

## 启用 lint

在 Vite+ 中，从 `@vizejs/vite-plugin/vite-plus` 导入 `defineConfig`，配置 `vue.linter`，然后运行 `vp run lint`。该命令会同时运行 Vize 的规则和 Oxlint。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
    linter: {
        preset: "happy-path",
        rules: {
            "vue/no-v-html": "off",
            "a11y/alt-text": "error",
        },
    },
});
```

每条规则可设为 `off`、`warn` 或 `error`。带设置项的规则从 `vue.linter.ruleOptions` 读取设置，见[规则选项](/rules/options.md)。

如果项目中已有名为 `lint` 的脚本，请改用 `vp run vize:lint`。内置的 `vp lint` 是 Vite+ 自己的检查器，不会运行 Vize 的规则。

## 预设

预设是一组起始规则，默认是 `happy-path`。

| 预设 | 规则数 | 启用的内容 |
| --- | ---: | --- |
| `essential` | 85 | 模板与指令的有效性、Vue 3 中移除的语法和 API（`vue/no-deprecated-*`、`script/no-deprecated-*`）、脚本宏、`nextTick` 和 Options API 的正确性、在 `await` 之后注册的生命周期钩子、侦听器和 `expose`、从 `vue` 而非 `@vue/*` 导入、未使用的组件、`vue/no-v-html`、`vue/no-unsafe-url`、`html/id-duplication` |
| `happy-path`（默认） | 147 | `essential`，加上模板格式约定（包括 `v-bind` 简写和连字符形式的组件事件）、无障碍、HTML 规范、SSR 风险、未使用的属性、已声明的 emits，以及带类型的 props 和 emits |
| `ecosystem` | 155 | `happy-path`，加上生态系统库的检查 |
| `nuxt` | 213 | 针对 Nuxt 自动导入调整过的 `opinionated` |
| `opinionated` | 213 | `happy-path`，加上更严格的约定、带类型的 props、基于类型的 `defineProps`/`defineEmits`、`<script setup>` 宏的顺序、全小写事件名、`useTemplateRef`、`defineSlots`、props 默认值与 emit 的规范、`defineOptions`，以及 `css/*` 样式规则 |
| `incremental` | 0 | 无。只想启用指定规则时从这里开始。 |

规则数不含跨文件规则。当 `compiler.compatibility.vueVersion` 为 `2` 或 `2.7` 时，Vue 3 移除项的检查会关闭。

## 查找规则

| 页面 | 内容 |
| --- | --- |
| [全部规则](./all.md) | 按类别列出全部规则，并给出启用每条规则的最小预设 |
| [Vue](./vue.md) | SFC 模板、指令、组件、代码块以及 Vue 3 迁移 |
| [类型与脚本](./type-and-script.md) | `<script setup>`、编译器宏、响应式以及使用类型信息的检查 |
| [HTML](./html.md) | HTML 有效性与语义化标记 |
| [无障碍](./accessibility.md) | ARIA、键盘交互、标签、地标和媒体替代内容 |
| [SSR](./ssr.md) | 服务端渲染与水合的风险 |
| [Vapor](./vapor.md) | 以 Vapor 模式编译的组件的限制 |
| [生态系统与 Nuxt](./ecosystem.md) | Nuxt、Vue Router、Pinia、Vue I18n、Vue Test Utils 和 Void Vue |
| [CSS](./css.md) | 普通 CSS 的正确性和可维护性检查 |
| [Musea](./musea.md) | Musea 的 `.art.vue` 故事、变体和设计令牌 |
| [petite-vue](./petite-vue.md) | 使用 petite-vue 的 HTML 页面 |
| [跨文件](./cross-file.md) | 跨组件、provide/inject 和路由的检查，用 `crossFile: true` 启用 |
| [规则选项](/rules/options.md) | 带类型的 `ruleOptions` 及其默认值 |
| [从 ESLint 迁移](/rules/migration.md) | `eslint-plugin-vue` 规则名与 Vize 的对应关系及差异 |

每个规则页面先说明规则检查的内容、默认严重程度和所属预设，再给出会被报告的错误示例和不会被报告的正确示例，各附一句原因，最后是启用该规则的配置。这些示例与 linter 测试实际运行的文件相同。

## 使用类型信息的规则

`type/` 下的规则会从 `tsconfig.json` 的 TypeScript 项目中读取类型信息。除了用预设或 `rules` 启用之外，还需要在 `vue.linter` 中设置 `typeAware: true`。共享的环境名称请在 `compilerOptions.types` 或项目引用中声明，不要在 Vize 配置里另建 `globals` 列表。
