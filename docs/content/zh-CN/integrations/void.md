---
title: Void
description: 在 Void 应用中使用 Vize，支持 @void/vue 页面、workerd 上的 SSR 以及 Void 生成的类型。
---

<!-- Generated translation; source: integrations/void.md -->

# Void

[Void](https://void.cloud) 基于 Vite 构建全栈应用，并在 Cloudflare 的 workerd 上运行。它的 Vue 适配器 `@void/vue` 在服务端渲染 `pages/`，并在浏览器中进行水合。在旁边加上 `vize()` 后，所有 `.vue` 文件都由 Vize 编译；路由、loader、岛屿和绑定仍由 Void 处理。

> [!WARNING]
> Vize 仍在积极开发中。采用前请在自己的应用上对比构建产物和诊断结果，并查看[支持状态](../stability.md)。

## 设置

### 1. 安装插件

```bash
npm install -D @vizejs/vite-plugin
```

### 2. 在 `vite.config.ts` 中添加 `vize()`

```ts
import { voidVue } from "@void/vue/plugin";
import { vize } from "@vizejs/vite-plugin";
import { defineConfig } from "vite";
import { voidPlugin } from "void";

export default defineConfig({
  plugins: [voidPlugin(), vize(), voidVue()],
});
```

`voidVue()` 自带 `@vitejs/plugin-vue`。无需移除它：`vize()` 会将其替换，使每个 SFC 只由一个编译器处理。如果设置了 `exclude`，plugin-vue 会保留，并编译 Vize 跳过的文件。

### 3. 照常运行 Void

```bash
npx vite dev
npx vite build
```

无需其他改动：

- **workerd 上的服务端渲染。** workerd 没有可导入的 `node_modules`，因此 Vue 和 `vue/server-renderer` 会被打包进 worker。
- **路由 CSS 与预加载。** 客户端清单以源码路径（`pages/index.vue`）为键，Void 用这个键把路由的样式表加入服务端渲染的 HTML。
- **`vite dev` 中的 HMR。** 编辑模板时保留组件状态，loader 的 props 也保持不变。
- **页面 props。** 使用从 `.server.ts` loader 导入的 `Props` 的 `defineProps<Props>()`，编译结果与 plugin-vue 相同。

## 类型检查

Void 会生成包含路由、环境变量和绑定类型的 `.void/tsconfig.json`，你的 `tsconfig.json` 继承它。先运行一次 `void prepare`（或启动开发服务器）生成这些文件，然后检查：

```bash
npx void prepare
vize check
```

`vize check` 会读取生成的 `files` 和 `paths`，因此在 `.vue` 文件中，下列内容的类型与 `vue-tsc` 一致：

- 通过 `InferProps<typeof loader>` 得到的 loader props；
- `void/client` 的 `fetch`（路由路径、方法和响应体）；
- `void/env` 的 `env`，以及 `void/kv` 等绑定模块。

## Lint 与格式化

`vize lint` 和 `vize fmt` 从项目根目录查找文件，并跳过 `.gitignore` 中列出的内容。Void 的 `.gitignore` 包含 `dist`、`.void`、`.wrangler` 和 `.void-wrangler.jsonc`，因此生成的文件不会被检查或重新格式化。

## 编辑器

[语言服务器](../guide/language-server.md)使用同一个 `tsconfig.json`。打开项目前请先运行 `void prepare`，确保生成的类型存在。

## 测试方式

每当 Vite 插件或原生编译器发生变化，CI 都会构建一个固定版本的 Void 1.1.12 + `@void/vue` 应用（`tools/support/compat/void/fixtures/void-vue`），并检查：

- 生产构建，以及 `vite preview` 在 workerd 上的 SSR（包括路由 CSS 和 KV 绑定）；
- 开发时的 SSR、水合，以及保留状态的 HMR；
- `vize check` 与 `vue-tsc` 报告相同的错误；
- `vize lint` 与 `vize fmt --check`。

Void 上的 Nuxt（在 `nuxt.config.ts` 中使用 `voidPlugin()`）尚未被此测试覆盖，这种情况请使用 [Nuxt 集成](./nuxt.md)。
