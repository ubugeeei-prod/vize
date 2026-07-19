---
title: 评论注释
---

<!-- Generated translation; source: guide/comment-annotations.md -->

# 评论注释

Vize 提供基于注释的注释来控制 linting、诊断和代码生成行为。根据使用地点不同，有两种注释系统：

- **`<!-- @vize:xxx -->`**— `<template>` HTML注释（Patina linter 指令）
- **`// @vize forget: reason`**— JS注释`<script>`（跨文件分析抑制）

所有`@vize:`模板指令都被剥离了构建输出——它们从生产代码中从未出现。

## 模板指令（`@vize:`）

在`<template>`内部用作HTML注释。这些控制了铜浆（内置的黏液）行为。

### `@vize:expected`

下一条线路会有诊断。如果没有诊断结果，则为无效行动。和`@ts-expect-error`类似。

```vue
<template>
  <ul>
    <!-- @vize:expected -->
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
```

### `@vize:ignore-start` / `@vize:ignore-end`

压制该区域内的所有诊断。

```vue
<template>
  <!-- @vize:ignore-start -->
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
  <!-- @vize:ignore-end -->
</template>
```

### `@vize:level(warn|error|off)`

在下一行中覆盖诊断的严重性。

```vue
<template>
  <!-- @vize:level(warn) -->
  <img src="/photo.png" />

  <!-- @vize:level(off) -->
  <li v-for="item in items">{{ item }}</li>
</template>
```

| 价值    | 效果       |
| ------- | ---------- |
| `warn`  | 降级为警告 |
| `error` | 升级到错误 |
| `off`   | 完全压制   |

### `@vize:todo`

发出待办事项警告。

```vue
<template>
  <!-- @vize:todo add loading state -->
  <div>{{ data }}</div>
</template>
```

### `@vize:fixme`

发出FIXME错误。

```vue
<template>
  <!-- @vize:fixme broken on mobile -->
  <div class="layout">...</div>
</template>
```

### `@vize:deprecated`

发出退役警告。

```vue
<template>
  <!-- @vize:deprecated use NewComponent instead -->
  <OldComponent />
</template>
```

### `@vize:docs`

文档评论。没有绒毛效应。

```vue
<template>
  <!-- @vize:docs Primary action button for form submission -->
  <button type="submit">Submit</button>
</template>
```

### `@vize:dev-only`

标记一个节点，在生产版本中要剥离，保持开发中。

```vue
<template>
  <!-- @vize:dev-only -->
  <div class="debug-panel">{{ internalState }}</div>
</template>
```

### 摘要

| 指令                     | 效果               | 严重程度 |
| ------------------------ | ------------------ | -------- |
| `@vize:expected`         | 下一行有诊断       | —        |
| `@vize:ignore-start/end` | 抑制区域内所有诊断 | —        |
| `@vize:level(...)`       | 覆盖下一行严重度   | —        |
| `@vize:todo <msg>`       | 发出 TODO          | 警告     |
| `@vize:fixme <msg>`      | 发射修正我         | 错误     |
| `@vize:deprecated <msg>` | 发布弃用通知       | 警告     |
| `@vize:docs <text>`      | 文档（无绒毛效应） | —        |
| `@vize:dev-only`         | 制作中的连环画     | —        |

## 剧本压制（`@vize forget`）

作为JS评论，在`<script>`内使用。下一行抑制跨文件分析警告（Croquis）。

### 语法

```vue
<script setup>
// @vize forget: <reason>
<suppressed line>
</script>
```

需要一个**理由**——你必须解释为什么需要抑制。

### 示例

```vue
<script setup>
import { inject } from "vue";

// @vize forget: intentionally destructuring for one-time read
const { count } = inject("state");
</script>
```

如果没有注释，Vize 会警告，拆解反应`inject()`返回值会破坏反应追踪。

### 规则

| 规则       | 描述                                                  |
| ---------- | ----------------------------------------------------- |
| 需要理由   | `// @vize forget`没有理由就是错误                     |
| 需要结肠   | 必须使用`// @vize forget: <reason>`（冒号加于理由前） |
| 仅限下一行 | 适用于下一个非注释、非空行                            |
| 无孤儿     | 文件末尾没有代码的抑制是错误                          |

### 多重抑制

每个`@vize forget`独立应用于下一个代码行：

```vue
<script setup>
import { inject } from "vue";

// @vize forget: one-time read for display name
const { name } = inject("user");

// @vize forget: static config value
const { theme } = inject("config");
</script>
```

### 跳过评论

抑制针对下一个**code**行，跳过注释和空行：

```vue
<script setup>
// @vize forget: read-only access
// This comment is skipped
const { count } = inject("state");
</script>
```

### 常见原因

| 原因                         | 何时使用             |
| ---------------------------- | -------------------- |
| `intentionally non-reactive` | 价值不必是反应性的   |
| `read-only access`           | 只读，不跟踪变更     |
| `legacy code`                | 已知问题，稍后会重构 |
| `third-party integration`    | 外部库要求           |

### 无效示例

```ts
// @vize forget
const { count } = inject("state");
// ^ Error: requires a reason

// @vize forget because I said so
const { count } = inject("state");
// ^ Error: requires a colon before the reason

// @vize forget:
const { count } = inject("state");
// ^ Error: reason cannot be empty
```
