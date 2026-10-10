---
title: "无障碍规则"
---

# 无障碍规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。

<span id="额外的无障碍规则"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`a11y/alt-text`](#a11y-alt-text) | [错误示例](#a11y-alt-text-bad) · [正确示例](#a11y-alt-text-good) | 要求媒体元素提供替代文本 |
| [`a11y/anchor-has-content`](#a11y-anchor-has-content) | [错误示例](#a11y-anchor-has-content-bad) · [正确示例](#a11y-anchor-has-content-good) | 要求锚点元素具有可访问的内容 |
| [`a11y/anchor-is-valid`](#a11y-anchor-is-valid) | [错误示例](#a11y-anchor-is-valid-bad) · [正确示例](#a11y-anchor-is-valid-good) | 要求锚点元素具有有效的 href |
| [`a11y/aria-props`](#a11y-aria-props) | [错误示例](#a11y-aria-props-bad) · [正确示例](#a11y-aria-props-good) | 禁止无效的 ARIA 属性 |
| [`a11y/aria-role`](#a11y-aria-role) | [错误示例](#a11y-aria-role-bad) · [正确示例](#a11y-aria-role-good) | 具有 ARIA 角色的元素必须使用有效的非抽象 ARIA 角色 |
| [`a11y/aria-unsupported-elements`](#a11y-aria-unsupported-elements) | [错误示例](#a11y-aria-unsupported-elements-bad) · [正确示例](#a11y-aria-unsupported-elements-good) | 禁止在不支持 ARIA 的元素上使用 ARIA 属性 |
| [`a11y/click-events-have-key-events`](#a11y-click-events-have-key-events) | [错误示例](#a11y-click-events-have-key-events-bad) · [正确示例](#a11y-click-events-have-key-events-good) | 要求点击事件配有键盘事件处理器 |
| [`a11y/form-control-has-label`](#a11y-form-control-has-label) | [错误示例](#a11y-form-control-has-label-bad) · [正确示例](#a11y-form-control-has-label-good) | 要求表单控件具有相关联的标签 |
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [错误示例](#a11y-heading-has-content-bad) · [正确示例](#a11y-heading-has-content-good) | 要求标题元素具有可访问的内容 |
| [`a11y/heading-levels`](#a11y-heading-levels) | [错误示例](#a11y-heading-levels-bad) · [正确示例](#a11y-heading-levels-good) | 禁止跳过标题层级 |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [错误示例](#a11y-iframe-has-title-bad) · [正确示例](#a11y-iframe-has-title-good) | 要求 iframe 元素具有 title 属性 |
| [`a11y/img-alt`](#a11y-img-alt) | [错误示例](#a11y-img-alt-bad) · [正确示例](#a11y-img-alt-good) | 要求图片提供 alt 属性，以支持无障碍访问 |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [错误示例](#a11y-interactive-supports-focus-bad) · [正确示例](#a11y-interactive-supports-focus-good) | 要求具有交互角色的元素可获得焦点 |
| [`a11y/label-has-for`](#a11y-label-has-for) | [错误示例](#a11y-label-has-for-bad) · [正确示例](#a11y-label-has-for-good) | 要求标签具有相关联的表单控件 |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [错误示例](#a11y-landmark-roles-bad) · [正确示例](#a11y-landmark-roles-good) | 验证地标角色的位置及唯一性 |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [错误示例](#a11y-media-has-caption-bad) · [正确示例](#a11y-media-has-caption-good) | 要求媒体元素具有字幕 |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [错误示例](#a11y-mouse-events-have-key-events-bad) · [正确示例](#a11y-mouse-events-have-key-events-good) | 要求鼠标事件配有 focus/blur 事件 |
| [`a11y/no-access-key`](#a11y-no-access-key) | [错误示例](#a11y-no-access-key-bad) · [正确示例](#a11y-no-access-key-good) | 禁止使用 accesskey 属性 |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [错误示例](#a11y-no-aria-hidden-on-focusable-bad) · [正确示例](#a11y-no-aria-hidden-on-focusable-good) | 禁止在可获得焦点的元素上使用 aria-hidden="true" |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [错误示例](#a11y-no-autofocus-bad) · [正确示例](#a11y-no-autofocus-good) | 禁止使用 autofocus 属性 |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [错误示例](#a11y-no-distracting-elements-bad) · [正确示例](#a11y-no-distracting-elements-good) | 禁止 &lt;marquee&gt; 和 &lt;blink&gt; 等分散注意力的元素 |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [错误示例](#a11y-no-i-for-icon-bad) · [正确示例](#a11y-no-i-for-icon-good) | 禁止使用 &lt;i&gt; 元素表示图标 |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [错误示例](#a11y-no-redundant-roles-bad) · [正确示例](#a11y-no-redundant-roles-good) | 禁止多余的 ARIA 角色 |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [错误示例](#a11y-no-refer-to-non-existent-id-bad) · [正确示例](#a11y-no-refer-to-non-existent-id-good) | 禁止引用不存在的 ID |
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [错误示例](#a11y-no-role-presentation-on-focusable-bad) · [正确示例](#a11y-no-role-presentation-on-focusable-good) | 禁止在可获得焦点的元素上使用 role="presentation" 或 role="none" |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [错误示例](#a11y-no-static-element-interactions-bad) · [正确示例](#a11y-no-static-element-interactions-good) | 禁止在静态元素上设置事件处理器 |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [错误示例](#a11y-placeholder-label-option-bad) · [正确示例](#a11y-placeholder-label-option-good) | 要求 select 的占位选项具有 disabled 或 hidden |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [错误示例](#a11y-role-has-required-aria-props-bad) · [正确示例](#a11y-role-has-required-aria-props-good) | 要求 ARIA 角色具有必需属性 |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [错误示例](#a11y-tabindex-no-positive-bad) · [正确示例](#a11y-tabindex-no-positive-good) | 禁止正数 tabindex 值 |
| [`a11y/use-list`](#a11y-use-list) | [错误示例](#a11y-use-list-bad) · [正确示例](#a11y-use-list-good) | 建议为类似项目符号的文本使用列表元素 |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [错误示例](#vue-use-unique-element-ids-bad) · [正确示例](#vue-use-unique-element-ids-good) | 要求使用 useId() 生成唯一元素 ID，而不是静态字面量 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `a11y/alt-text`

要求媒体元素提供替代文本

[错误示例](#a11y-alt-text-bad) · [正确示例](#a11y-alt-text-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-alt-text-bad"></span>

**错误示例**

图片提交控件只提供图片 URL，没有描述操作的 `alt` 文本。

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

<span id="a11y-alt-text-good"></span>

**正确示例**

`alt="Submit search"` 为图片控件提供描述提交搜索操作的无障碍名称。

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [全部规则](all.md)

### `a11y/anchor-has-content`

要求锚点元素具有可访问的内容

[错误示例](#a11y-anchor-has-content-bad) · [正确示例](#a11y-anchor-has-content-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-has-content-bad"></span>

**错误示例**

`/settings` 链接没有文本或其他命名内容，因此其目标没有可访问的描述。

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

<span id="a11y-anchor-has-content-good"></span>

**正确示例**

可见的 `Settings` 文本为同一目标链接提供内容。

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) · [全部规则](all.md)

### `a11y/anchor-is-valid`

要求锚点元素具有有效的 href

[错误示例](#a11y-anchor-is-valid-bad) · [正确示例](#a11y-anchor-is-valid-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/anchor-is-valid": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-is-valid-bad"></span>

**错误示例**

第一个锚点使用 `#` 执行操作；第二个使用 JavaScript URL。两者都没有提供普通导航目标。

```vue annotate="remove:2,3"
<template>
  <a href="#" @click="openPanel">Open panel</a>
  <a href="JaVaScRiPt:void(0)">Run action</a>
</template>
```

<span id="a11y-anchor-is-valid-good"></span>

**正确示例**

原生按钮执行 `openPanel`，剩余锚点则具有真实的 `/docs/javascript-urls` 目标。

```vue annotate="add:2,3"
<template>
  <button type="button" @click="openPanel">Open panel</button>
  <a href="/docs/javascript-urls">JavaScript URL guide</a>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) · [全部规则](all.md)

### `a11y/aria-props`

禁止无效的 ARIA 属性

[错误示例](#a11y-aria-props-bad) · [正确示例](#a11y-aria-props-good)

默认严重程度: `error`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/aria-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-props-bad"></span>

**错误示例**

`aria-lable` 拼写错误，不是受支持的 ARIA 属性。

```vue annotate="remove:2"
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

<span id="a11y-aria-props-good"></span>

**正确示例**

受支持的 `aria-label` 属性提供按钮名称。

```vue annotate="add:2"
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [全部规则](all.md)

### `a11y/aria-role`

具有 ARIA 角色的元素必须使用有效的非抽象 ARIA 角色

[错误示例](#a11y-aria-role-bad) · [正确示例](#a11y-aria-role-good)

默认严重程度: `error`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/aria-role": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-role-bad"></span>

**错误示例**

`datepicker` 不是此 section 上可识别的 ARIA 角色。

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

<span id="a11y-aria-role-good"></span>

**正确示例**

section 使用可识别的 `dialog` 角色，以及描述日期选择的标签。

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [全部规则](all.md)

### `a11y/aria-unsupported-elements`

禁止在不支持 ARIA 的元素上使用 ARIA 属性

[错误示例](#a11y-aria-unsupported-elements-bad) · [正确示例](#a11y-aria-unsupported-elements-good)

默认严重程度: `error`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/aria-unsupported-elements": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-unsupported-elements-bad"></span>

**错误示例**

元数据元素带有 `aria-hidden`，但 `meta` 不支持 ARIA 属性。

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

<span id="a11y-aria-unsupported-elements-good"></span>

**正确示例**

移除 ARIA 属性，保留字符集声明。

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [全部规则](all.md)

### `a11y/click-events-have-key-events`

要求点击事件配有键盘事件处理器

[错误示例](#a11y-click-events-have-key-events-bad) · [正确示例](#a11y-click-events-have-key-events-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

检查没有交互角色的非交互式元素。原生按钮和具有交互式 ARIA 角色的元素不在此规则诊断范围内。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/click-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-click-events-have-key-events-bad"></span>

**错误示例**

非交互式 `div` 有点击处理器，却没有键盘事件处理。

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

<span id="a11y-click-events-have-key-events-good"></span>

**正确示例**

原生 `button` 为同一 `activate` 处理器提供键盘激活方式。

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) · [全部规则](all.md)

### `a11y/form-control-has-label`

要求表单控件具有相关联的标签

[错误示例](#a11y-form-control-has-label-bad) · [正确示例](#a11y-form-control-has-label-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/form-control-has-label": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-form-control-has-label-bad"></span>

**错误示例**

搜索输入框没有说明用户应输入什么的标签。

```vue annotate="remove:2"
<template>
  <input type="search" />
</template>
```

<span id="a11y-form-control-has-label-good"></span>

**正确示例**

将输入框包在 label 中，使可见的 `Search` 文本与控件关联。

```vue annotate="add:2,3,4,5"
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [全部规则](all.md)

### `a11y/heading-has-content`

要求标题元素具有可访问的内容

[错误示例](#a11y-heading-has-content-bad) · [正确示例](#a11y-heading-has-content-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-has-content-bad"></span>

**错误示例**

`h2` 提供了标题层级，却没有标题内容。

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

<span id="a11y-heading-has-content-good"></span>

**正确示例**

`Billing settings` 为原有二级标题提供内容。

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) · [全部规则](all.md)

### `a11y/heading-levels`

禁止跳过标题层级

[错误示例](#a11y-heading-levels-bad) · [正确示例](#a11y-heading-levels-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/heading-levels": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-levels-bad"></span>

**错误示例**

标题序列从 `h1` 直接跳到 `h3`，跳过二级。

```vue annotate="remove:3"
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

<span id="a11y-heading-levels-good"></span>

**正确示例**

将账单标题改为 `h2`，保持连续的标题层级。

```vue annotate="add:3"
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) · [全部规则](all.md)

### `a11y/iframe-has-title`

要求 iframe 元素具有 title 属性

[错误示例](#a11y-iframe-has-title-bad) · [正确示例](#a11y-iframe-has-title-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/iframe-has-title": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-iframe-has-title-bad"></span>

**错误示例**

结账框架有来源 URL，却没有描述嵌入内容的 `title`。

```vue annotate="remove:2"
<template>
  <iframe src="/checkout"></iframe>
</template>
```

<span id="a11y-iframe-has-title-good"></span>

**正确示例**

`title="Checkout preview"` 为该框架的内容命名。

```vue annotate="add:2"
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) · [全部规则](all.md)

### `a11y/img-alt`

要求图片提供 alt 属性，以支持无障碍访问

[错误示例](#a11y-img-alt-bad) · [正确示例](#a11y-img-alt-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-img-alt-bad"></span>

**错误示例**

头像图片缺少 `alt` 属性。

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

<span id="a11y-img-alt-good"></span>

**正确示例**

`alt="User avatar"` 为头像提供文本替代。

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) · [全部规则](all.md)

### `a11y/interactive-supports-focus`

要求具有交互角色的元素可获得焦点

[错误示例](#a11y-interactive-supports-focus-bad) · [正确示例](#a11y-interactive-supports-focus-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-interactive-supports-focus-bad"></span>

**错误示例**

为 `span` 设置按钮角色和点击处理器，并不会让它能够通过键盘获得焦点。

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

<span id="a11y-interactive-supports-focus-good"></span>

**正确示例**

原生按钮可以获得焦点，并保留同一 `open` 操作。

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [全部规则](all.md)

### `a11y/label-has-for`

要求标签具有相关联的表单控件

[错误示例](#a11y-label-has-for-bad) · [正确示例](#a11y-label-has-for-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/label-has-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-label-has-for-bad"></span>

**错误示例**

独立的 label 既没有通过 `for` 关联输入框，也没有包住输入框。

```vue annotate="remove:2"
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

<span id="a11y-label-has-for-good"></span>

**正确示例**

`for="email"` 与输入框 ID 一致，显式关联两个元素。

```vue annotate="add:2"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) · [全部规则](all.md)

### `a11y/landmark-roles`

验证地标角色的位置及唯一性

[错误示例](#a11y-landmark-roles-bad) · [正确示例](#a11y-landmark-roles-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/landmark-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-landmark-roles-bad"></span>

**错误示例**

同一模板中的两个 `main` 元素声明了重复的主地标。

```vue annotate="remove:3"
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

<span id="a11y-landmark-roles-good"></span>

**正确示例**

仪表盘保留为主地标；设置区域改为具名的导航地标。

```vue annotate="add:3"
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) · [全部规则](all.md)

### `a11y/media-has-caption`

要求媒体元素具有字幕

[错误示例](#a11y-media-has-caption-bad) · [正确示例](#a11y-media-has-caption-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/media-has-caption": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-media-has-caption-bad"></span>

**错误示例**

视频有播放控件，却没有字幕轨道。

```vue annotate="remove:2"
<template>
  <video src="/demo.mp4" controls />
</template>
```

<span id="a11y-media-has-caption-good"></span>

**正确示例**

带有 `kind="captions"` 的 `track` 为同一视频提供英文字幕。

```vue annotate="add:2,3,4"
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) · [全部规则](all.md)

### `a11y/mouse-events-have-key-events`

要求鼠标事件配有 focus/blur 事件

[错误示例](#a11y-mouse-events-have-key-events-bad) · [正确示例](#a11y-mouse-events-have-key-events-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/mouse-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-mouse-events-have-key-events-bad"></span>

**错误示例**

预览可见性只通过鼠标进入和离开处理器改变。

```vue annotate="remove:2"
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

<span id="a11y-mouse-events-have-key-events-good"></span>

**正确示例**

相同的预览操作也在 focus 和 blur 时执行，并且按钮可以获得键盘焦点。

```vue annotate="add:2,3,4,5,6,7,8,9,10"
<template>
  <button
    type="button"
    @focus="showPreview"
    @blur="hidePreview"
    @mouseenter="showPreview"
    @mouseleave="hidePreview"
  >
    Preview
  </button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) · [全部规则](all.md)

### `a11y/no-access-key`

禁止使用 accesskey 属性

[错误示例](#a11y-no-access-key-bad) · [正确示例](#a11y-no-access-key-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-access-key-bad"></span>

**错误示例**

`accesskey="s"` 快捷键可能与浏览器或辅助技术快捷键冲突。

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

<span id="a11y-no-access-key-good"></span>

**正确示例**

移除 `accesskey`，普通的 Save 按钮仍可使用。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [全部规则](all.md)

### `a11y/no-aria-hidden-on-focusable`

禁止在可获得焦点的元素上使用 aria-hidden="true"

[错误示例](#a11y-no-aria-hidden-on-focusable-bad) · [正确示例](#a11y-no-aria-hidden-on-focusable-good)

默认严重程度: `error`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

**错误示例**

可获得焦点的 Close 按钮使用 `aria-hidden="true"`，从无障碍树中隐藏。

```vue annotate="remove:2"
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

**正确示例**

按钮继续对辅助技术可见，并获得 `Close` 标签，不再被隐藏。

```vue annotate="add:2"
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [全部规则](all.md)

### `a11y/no-autofocus`

禁止使用 autofocus 属性

[错误示例](#a11y-no-autofocus-bad) · [正确示例](#a11y-no-autofocus-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-autofocus-bad"></span>

**错误示例**

输入框在出现时请求自动获得焦点。

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

<span id="a11y-no-autofocus-good"></span>

**正确示例**

移除 `autofocus`，避免该自动焦点请求，同时保留查询输入框。

```vue annotate="add:2"
<template>
  <input name="query" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) · [全部规则](all.md)

### `a11y/no-distracting-elements`

禁止 &lt;marquee&gt; 和 &lt;blink&gt; 等分散注意力的元素

[错误示例](#a11y-no-distracting-elements-bad) · [正确示例](#a11y-no-distracting-elements-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-distracting-elements": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-distracting-elements-bad"></span>

**错误示例**

`marquee` 元素引入了自动移动的文本。

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

<span id="a11y-no-distracting-elements-good"></span>

**正确示例**

段落显示同样的优惠信息，不使用分散注意力的 marquee 元素。

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) · [全部规则](all.md)

### `a11y/no-i-for-icon`

禁止使用 &lt;i&gt; 元素表示图标

[错误示例](#a11y-no-i-for-icon-bad) · [正确示例](#a11y-no-i-for-icon-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-i-for-icon": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-i-for-icon-bad"></span>

**错误示例**

图标通过 `i` 渲染，但其文本语义无法描述仅有图标的操作。

```vue annotate="remove:3"
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

<span id="a11y-no-i-for-icon-good"></span>

**正确示例**

装饰性 span 隐藏图标字形，独立的 `Delete item` 文本为按钮操作命名。

```vue annotate="add:3,4"
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) · [全部规则](all.md)

### `a11y/no-redundant-roles`

禁止多余的 ARIA 角色

[错误示例](#a11y-no-redundant-roles-bad) · [正确示例](#a11y-no-redundant-roles-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-redundant-roles-bad"></span>

**错误示例**

原生按钮已具有按钮角色，因此 `role="button"` 重复了其隐式语义。

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

<span id="a11y-no-redundant-roles-good"></span>

**正确示例**

移除重复角色，保留 HTML 提供的按钮语义。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) · [全部规则](all.md)

### `a11y/no-refer-to-non-existent-id`

禁止引用不存在的 ID

[错误示例](#a11y-no-refer-to-non-existent-id-bad) · [正确示例](#a11y-no-refer-to-non-existent-id-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-refer-to-non-existent-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

**错误示例**

`aria-labelledby` 指向 `save-label`，但没有元素声明该 ID。

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

<span id="a11y-no-refer-to-non-existent-id-good"></span>

**正确示例**

添加匹配的 span，解决引用问题并提供按钮标签。

```vue annotate="add:2"
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) · [全部规则](all.md)

### `a11y/no-role-presentation-on-focusable`

禁止在可获得焦点的元素上使用 role="presentation" 或 role="none"

[错误示例](#a11y-no-role-presentation-on-focusable-bad) · [正确示例](#a11y-no-role-presentation-on-focusable-good)

默认严重程度: `error`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-role-presentation-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

**错误示例**

可获得焦点的账单链接请求 role=presentation，与其交互式链接角色冲突；浏览器必须忽略该呈现角色请求。

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

<span id="a11y-no-role-presentation-on-focusable-good"></span>

**正确示例**

移除冲突的呈现角色请求，使用原生链接角色和账单目标。

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [全部规则](all.md)

### `a11y/no-static-element-interactions`

禁止在静态元素上设置事件处理器

[错误示例](#a11y-no-static-element-interactions-bad) · [正确示例](#a11y-no-static-element-interactions-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/no-static-element-interactions": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-static-element-interactions-bad"></span>

**错误示例**

静态 section 接收 Enter 键操作，却没有交互角色。

```vue annotate="remove:2"
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

<span id="a11y-no-static-element-interactions-good"></span>

**正确示例**

原生按钮以适当的交互元素承载相同操作。

```vue annotate="add:2"
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) · [全部规则](all.md)

### `a11y/placeholder-label-option`

要求 select 的占位选项具有 disabled 或 hidden

[错误示例](#a11y-placeholder-label-option-bad) · [正确示例](#a11y-placeholder-label-option-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-placeholder-label-option-bad"></span>

**错误示例**

空值提示仍可被选择，仿佛它是一个国家值。

```vue annotate="remove:3"
<template>
  <select v-model="country">
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

<span id="a11y-placeholder-label-option-good"></span>

**正确示例**

添加 `disabled`，将提示与可选择的 Japan 选项区分开。

```vue annotate="add:3"
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) · [全部规则](all.md)

### `a11y/role-has-required-aria-props`

要求 ARIA 角色具有必需属性

[错误示例](#a11y-role-has-required-aria-props-bad) · [正确示例](#a11y-role-has-required-aria-props-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-role-has-required-aria-props-bad"></span>

**错误示例**

checkbox 角色省略了表达复选框状态的 `aria-checked`。

```vue annotate="remove:2"
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

<span id="a11y-role-has-required-aria-props-good"></span>

**正确示例**

`aria-checked="false"` 提供 checkbox 角色所需的状态。

```vue annotate="add:2"
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) · [全部规则](all.md)

### `a11y/tabindex-no-positive`

禁止正数 tabindex 值

[错误示例](#a11y-tabindex-no-positive-bad) · [正确示例](#a11y-tabindex-no-positive-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/tabindex-no-positive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-tabindex-no-positive-bad"></span>

**错误示例**

正数 tabindex 3 建立了位于普通控件之前的自定义焦点顺序。

```vue annotate="remove:2"
<template>
  <button tabindex="3">Save</button>
</template>
```

<span id="a11y-tabindex-no-positive-good"></span>

**正确示例**

按钮使用原生焦点顺序，不设置正数 tabindex。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) · [全部规则](all.md)

### `a11y/use-list`

建议为类似项目符号的文本使用列表元素

[错误示例](#a11y-use-list-bad) · [正确示例](#a11y-use-list-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/use-list": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-use-list-bad"></span>

**错误示例**

任务项使用单独的段落和手写短横线标记，没有使用列表元素。

```vue annotate="remove:2,3"
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

<span id="a11y-use-list-good"></span>

**正确示例**

无序列表及列表项以列表语义表达相同任务。

```vue annotate="add:2,3,4,5"
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) · [全部规则](all.md)

### `vue/use-unique-element-ids`

要求使用 useId() 生成唯一元素 ID，而不是静态字面量

[错误示例](#vue-use-unique-element-ids-bad) · [正确示例](#vue-use-unique-element-ids-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-unique-element-ids": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-unique-element-ids-bad"></span>

**错误示例**

组件的每个实例都会复用字面量 ID `email`，多个实例同时渲染时，标签可能指向错误的输入框。

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**正确示例**

`useId()` 生成实例的 `emailId`；将同一值绑定到标签的 `for` 和输入框的 `id`。

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [全部规则](all.md)
