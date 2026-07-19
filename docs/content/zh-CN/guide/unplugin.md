---
title: 实验性捆绑器集成
---

<!-- Generated translation; source: guide/unplugin.md -->

# 实验性捆绑器积分

> **⚠️ 实验性：**`@vizejs/unplugin`和`@vizejs/rspack-plugin`仍然不稳定。
> `@vizejs/vite-plugin`至今仍是推荐且测试最完善的捆绑器集成。

Vize 提供了一个实验性的 [unplugin](https://unplugin.unjs.io/) 包，适用于 `rollup`、`webpack` 和 `esbuild`，以及一个专用的 `Rspack` 包：

- `@vizejs/unplugin` — `rollup` / `webpack` / `esbuild`
- `@vizejs/rspack-plugin` — 仅`Rspack`

Rspack 故意**不**通过共享的卸插件路径。
其加载链、`experiments.css`和HMR行为需要针对Rspack的具体处理。

## 安装

从[Vite+安装指南](https://viteplus.dev/guide/install)中安装一次`vp`，然后添加这些软件包：

```bash
vp install @vizejs/unplugin
```

关于Rspack：

```bash
vp install -D @vizejs/rspack-plugin @rspack/core
```

## 滚动

```javascript
// rollup.config.mjs
import vize from "@vizejs/unplugin/rollup";

export default {
  plugins: [vize()],
};
```

## webpack

```javascript
// webpack.config.mjs
import Vize from "@vizejs/unplugin/webpack";

export default {
  plugins: [Vize()],
};
```

## 埃斯建筑

```javascript
// build.mjs
import { build } from "esbuild";
import vize from "@vizejs/unplugin/esbuild";

await build({
  entryPoints: ["src/main.ts"],
  bundle: true,
  plugins: [vize()],
});
```

## Rspack

使用专用的`@vizejs/rspack-plugin`套餐代替`@vizejs/unplugin`：

```javascript
// rspack.config.mjs
import { VizePlugin } from "@vizejs/rspack-plugin";

export default {
  experiments: {
    css: true,
  },
  module: {
    rules: [
      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
      },
    ],
  },
  plugins: [new VizePlugin()],
};
```

完整 Rspack 配置表请参见包 README。

## 注意事项

- 如果你需要最完整、最经过测试的行为，Vite仍然是推荐的集成。
- CSS 模块和 Vite 以外的样式预处理器依赖于主机捆绑器的 CSS 流水线，且更可能发生变化。
- 如果你的捆绑器将Vue运行时内嵌而非外部化，确保该捆绑器配置了通常的Vue编译时功能标志。
- 将这些集成视为实验性，并在推广前与自身应用进行验证。
