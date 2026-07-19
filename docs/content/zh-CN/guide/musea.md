---
title: 博物馆
---

<!-- Generated translation; source: guide/musea.md -->

# 博物馆

> **⚠️ 进行中的作品：**Musea仍在不断发展。文件格式、API 和界面行为可能会发生变化。

Musea是Vize的艺术文件和组件画廊工具链。

- `vize_musea` 是用于解析`*.art.vue`、生成文档、构建道具调色板的 Rust 核心，
  自动生成变体，并准备VRT数据。
- `@vizejs/vite-plugin-musea` 是目前推荐的画廊和开发服务器工作流程。
- `musea-vrt` 是用于可视化回归快照、a11y 审计、审批、清理和
  生成的艺术文件。

## 概述

![Musea Component Gallery — 主页](/musea-home.png)

Musea 使用`*.art.vue`文件描述具有 Vue 原生语法的组件变体。

## 安装

从[Vite+安装指南](https://viteplus.dev/guide/install)中安装一次`vp`，然后添加以下包：

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

## 推荐用法：Vite 插件

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
      previewSetup: "musea.preview.ts",
    }),
  ],
});
```

运行你普通的 Vite 开发服务器，打开配置好的 Musea 路由：

```bash
vp dev
```

```txt
http://localhost:5173/__musea__
```

如果你安装了`vize` npm包，`vp exec vize musea`是Vite的一个便利包装器：

```bash
vp exec vize musea
vp exec vize musea --build
```

## 共享配置

`musea()`选项覆盖共享配置。把稳定的项目默认设置在`vize.config.ts`，保持
`vite.config.ts`中仅有预览设置。

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  musea: {
    include: ["src/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
    basePath: "/__musea__",
    storybookCompat: false,
    inlineArt: false,
  },
});
```

共享配置目前涵盖`include`、`exclude`、`basePath`、`storybookCompat`
`inlineArt`。传递`previewCss`、`previewSetup`、`tokensPath`、`theme`和`storybookOutDir`
直接给`musea()`。

## 艺术档案

```art-vue
<script setup lang="ts">
import { ref } from "vue";

defineArt("./MyButton.vue", {
  title: "MyButton",
  category: "Components",
  status: "ready",
  tags: ["button", "ui", "input"],
});

const pressed = ref(false);
</script>

<art>
  <variant name="Default" default>
    <MyButton type="button" :pressed="pressed">Click me</MyButton>
  </variant>

  <variant name="Outlined">
    <MyButton type="button" outlined :pressed="pressed">Click me</MyButton>
  </variant>
</art>
```

`defineArt(source, options)` 是一个编译器宏。它声明了 Musea 应加载的组件，
还有以前存在`<art>`的元数据。偏好使用相对分量路径串，如
`defineArt("./MyButton.vue", { title: "MyButton" })`;Musea会导入生成的组件
运行时代码和语言服务器在prop和slot推理中使用相同的源代码。
源字符串参与路径补全、未解析文件诊断、文档链接和
必看定义。

`<art title="..." component="...">`仍然兼容，并且有明确的`<art>`属性
当两者同时存在时，覆盖`defineArt`元数据。

### 变体地方国家

根`<script setup>`状态默认是每个变体的隔离状态。每个变体都有自己的设置
实例，这样一个变体中的参考和计算值不会泄漏到另一个变体中：

```art-vue
<script setup lang="ts">
import { computed, ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>

<art>
  <variant name="Base" default>
    <Counter :count="count" />
  </variant>
  <variant name="Doubled">
    <Counter :count="doubled" />
  </variant>
</art>
```

只有当艺术文件需要一个共享设置时才使用`<script setup isolate="false">`
所有变体的实例：

```art-vue
<script setup lang="ts" isolate="false">
import { ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const sharedCount = ref(0);
</script>
```

### 解剖学

| 元素 / 宏                        | 目的                 |
| -------------------------------- | -------------------- |
| `defineArt(source, options)`     | 目标组件与艺术元数据 |
| `defineArt(...).title`           | 显示名称             |
| `defineArt(...).category`        | 侧边栏分组           |
| `defineArt(...).status`          | 可选身份徽章         |
| `defineArt(...).tags`            | 搜索和筛选标签       |
| `<script setup>`                 | 变体本地设置状态默认 |
| `<script setup isolate="false">` | 所有变体共享设置状态 |
| `<art>`                          | 根变体块             |
| `<art title component ...>`      | 兼容性元数据属性     |
| `<variant>`                      | 命名分量变体         |
| `default`                        | 标记默认变体         |
| `args`，`viewport`，`skip-vrt`   | 可选变体配置         |

当变体是组件合同的一部分时，保持美术文件靠近组件：

```txt
src/components/Button.vue
src/components/Button.art.vue
```

当设计系统拥有许多横切实例时，使用单独的`stories`或`art`目录，
或者当Nuxt组件自动发现扫描组件目录时：

```txt
src/components/Button.vue
stories/forms/Button.art.vue
stories/navigation/Menu.art.vue
```

## 内联艺术

启用`inlineArt`时，包含`<art>`块的普通`.vue`文件可以出现在
画廊。这对于小型组件非常有用，因为示例应存在于同一文件中。

```ts
musea({
  inlineArt: true,
});
```

在内联艺术中，使用`<Self>`来渲染主机组件。

## 画廊特色

![Musea 组件细节 — 变体](/musea-component.png)

Musaa可以浮现：

- 组件和变体元数据
- 道具调色板生成
- 设计标记视图
- 无障碍检查
- 视觉回归测试辅助工具
- 按需输出与故事书兼容的输出

## 道具调色板

![博物馆道具面板](/musea-props.png)

调色板流水线可以从组件元数据和美术定义中推断交互式控制。

## 设计代币

![博物馆设计代币](/musea-tokens.png)

`@vizejs/vite-plugin-musea`可以导入一个与样式字典兼容的令牌文件，并在
画廊界面。

```ts
musea({
  tokensPath: "src/tokens.json",
});
```

## 预览配置

你可以注入项目CSS和预览设置代码：

```ts
musea({
  previewCss: ["src/styles/main.css", "src/styles/musea-preview.css"],
  previewSetup: "musea.preview.ts",
});
```

这对于在预览 iframe 中安装 `vue-i18n` 或 `vue-router` 等插件非常有用。

```ts
// musea.preview.ts
import type { App } from "vue";
import { createI18n } from "vue-i18n";

export default function setup(app: App) {
  app.use(
    createI18n({
      legacy: false,
      locale: "en",
      messages: {
        en: {},
      },
    }),
  );
}
```

## 视觉回归测试

该包暴露了`musea-vrt`二进制：

```bash
vp exec musea-vrt --base-url http://localhost:5173
vp exec musea-vrt --update
vp exec musea-vrt --ci --json
vp exec musea-vrt --a11y
vp exec musea-vrt approve
vp exec musea-vrt approve "Button/*"
vp exec musea-vrt clean
```

典型的CI流程在一个进程启动Vite服务器，然后对该进程运行快照命令：

```bash
vp dev --host 0.0.0.0
vp exec musea-vrt --base-url http://localhost:5173 --ci --json
```

工作流程是：在快照目录下提交基线，`musea-vrt --ci --json`对
运行开发服务器，然后检查`vrt-report.json`/`vrt-report.html`加`snapshots/current`
`snapshots/diff`失败。重新运行时`--update`（或部分变体`approve`）
有意识的修改，并且在移除美术文件后运行`clean`，这样陈旧的基线就不会遮盖空洞。
`--ci` 因视觉差异和预览/捕获错误（缺少路由、浏览器）而非零退出
失败，选择器超时）;新的基线会报告为`new`，因此请先在本地`--update`运行。

示例应用还连接了Playwright原生的VRT路径（`examples/vite-musea`，运行于
`vp run test:vrt` / `vp run test:vrt:update`）。快照存在于`e2e/vrt/__snapshots__`，失败
工件在`e2e/vrt/test-results`，HTML报告在`playwright-report`;GitHub 操作
故障时上传，方便审核员检查基线、当前和差异图像。

## 生成艺术文件

使用生成器从现有组件创建初稿`.art.vue`：

```bash
vp exec musea-vrt generate src/components/Button.vue
```

生成的文件是一个起点。在阅读前请复习变体、标题、标签和道具内容
犯下了。

## 童话书输出

当你希望 Musea 艺术文件为故事书设置提供时，启用兼容 Storybook 的 CSF 生成：

```ts
musea({
  storybookCompat: true,
  storybookOutDir: ".storybook/stories",
});
```

## CLI状态

`vize musea`存在于 Rust CLI 中，但目前推荐的 Musea 工作流程仍然是 Vite
插件路径。在专用画廊工作流程稳定下来之前，把 Rust 子命令当作实验性操作。

Rust子指挥部可以搭建一个入门艺术项目：

```bash
vize musea new
```

## 相关套餐

- `@vizejs/vite-plugin-musea`
- `@vizejs/musea-mcp-server`
- `vize_musea`
