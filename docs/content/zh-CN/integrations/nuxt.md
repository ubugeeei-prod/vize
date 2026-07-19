---
title: 努克斯特
---

<!-- Generated translation; source: integrations/nuxt.md -->

# Nuxt 整合

> **⚠️ 正在开发中：**Vize正在积极开发中，尚未准备好投入生产使用。在Nuxt项目中采用前请彻底测试。

Vize通过`@vizejs/nuxt`模块提供一流的Nuxt集成。这用 Vize 的 Rust 原生编译器取代了 Nuxt 默认的 Vue 编译器，从而在 Nuxt 项目中实现了同样的速度提升。

## 开始

### 1.安装模块

从[Vite+安装指南](https://viteplus.dev/guide/install)中安装一次`vp`，然后添加模块：

```bash
vp install @vizejs/nuxt
```

如果你想用`pkl`配置配合pnpm，可能需要安装`vize`包本身。
`@vizejs/nuxt`安装`vize`，默认配置`vize.pkl`，但使用 pnpm 时 pnpm 的 `vize.pkl` 位置可能会不同。

```bash
vp install vize
```

### 2.注册Nuxt模块

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

### 3.开始Nuxt

像往常一样启动开发服务器：

```bash
vp run dev
```

该模块将`@vizejs/vite-plugin`注入Nuxt的Vite配置，并保留Nuxt专属的变换
因此自动导入、组件、中间件和SSR行为都能继续运行
Nuxt。
在开发过程中，服务器响应清理会保留有效的 URL 编码的 Nuxt 资产链接，如
作为`%40fs/`和编码的`assets/`路径，同时丢弃解码的空字节或遍历路径。

## 模块选项

`@vizejs/nuxt`保留了简单的`compiler: true | false`开关，但模块选项也会暴露
Vize编译器和Nuxt兼容桥，用于需要更严格控制的项目：

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

| 选项                  | 类型                                 | 默认                       | 描述                                                                                                                                                       |
| --------------------- | ------------------------------------ | -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `compatibility`       | `VizeNuxtCompatibilityOptions`       | 自动检测                   | 覆盖检测到Nuxt/Vue主版本的异常包装。Nuxt 2 默认支持 Vue 2 主机-编译器兼容性;Nuxt 3/4默认使用 Vue 3。Vue 0.11/1/2 均使用主机编译器模式。                    |
| `compiler`            | `boolean \| VizeNuxtCompilerOptions` | `true`                     | 使 Vize 成为 Vue SFC 编译器。传递对象会将选项转发给`@vizejs/vite-plugin`，同时保持 Nuxt 默认的 `root`、`devUrlBase`、按需 `scanPatterns` 和依赖 SFC 处理。 |
| `bridge`              | `boolean \| VizeNuxtBridgeOptions`   | `true`                     | 控制 Nuxt 变换桥，用于自动导入、组件导入、i18n 辅助工具以及 Vize 虚拟模块上的稳定异步数据键。                                                              |
| `unocss`              | `boolean \| VizeNuxtUnoCssOptions`   | `true`                     | 控制 Vize 虚拟模块的 UnoCSS 桥接器。`originalSource: false` 会禁用读取源 SFC;`maxBytes`限制了内存的使用。                                                  |
| `dev.stylesheetLinks` | `boolean`                            | `true`                     | 支持仅限开发者的SSR HTML样式表链接清理，用于Vize生成的Nuxt资源URL。                                                                                        |
| `musea`               | `boolean \| MuseaOptions`            | `false`                    | 选择加入 Musea 画廊集成。使用`true` 来设置 Musea 默认值，或传递对象以配置包含模式、令牌、预览 CSS 和路由。                                                 |
| `nuxtMusea`           | `NuxtMuseaOptions`                   | `{ route: { path: "/" } }` | 文档说明了 Musea 预览助手使用的 Nuxt 模拟形状。Nuxt 模块不会全局安装模拟层，因为这样做会影响 Nuxt 自身的`#imports`。                                       |

## 高级设置

### Nuxt 2 与 Legacy Vue

Nuxt 2 项目使用 Vue 2 编译器的输出。Vize 的原生 SFC 编译器针对 Vue 3，所以 Nuxt
模块在检测到 Nuxt 2 时会自动避免更换主机编译器。对于Nuxt 2号桥
或其他基于Vite的Vue 2设置，Vite插件接收`vueVersion: 2`，保持
`@vitejs/plugin-vue2`、`vue-loader`，或者Nuxt自家负责`.vue`文件的编译器。

同样的主机编译器模式也可以通过`vueVersion: 0.11`在较老的Vue项目中提供，
`vueVersion: 1`，或者说`vueVersion: "legacy"`。

如果你的项目以一种方式包裹 Nuxt，导致 Nuxt Kit 中隐藏了该版本，请设置兼容性
明确覆盖：

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

### 直接使用 Vite 插件

或者，你也可以直接使用Vite插件。由于Nuxt在底层使用Vite，这可行，但缺少一些针对Nuxt的优化：

```ts
// nuxt.config.ts
import vize from "@vizejs/vite-plugin";

export default defineNuxtConfig({
  vite: {
    plugins: [vize()],
  },
});
```

## 博物馆整合

Nuxt 模块还支持 Musea（组件画廊）集成：

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

配置完成后，博物馆展厅在开发期间可`/__musea__/`开放。

### 艺术文件放置

Nuxt 组件自动发现扫描已配置组件目录中的`.vue`文件。因为
Musea的艺术文件结尾也`.vue`，`*.art.vue`文件放在Nuxt目录之外
项目并指向该地点的博物馆：

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

当 Musea 通过 `@vizejs/nuxt` 启用时，模块也会从 Nuxt 中排除 `**/*.art.vue`
组件扫描器，因此共址的遗留文件无法到达Nuxt的webpack或Vite组件流水线。

### Nuxt 预览设置

Nuxt 项目通常使用需要在 Musea 预览环境中具备的功能
（`NuxtLink`、`useRoute`、`useNuxtApp`、`useRuntimeConfig`、数据组合和内置Nuxt
组件）。在独立的 Musea Vite 配置中使用 `@vizejs/musea-nuxt` 并安装预览版
`previewSetup`的模拟图层：

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
      ja: {
        /* ... */
      },
      en: {
        /* ... */
      },
    },
  });
  app.use(i18n);
}) satisfies MuseaPreviewSetup;
```

## 工作原理

安装Nuxt模块时：

1.**Vite 插件注入**— 该模块将`@vizejs/vite-plugin`注册为 Vite 插件，拦截`.vue`文件编译。2.**兼容性提示**— 该插件会暴露一个`@vitejs/plugin-vue`兼容API，因此Nuxt的内部检查（用于探测Vue插件）能够正常工作。3.**SSR 支持**— Vize 的`vize_atelier_ssr`负责服务器端编译。该插件隔离客户端和服务器环境变量，以防止交叉污染。4.**Nuxt 功能被保留**— 自动导入、组合、中间件及其他 Nuxt 功能通过 Nuxt 自身的变换层工作，该层运行于 Vize 编译之后。

## 现实世界的例子

[Vue Fes Japan 2026](https://vuefes.jp/2026)会议网站使用Vize配合Nuxt 4：

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

该配置使用 Musea 进行组件开发和文档，同时保留 Nuxt 默认编译器用于生产构建。

## 注释

- Vize处于积极开发阶段——在投入生产环境前彻底测试
- 通过`vize_atelier_ssr`支持SSR编译
- Nuxt 专属功能（自动导入、可组合、中间件）通过 Nuxt 自身的变换层工作
- Nuxt 模块支持 Nuxt 2、Nuxt 3 和 Nuxt 4。Nuxt 2 采用主机-编译器兼容性模式，因为 Vize 的原生 SFC 编译器针对的是 Vue 3 输出。
