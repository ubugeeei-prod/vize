---
title: "HTML 规则"
---

# HTML 规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。

<span id="html规则"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`html/deprecated-attr`](#html-deprecated-attr) | [错误示例](#html-deprecated-attr-bad) · [正确示例](#html-deprecated-attr-good) | 禁止已弃用的 HTML 属性 |
| [`html/deprecated-element`](#html-deprecated-element) | [错误示例](#html-deprecated-element-bad) · [正确示例](#html-deprecated-element-good) | 禁止已弃用的 HTML 元素 |
| [`html/id-duplication`](#html-id-duplication) | [错误示例](#html-id-duplication-bad) · [正确示例](#html-id-duplication-good) | 禁止重复的元素 ID |
| [`html/no-consecutive-br`](#html-no-consecutive-br) | [错误示例](#html-no-consecutive-br-bad) · [正确示例](#html-no-consecutive-br-good) | 禁止连续的 &lt;br&gt; 元素 |
| [`html/no-dupe-style-properties`](#html-no-dupe-style-properties) | [错误示例](#html-no-dupe-style-properties-bad) · [正确示例](#html-no-dupe-style-properties-good) | 禁止内联 style 属性中重复的属性声明 |
| [`html/no-duplicate-class`](#html-no-duplicate-class) | [错误示例](#html-no-duplicate-class-bad) · [正确示例](#html-no-duplicate-class-good) | 禁止静态 class 属性中重复的类名 |
| [`html/no-duplicate-dt`](#html-no-duplicate-dt) | [错误示例](#html-no-duplicate-dt-bad) · [正确示例](#html-no-duplicate-dt-good) | 禁止 &lt;dl&gt; 中重复的 &lt;dt&gt; 名称 |
| [`html/no-empty-palpable-content`](#html-no-empty-palpable-content) | [错误示例](#html-no-empty-palpable-content-bad) · [正确示例](#html-no-empty-palpable-content-good) | 禁止预期具有可见内容的空元素 |
| [`html/require-datetime`](#html-require-datetime) | [错误示例](#html-require-datetime-bad) · [正确示例](#html-require-datetime-good) | 要求 &lt;time&gt; 元素具有 datetime 属性 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `html/deprecated-attr`

禁止已弃用的 HTML 属性

[错误示例](#html-deprecated-attr-bad) · [正确示例](#html-deprecated-attr-good)

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
        "html/deprecated-attr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-attr-bad"></span>

**错误示例**

段落使用了已弃用的呈现属性 `align`。

```vue annotate="remove:1,2,3"
<template>
<p align="center">Notice</p>
</template>
```

<span id="html-deprecated-attr-good"></span>

**正确示例**

类名和 `text-align: center` 声明通过 CSS 表达对齐方式。

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) · [全部规则](all.md)

### `html/deprecated-element`

禁止已弃用的 HTML 元素

[错误示例](#html-deprecated-element-bad) · [正确示例](#html-deprecated-element-good)

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
        "html/deprecated-element": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-element-bad"></span>

**错误示例**

`center` 使用了已弃用的 HTML 呈现元素。

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

<span id="html-deprecated-element-good"></span>

**正确示例**

section 和样式类替代弃用元素，同时保留内容。

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) · [全部规则](all.md)

### `html/id-duplication`

禁止重复的元素 ID

[错误示例](#html-id-duplication-bad) · [正确示例](#html-id-duplication-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "html/id-duplication": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-id-duplication-bad"></span>

**错误示例**

输入框和帮助段落都声明 `id="email"`，使标签目标不明确。

```vue annotate="remove:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

<span id="html-id-duplication-good"></span>

**正确示例**

输入框保留 `email`；帮助段落使用 `email-help`，aria-describedby 引用该独立 ID。

```vue annotate="add:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) · [全部规则](all.md)

### `html/no-consecutive-br`

禁止连续的 &lt;br&gt; 元素

[错误示例](#html-no-consecutive-br-bad) · [正确示例](#html-no-consecutive-br-good)

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
        "html/no-consecutive-br": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-consecutive-br-bad"></span>

**错误示例**

两个连续换行元素在同一段落内为内容块制造间距。

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

<span id="html-no-consecutive-br-good"></span>

**正确示例**

使用独立段落表达两个内容块，无需重复换行元素。

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) · [全部规则](all.md)

### `html/no-dupe-style-properties`

禁止内联 style 属性中重复的属性声明

[错误示例](#html-no-dupe-style-properties-bad) · [正确示例](#html-no-dupe-style-properties-good)

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
        "html/no-dupe-style-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-dupe-style-properties-bad"></span>

**错误示例**

每个静态 style 都重复了一个属性；`margin` 和 `MARGIN` 也算作同一属性。

```vue annotate="remove:2,3"
<template>
<div style="color: red; color: blue">text</div>
<div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

<span id="html-no-dupe-style-properties-good"></span>

**正确示例**

静态 style 使用不同的 color 和 background 属性。动态样式绑定不在此静态属性检查范围内。

```vue annotate="add:2,3"
<template>
<div style="color: red; background: blue">text</div>
<div :style="{ color: a, color: b }">text</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) · [全部规则](all.md)

### `html/no-duplicate-class`

禁止静态 class 属性中重复的类名

[错误示例](#html-no-duplicate-class-bad) · [正确示例](#html-no-duplicate-class-good)

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
        "html/no-duplicate-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-class-bad"></span>

**错误示例**

静态类名列表重复了 `btn`。

```vue annotate="remove:2"
<template>
<div class="btn btn primary">click</div>
</template>
```

<span id="html-no-duplicate-class-good"></span>

**正确示例**

类名列表保留一个 `btn` 及不同的 `primary`。

```vue annotate="add:2"
<template>
<div class="btn primary">click</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) · [全部规则](all.md)

### `html/no-duplicate-dt`

禁止 &lt;dl&gt; 中重复的 &lt;dt&gt; 名称

[错误示例](#html-no-duplicate-dt-bad) · [正确示例](#html-no-duplicate-dt-good)

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
        "html/no-duplicate-dt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-dt-bad"></span>

**错误示例**

同一定义列表为两项描述重复了 `API` 术语。

```vue annotate="remove:5"
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

<span id="html-no-duplicate-dt-good"></span>

**正确示例**

一个 API 术语后跟两项描述，避免重复术语。

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) · [全部规则](all.md)

### `html/no-empty-palpable-content`

禁止预期具有可见内容的空元素

[错误示例](#html-no-empty-palpable-content-bad) · [正确示例](#html-no-empty-palpable-content-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "html/no-empty-palpable-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-empty-palpable-content-bad"></span>

**错误示例**

段落、列表项和表格单元格都没有可感知的内容。

```vue annotate="remove:2,3,4"
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

<span id="html-no-empty-palpable-content-good"></span>

**正确示例**

文本填充段落，插值提供列表项内容，aria-label 则显式命名原本为空的单元格。

```vue annotate="add:2,3,4"
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) · [全部规则](all.md)

### `html/require-datetime`

要求 &lt;time&gt; 元素具有 datetime 属性

[错误示例](#html-require-datetime-bad) · [正确示例](#html-require-datetime-good)

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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-require-datetime-bad"></span>

**错误示例**

time 元素包含人类可读的日期，却没有机器可读的 datetime 值。

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

<span id="html-require-datetime-good"></span>

**正确示例**

`datetime="2026-05-13"` 提供对应的机器可读日期。

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [全部规则](all.md)
