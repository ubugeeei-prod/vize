---
title: Nuxt
description: 只需添加一个模块，即可在 Nuxt 项目中用 Vize 编译、检查和预览 Vue 组件。
---

<!-- Generated translation; source: integrations/nuxt.md -->

# Nuxt

`@vizejs/nuxt` 以单个模块的形式把 Vize 加入 Nuxt 项目。它用 Vize 的 Rust 编译器代替 Nuxt 默认的 Vue 编译器来编译 `.vue` 文件，生成适配 Nuxt 的 lint 配置，并可以提供 [Musea](../guide/musea.md) 组件画廊。自动导入、组件、中间件和 SSR 仍由 Nuxt 照常处理。

> [!WARNING]
> Vize 仍在积极开发中，尚不适合用于生产环境。采用之前，请在自己的项目中对比构建产物和诊断结果，并查看[支持状态](../stability.md)。

## 安装与配置

### 1. 安装模块

先按照 [Vite+ 安装指南](https://viteplus.dev/guide/install)安装一次 `vp`，然后添加模块：

```bash
vp install @vizejs/nuxt
```

### 2. 注册模块

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

`compiler: true` 是默认值，因此只写 `modules: ["@vizejs/nuxt"]` 就够了。本页的所有选项都写在 `vize` 键下。

### 3. 启动 Nuxt

照常启动开发服务器：

```bash
vp run dev
```

模块会把 `@vizejs/vite-plugin` 加入 Nuxt 的 Vite 配置，并保留 Nuxt 自身的转换。页面、布局、自动导入的组合式函数和组件都与之前一样工作。

### 4. 确认由 Vize 编译组件

开启编译器的调试日志，查看 Vize 处理了哪些文件：

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: { debug: true },
  },
});
```

Nuxt 按需编译组件，因此只有请求实际访问到的文件才会输出 `[vize] load: on-demand compiling ...`。确认配置无误后，请去掉 `debug`。

<span id="options"></span>

## 选项

所有选项都写在 `nuxt.config.ts` 的 `vize` 下。大多数项目只需要前四项。

| 选项                  | 类型                                    | 默认值                     | 作用                                                                                                   |
| --------------------- | --------------------------------------- | -------------------------- | ------------------------------------------------------------------------------------------------------ |
| `compiler`            | `boolean \| VizeNuxtCompilerOptions`    | `true`                     | 用 Vize 编译 Vue SFC。传入对象时，会把其中的选项转交给 `@vizejs/vite-plugin`。                         |
| `lint`                | `boolean \| VizeNuxtLintOptions`        | `true`                     | 生成适配 Nuxt 的 Oxlint 配置。参见 [Lint](#lint)。                                                     |
| `checker`             | `boolean \| VizeNuxtLintCheckerOptions` | `false`                    | 在开发服务器旁运行 lint，并在终端和浏览器浮层中显示结果。参见[开发时的 lint](#lint-while-developing)。 |
| `musea`               | `boolean \| MuseaOptions`               | `false`                    | 提供 Musea 画廊。参见 [Musea](#musea)。                                                                |
| `ui`                  | `boolean \| VizeNuxtUiOptions`          | `false`                    | 自动注册 `@vizejs/ui` 组件。参见[自动导入](../guide/auto-imports.md#nuxt)。                            |
| `composables`         | `boolean \| VizeNuxtComposablesOptions` | `false`                    | 自动导入 `@vizejs/composable` 的函数。参见[自动导入](../guide/auto-imports.md#nuxt)。                  |
| `bridge`              | `boolean \| VizeNuxtBridgeOptions`      | `true`                     | 对 Vize 编译的模块重新应用 Nuxt 的自动导入、组件解析、i18n 和异步数据键的转换。                        |
| `unocss`              | `boolean \| VizeNuxtUnoCssOptions`      | `true`                     | 让 UnoCSS 读取原始的 `.vue` 源码。`originalSource: false` 可关闭；`maxBytes` 限制每个文件读取的大小。  |
| `dev.stylesheetLinks` | `boolean`                               | `true`                     | 清理开发环境 SSR HTML 中重复或不安全的样式表链接。                                                     |
| `compatibility`       | `VizeNuxtCompatibilityOptions`          | 自动检测                   | 覆盖检测到的 Nuxt 和 Vue 主版本。参见 [Nuxt 2 与 Vue 2](#nuxt-2-and-vue-2)。                           |
| `nuxtMusea`           | `NuxtMuseaOptions`                      | `{ route: { path: "/" } }` | 描述 Musea 预览中可见的路由、运行时配置和 fetch 模拟数据。                                             |

### 编译器

`compiler` 可以传入任意 `@vizejs/vite-plugin` 选项。项目根目录、按需编译以及依赖中的 `.vue` 文件仍使用 Nuxt 的默认设置：

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compatibility: {
      // Usually inferred automatically.
      // Nuxt 2 defaults to Vue 2 compatibility mode; Nuxt 3/4 defaults to Vue 3.
      vueVersion: 3,
    },
    compiler: {
      // Any @vizejs/vite-plugin option can be passed here.
      configMode: "auto",
      customRenderer: false,
      debug: false,
      handleNodeModulesVue: true,
      ignorePatterns: ["node_modules/**", ".nuxt/**", ".output/**"],
      precompileBatchSize: 64,
      scanPatterns: [], // Nuxt defaults to on-demand compilation
      sourceMap: true,
      vapor: false,
    },
    bridge: {
      autoImports: true,
      components: true,
      i18n: true,
      stableInjectedKeys: true,
    },
    unocss: {
      originalSource: {
        maxBytes: 2 * 1024 * 1024,
      },
    },
    dev: {
      stylesheetLinks: true,
    },
    musea: false,
  },
});
```

[Vite 插件指南](../guide/vite-plugin.md#compiler-options)介绍了每个编译器选项。设置 `compiler: false` 时，Nuxt 保留默认编译器，你只使用 lint 或 Musea。

<span id="lint"></span>

### Lint

启用 `lint` 后，模块会在项目根目录写出 `.oxlint.vize.json`。它相当于 `@nuxt/eslint` 所生成配置的 Oxlint 版本：其中的规则块与你的 Nuxt 版本和目录结构相匹配。请把该文件加入 `.gitignore`。

如果项目还没有 Oxlint 或 Vite 配置，模块还会创建一个加载生成文件的 `oxlint.config.mts`。已有的配置不会被改写；请在其中自行加载 `.oxlint.vize.json`。

| 选项         | 默认值                | 作用                                                     |
| ------------ | --------------------- | -------------------------------------------------------- |
| `configFile` | `".oxlint.vize.json"` | 生成配置的写入位置。必须位于 lint 根目录或其上级目录中。 |
| `autoInit`   | `true`                | 在没有 Oxlint 或 Vite 配置时创建 `oxlint.config.mts`。   |
| `rootDir`    | Nuxt 根目录           | 生成的 glob 所基于的目录。                               |
| `devtools`   | `{ enabled: "lazy" }` | 在 Nuxt DevTools 的标签页中显示解析后的 lint 计划。      |

`lint` 也接受 `@nuxt/eslint-config` 的功能开关：`standalone`（默认 `true`）、`import`（默认 `true`）、`typescript`、`stylistic`、`tooling`、`formatters` 和 `nuxt`。

如果 lint 配置位于 Nuxt 根目录之外，请设置 `lint.autoInit: false`。[Oxlint 插件指南](../guide/oxlint.md)介绍了如何运行这些规则。

<span id="lint-while-developing"></span>

### 开发时的 lint

`checker` 会像 `@nuxt/eslint` 的 checker 一样，在开发服务器旁运行 Oxlint 和 Vize 的 lint 规则：

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    checker: true,
  },
});
```

| 选项          | 默认值                              | 作用                           |
| ------------- | ----------------------------------- | ------------------------------ |
| `lintOnStart` | `true`                              | 开发服务器启动时检查整个项目。 |
| `cache`       | `true`                              | 启动后只检查发生变化的文件。   |
| `include`     | `<srcDir>/**/*.{js,jsx,ts,tsx,vue}` | 要检查的文件。                 |
| `exclude`     | `**/node_modules/**` 和构建目录     | 跳过的文件和目录。             |
| `emitWarning` | `true`                              | 在终端和浏览器浮层中显示警告。 |
| `emitError`   | `true`                              | 在终端和浏览器浮层中显示错误。 |
| `fix`         | `false`                             | 报告前先应用安全的修复。       |
| `formatter`   | `"stylish"`                         | Oxlint 的输出格式。            |

<span id="musea"></span>

## Musea

用 `musea` 启用组件画廊：

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
    musea: {
      include: ["**/*.art.vue"],
      tokensPath: "assets/tokens.json",
      previewCss: ["assets/styles/main.css", "assets/styles/musea-preview.css"],
      previewSetup: "musea.preview.ts",
    },
    nuxtMusea: {
      route: { path: "/" }, // Musea UI route within __musea__
    },
  },
});
```

开发期间可以在 `/__musea__/` 打开画廊。[Musea 指南](../guide/musea.md)介绍了 art 文件、设计令牌和视觉回归测试。

### art 文件放在哪里

Nuxt 会扫描组件目录中的所有 `.vue` 文件，而 Musea 的 art 文件也以 `.vue` 结尾。请把 `*.art.vue` 放在这些目录之外，并告诉 Musea 它们的位置：

```txt
app/components/Tag.vue
stories/shared/Tag.art.vue
```

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    musea: {
      include: ["stories/**/*.art.vue"],
    },
  },
});
```

启用 Musea 后，模块还会把 `**/*.art.vue` 排除在 Nuxt 的组件扫描之外，因此留在组件旁边的 art 文件不会被注册为组件。

### 在预览中使用 Nuxt 的功能

组件常常依赖 `NuxtLink`、`useRoute`、`useNuxtApp`、`useRuntimeConfig`、数据获取类组合式函数或 Nuxt 内置组件。在独立的 Musea Vite 配置中，添加 `@vizejs/musea-nuxt`，并在 `previewSetup` 中安装它的模拟实现：

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { musea } from "@vizejs/vite-plugin-musea";
import { nuxtMusea } from "@vizejs/musea-nuxt";

export default defineConfig({
  plugins: [
    nuxtMusea({
      route: { path: "/preview" },
      runtimeConfig: { public: { apiBase: "/api" } },
      fetchMocks: {
        "/api/user": { id: 1, name: "Ada" },
      },
    }),
    musea({
      previewSetup: "musea.preview.ts",
    }),
  ],
});
```

```ts
// musea.preview.ts
import { installNuxtMuseaMocks } from "@vizejs/musea-nuxt";
import { createI18n } from "vue-i18n";
import type { MuseaPreviewSetup } from "@vizejs/vite-plugin-musea";

export default ((app) => {
  installNuxtMuseaMocks(app, {
    route: { path: "/preview" },
    runtimeConfig: { public: { apiBase: "/api" } },
  });

  const i18n = createI18n({
    locale: "ja",
    messages: {
      ja: {/* ... */},
      en: {/* ... */},
    },
  });
  app.use(i18n);
}) satisfies MuseaPreviewSetup;
```

在 Nuxt 应用内部，模块不会全局安装这些模拟实现，因为它们会替换 Nuxt 自己的 `#imports`。此时使用的是真正的 Nuxt 组合式函数。

<span id="nuxt-2-and-vue-2"></span>

## Nuxt 2 与 Vue 2

模块支持 Nuxt 2、3 和 4。Vize 的编译器输出 Vue 3 代码，因此在 Nuxt 2 中，模块会把 `.vue` 的编译交给现有的编译器（`@vitejs/plugin-vue2`、`vue-loader` 或 Nuxt 自带的编译器），同时继续提供 lint 和 Musea。更早的 Vue 项目也可以通过 `vueVersion: 0.11`、`vueVersion: 1` 或 `vueVersion: "legacy"` 获得同样的行为。

版本会被自动检测。如果某个封装让 Nuxt Kit 无法看到版本，请手动指定：

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compatibility: {
      nuxtVersion: 2,
      vueVersion: 2,
    },
  },
});
```

## 直接使用 Vite 插件

Nuxt 使用 Vite 构建，所以不使用模块也可以直接注册 Vite 插件。但这样会失去上文介绍的 Nuxt 专用桥接和 lint 配置生成：

```ts
// nuxt.config.ts
import vize from "@vizejs/vite-plugin";

export default defineNuxtConfig({
  vite: {
    plugins: [vize()],
  },
});
```

## 工作原理

1. **Vite 插件：** 模块注册 `@vizejs/vite-plugin`，由它编译 `.vue` 文件。
2. **与 Vue 插件兼容：** 插件提供与 `@vitejs/plugin-vue` 相同的 API，因此 Nuxt 内部对 Vue 插件的检查仍然能够通过。
3. **SSR：** Vize 也会编译服务端构建，并分别处理客户端和服务端的环境变量。
4. **Nuxt 的转换：** 自动导入、组合式函数和中间件由 Nuxt 自己的转换层处理，该层在 Vize 编译组件之后运行。

已发布的 `@vizejs/nuxt` 会把 `vize`、`@vizejs/vite-plugin`、`@vizejs/native` 以及其他 Vize 包固定为同一版本。

## 示例：Vue Fes Japan 2026

[Vue Fes Japan 2026](https://vuefes.jp/2026) 的网站在 Nuxt 4 中使用了该模块：

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: false, // compiler disabled (using Nuxt's default)
    musea: {
      include: ["**/*.art.vue"],
      inlineArt: false,
      tokensPath: "assets/tokens.json",
      previewCss: ["assets/styles/main.css", "assets/styles/musea-preview.css"],
      previewSetup: "musea.preview.ts",
    },
  },
});
```

它使用 Musea 开发和记录组件，生产构建仍使用 Nuxt 的默认编译器。

## 故障排查

- **在 pnpm 中使用 Pkl 配置：** `@vizejs/nuxt` 依赖 `vize`，后者附带默认的 `vize.pkl`。使用 pnpm 时，该文件可能不在 `node_modules/vize` 中。如果使用 Pkl 配置，请直接安装 `vize`：

  ```bash
  vp install vize
  ```

- **自定义的 `lint.configFile` 没有生效：** 该文件必须位于 `lint.rootDir` 或其上级目录中。放在 `.nuxt/` 中的配置无法应用 Oxlint 的项目级忽略规则。
- **开发环境 SSR HTML 中的资源链接：** `dev.stylesheetLinks` 会保留有效的、经过 URL 编码的 Nuxt 资源链接（例如 `%40fs/` 和编码后的 `assets/` 路径），并删除解码后包含空字节或路径穿越的链接。

其他问题请参阅[故障排查](../guide/troubleshooting.md)或[报告问题](../guide/testing.md)。
