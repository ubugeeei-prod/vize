---
title: Oxlint 插件
---

<!-- Generated translation; source: guide/oxlint.md -->

# Oxlint 插件

`oxlint-plugin-vize` 允许 Oxlint 通过 Oxlint 的 JS 插件系统执行 Vize Patina 的诊断。
当你想同时使用Oxlint的Rust原生JS和TS规则，以及Vize的Vue感知规则时，可以使用它
一次完成诊断。

关于Oxlint之外的原生lint和类型检查流水线，请参见
[静态分析](./static-analysis.md)。

> [!重要]
> 该软件包可在 npm 上使用，但集成阶段还处于早期阶段。对于人类可读终端
> 输出，偏好`oxlint-vize -f stylish`，而原版SFC的射程保真度也在不断提升。

## 安装

从[Vite+安装指南](https://viteplus.dev/guide/install)中安装一次`vp`，然后添加这些软件包：

```bash
vp install -D oxlint oxlint-plugin-vize
```

`oxlint-plugin-vize`通过可选依赖解决匹配的 Vize 本地绑定，因此
大多数用户不需要单独安装`@vizejs/native`。

## 基本用法

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "helpLevel": "short"
    }
  },
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn",
    "no-console": "warn"
  }
}
```

如果你使用 JS 或 TS Oxlint 配置，该包还导出预设规则映射：

```js
import { configs } from "oxlint-plugin-vize";

export default {
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      helpLevel: "short",
      preset: "opinionated",
      typeAware: true,
    },
  },
  rules: configs.opinionatedWithTypeAware,
};
```

可用的预设导出包括：

- `configs.recommended`
- `configs.essential`
- `configs.opinionated`
- `configs.nuxt`
- `configs.all`
- `configs.recommendedWithTypeAware`
- `configs.ecosystemWithTypeAware`
- `configs.opinionatedWithTypeAware`

## 推荐指挥

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

`oxlint-vize`是一个薄薄的包裹`oxlint`，能够平滑无脚本的 `.vue` 边缘情况
而上游 JS 插件覆盖率持续提升。

## 设定

设置会通过`settings.vize`传递：

```json
{
  "settings": {
    "vize": {
      "locale": "ja",
      "preset": "general-recommended",
      "helpLevel": "short",
      "typeAware": true
    }
  }
}
```

- `locale`控制诊断语言。
- `preset`接受`"general-recommended"`、`"essential"`、`"ecosystem"`、`"incremental"`、`"opinionated"`或`"nuxt"`。
- `preset`默认`"general-recommended"`。
- `incremental`只运行你明确配置的规则。
- `helpLevel`接受`"full"`、`"short"`或`"none"`。
- `typeAware: true` 在共享 Patina 传递时启用 Corsa 支持的 `vize/type/*` 规则。
- `corsaPath`选择Corsa或`tsgo`可执行文件进行类型识别线条处理。
- `showHelp` 和 `settings.patina` 仍然被接受以实现向后兼容。

## 当前限制

- 原始`oxlint`仍可能遗漏部分`.vue`文件，无需 `<script>` 或 `<script setup>`。用途
  `oxlint-vize`你的项目是否包含仅模板的SFC。
- Oxlint JS 插件仍然将范围锚定到提取后的脚本程序，因此模板和样式
  诊断尚未在每个格式化器中保留原始SFC范围。
- `stylish` 目前是混合 Oxlint + Vize 输出的最佳人类可读格式化器。JSON 和
  其他机器可读格式应视为原始模板/样式的尽力而为
  职位。
- 类型感知规则导出是实验性的。用`*WithTypeAware`配置并设置
  `settings.vize.typeAware: true`你想让共享的完整文件通行证能热切运行这些规则。

## 地方发展

```bash
nix develop
vp install --frozen-lockfile
vp run --filter './npm/native' build
vp run --filter './npm/oxint' build
```
