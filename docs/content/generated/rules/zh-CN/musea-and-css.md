---
title: "Musea 与 CSS 规则"
---

# Musea 与 CSS 规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。

<span id="博物馆与css规则"></span>
<span id="额外的css规则"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`css/no-display-none`](#css-no-display-none) | [错误示例](#css-no-display-none-bad) · [正确示例](#css-no-display-none-good) | 建议使用 v-show 代替 display: none |
| [`css/no-hardcoded-values`](#css-no-hardcoded-values) | [错误示例](#css-no-hardcoded-values-bad) · [正确示例](#css-no-hardcoded-values-good) | 建议使用 CSS 变量代替硬编码值 |
| [`css/no-id-selectors`](#css-no-id-selectors) | [错误示例](#css-no-id-selectors-bad) · [正确示例](#css-no-id-selectors-good) | 不建议在 CSS 中使用 ID 选择器 |
| [`css/no-important`](#css-no-important) | [错误示例](#css-no-important-bad) · [正确示例](#css-no-important-good) | 不建议在 CSS 中使用 !important |
| [`css/no-utility-classes`](#css-no-utility-classes) | [错误示例](#css-no-utility-classes-bad) · [正确示例](#css-no-utility-classes-good) | 警告在组件样式中实现工具类 |
| [`css/no-v-bind-performance`](#css-no-v-bind-performance) | [错误示例](#css-no-v-bind-performance-bad) · [正确示例](#css-no-v-bind-performance-good) | 警告 CSS v-bind() 的性能开销 |
| [`css/prefer-logical-properties`](#css-prefer-logical-properties) | [错误示例](#css-prefer-logical-properties-bad) · [正确示例](#css-prefer-logical-properties-good) | 建议使用 CSS 逻辑属性，以更好地支持国际化 |
| [`css/prefer-nested-selectors`](#css-prefer-nested-selectors) | [错误示例](#css-prefer-nested-selectors-bad) · [正确示例](#css-prefer-nested-selectors-good) | 建议为后代选择器使用 CSS 嵌套 |
| [`css/prefer-slotted`](#css-prefer-slotted) | [错误示例](#css-prefer-slotted-bad) · [正确示例](#css-prefer-slotted-good) | 建议使用 ::v-slotted() 为插槽内容设置样式 |
| [`css/require-font-display`](#css-require-font-display) | [错误示例](#css-require-font-display-bad) · [正确示例](#css-require-font-display-good) | 要求 @font-face 规则具有 font-display |
| [`musea/no-empty-variant`](#musea-no-empty-variant) | [错误示例](#musea-no-empty-variant-bad) · [正确示例](#musea-no-empty-variant-good) | 禁止空的 &lt;variant&gt; 块 |
| [`musea/prefer-design-tokens`](#musea-prefer-design-tokens) | [错误示例](#musea-prefer-design-tokens-bad) · [正确示例](#musea-prefer-design-tokens-good) | 优先使用设计令牌 CSS 变量，而不是硬编码的原始值 |
| [`musea/require-component`](#musea-require-component) | [错误示例](#musea-require-component-bad) · [正确示例](#musea-require-component-good) | 要求 &lt;art&gt; 块具有 component 属性 |
| [`musea/require-title`](#musea-require-title) | [错误示例](#musea-require-title-bad) · [正确示例](#musea-require-title-good) | 要求 &lt;art&gt; 块具有 title 属性 |
| [`musea/unique-variant-names`](#musea-unique-variant-names) | [错误示例](#musea-unique-variant-names-bad) · [正确示例](#musea-unique-variant-names-good) | 要求 variant 名称唯一 |
| [`musea/valid-variant`](#musea-valid-variant) | [错误示例](#musea-valid-variant-bad) · [正确示例](#musea-valid-variant-good) | 要求 &lt;variant&gt; 块具有 name 属性 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `css/no-display-none`

建议使用 v-show 代替 display: none

[错误示例](#css-no-display-none-bad) · [正确示例](#css-no-display-none-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-display-none": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-display-none-bad"></span>

**错误示例**

`.message` 声明通过 CSS 隐藏当前段落，没有使用模板可见性条件。

```vue annotate="remove:2,4,5,6,7,8,9"
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

<span id="css-no-display-none-good"></span>

**正确示例**

`v-show="isSaved"` 在当前段落上显式表达可见性条件，并移除 `display: none`。

```vue annotate="add:2"
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) · [全部规则](all.md)

### `css/no-hardcoded-values`

建议使用 CSS 变量代替硬编码值

[错误示例](#css-no-hardcoded-values-bad) · [正确示例](#css-no-hardcoded-values-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-hardcoded-values": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-hardcoded-values-bad"></span>

**错误示例**

按钮将间距数字和十六进制颜色直接写入声明。

```vue annotate="remove:3,4"
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

<span id="css-no-hardcoded-values-good"></span>

**正确示例**

声明引用具名的间距和颜色自定义属性，使这些值可作为设计令牌维护。

```vue annotate="add:3,4"
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) · [全部规则](all.md)

### `css/no-id-selectors`

不建议在 CSS 中使用 ID 选择器

[错误示例](#css-no-id-selectors-bad) · [正确示例](#css-no-id-selectors-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-id-selectors-bad"></span>

**错误示例**

`#submit` 将样式规则绑定到 ID 选择器。

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

<span id="css-no-id-selectors-good"></span>

**正确示例**

`.submit` 类提供可复用的样式入口，不使用 ID 选择器。

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) · [全部规则](all.md)

### `css/no-important`

不建议在 CSS 中使用 !important

[错误示例](#css-no-important-bad) · [正确示例](#css-no-important-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-important": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-important-bad"></span>

**错误示例**

颜色声明通过 `!important` 覆盖普通层叠优先级。

```vue annotate="remove:3"
<style scoped>
.button {
  color: red !important;
}
</style>
```

<span id="css-no-important-good"></span>

**正确示例**

颜色来自自定义属性，没有使用 important 声明。

```vue annotate="add:3"
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) · [全部规则](all.md)

### `css/no-utility-classes`

警告在组件样式中实现工具类

[错误示例](#css-no-utility-classes-bad) · [正确示例](#css-no-utility-classes-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-utility-classes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-utility-classes-bad"></span>

**错误示例**

编写的选择器采用 `.flex`、`.mt-4` 和 `.text-center` 等工具类形式的名称。

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

<span id="css-no-utility-classes-good"></span>

**正确示例**

组件专用的 `.my-component` 选择器以一个语义名称组织组件样式。

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) · [全部规则](all.md)

### `css/no-v-bind-performance`

警告 CSS v-bind() 的性能开销

[错误示例](#css-no-v-bind-performance-bad) · [正确示例](#css-no-v-bind-performance-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-v-bind-performance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-v-bind-performance-bad"></span>

**错误示例**

样式表通过 SFC CSS 的 `v-bind()` 机制读取变化中的 `offset`。

```vue annotate="remove:1,2,3,4,5"
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

<span id="css-no-v-bind-performance-good"></span>

**正确示例**

元素通过自身的样式绑定直接接收变化中的 transform。

```vue annotate="add:1,2,3"
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) · [全部规则](all.md)

### `css/prefer-logical-properties`

建议使用 CSS 逻辑属性，以更好地支持国际化

[错误示例](#css-prefer-logical-properties-bad) · [正确示例](#css-prefer-logical-properties-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-logical-properties-bad"></span>

**错误示例**

`margin-left` 无论书写方向如何，都将外边距固定在物理侧。

```vue annotate="remove:3"
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

<span id="css-prefer-logical-properties-good"></span>

**正确示例**

`margin-inline-start` 改为跟随行内方向的起始侧。

```vue annotate="add:3"
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [全部规则](all.md)

### `css/prefer-nested-selectors`

建议为后代选择器使用 CSS 嵌套

[错误示例](#css-prefer-nested-selectors-bad) · [正确示例](#css-prefer-nested-selectors-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-nested-selectors-bad"></span>

**错误示例**

`.card .title` 后代选择器在平铺规则中重复了父选择器。

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

<span id="css-prefer-nested-selectors-good"></span>

**正确示例**

`.title` 规则嵌套在 `.card` 内，将父子样式关系放在一起。

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) · [全部规则](all.md)

### `css/prefer-slotted`

建议使用 ::v-slotted() 为插槽内容设置样式

[错误示例](#css-prefer-slotted-bad) · [正确示例](#css-prefer-slotted-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-slotted-bad"></span>

**错误示例**

scoped 样式表选择的是 `slot` 出口，而不是通过插槽传入的元素。

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

<span id="css-prefer-slotted-good"></span>

**正确示例**

`:slotted(.label)` 通过作用域插槽选择器选中传入的 label 元素。

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [全部规则](all.md)

### `css/require-font-display`

要求 @font-face 规则具有 font-display

[错误示例](#css-require-font-display-bad) · [正确示例](#css-require-font-display-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/require-font-display": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-require-font-display-bad"></span>

**错误示例**

font-face 声明定义了字体来源，却省略了 font-display 策略。

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

<span id="css-require-font-display-good"></span>

**正确示例**

`font-display: swap` 显式选择先显示后备字体、再切换到目标字体的策略。

```vue annotate="add:5"
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) · [全部规则](all.md)

### `musea/no-empty-variant`

禁止空的 &lt;variant&gt; 块

[错误示例](#musea-no-empty-variant-bad) · [正确示例](#musea-no-empty-variant-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/no-empty-variant": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-no-empty-variant-bad"></span>

**错误示例**

名为 primary 的 variant 为空，没有提供预览内容。

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-no-empty-variant-good"></span>

**正确示例**

variant 渲染带有 Save 内容的 primary Button。

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) · [全部规则](all.md)

### `musea/prefer-design-tokens`

优先使用设计令牌 CSS 变量，而不是硬编码的原始值

[错误示例](#musea-prefer-design-tokens-bad) · [正确示例](#musea-prefer-design-tokens-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 参见[类型化选项和默认值](/rules/options.md)。

需要 .art.vue 文件及下方所示的令牌清单。不会从任意颜色推断设计令牌。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-prefer-design-tokens-bad"></span>

**错误示例**

art 示例使用蓝色字面量，而不是配置的 primary 设计令牌。

`Button.art.vue`

```vue annotate="remove:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

<span id="musea-prefer-design-tokens-good"></span>

**正确示例**

样式引用此示例配置的令牌 --color-primary。

`Button.art.vue`

```vue annotate="add:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) · [全部规则](all.md)

### `musea/require-component`

要求 &lt;art&gt; 块具有 component 属性

[错误示例](#musea-require-component-bad) · [正确示例](#musea-require-component-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-component": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-component-bad"></span>

**错误示例**

art 块提供了标题，却未标识要预览的组件。

```vue annotate="remove:1"
<art title="Button">
  <variant name="primary" />
</art>
```

<span id="musea-require-component-good"></span>

**正确示例**

defineArt 将 ./Button.vue 指定为 art 块的组件。

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) · [全部规则](all.md)

### `musea/require-title`

要求 &lt;art&gt; 块具有 title 属性

[错误示例](#musea-require-title-bad) · [正确示例](#musea-require-title-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-title": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-title-bad"></span>

**错误示例**

art 块标识了 Button.vue，却没有提供标题。

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-require-title-good"></span>

**正确示例**

defineArt 选项为 art 块提供 Button 标题。

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) · [全部规则](all.md)

### `musea/unique-variant-names`

要求 variant 名称唯一

[错误示例](#musea-unique-variant-names-bad) · [正确示例](#musea-unique-variant-names-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/unique-variant-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-unique-variant-names-bad"></span>

**错误示例**

同一 art 块中的两个 variant 都使用 primary 名称。

```vue annotate="remove:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

<span id="musea-unique-variant-names-good"></span>

**正确示例**

两个 variant 分别使用不同的 primary 和 secondary 名称。

```vue annotate="add:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) · [全部规则](all.md)

### `musea/valid-variant`

要求 &lt;variant&gt; 块具有 name 属性

[错误示例](#musea-valid-variant-bad) · [正确示例](#musea-valid-variant-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/valid-variant": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-valid-variant-bad"></span>

**错误示例**

variant 省略了标识预览所需的名称。

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

<span id="musea-valid-variant-good"></span>

**正确示例**

primary 名称标识了该 variant。

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) · [全部规则](all.md)
