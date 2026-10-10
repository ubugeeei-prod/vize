---
title: "Vapor 规则"
---

# Vapor 规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。

<span id="蒸汽统治"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [错误示例](#script-no-get-current-instance-bad) · [正确示例](#script-no-get-current-instance-good) | 禁止在 Vapor 模式使用 getCurrentInstance()（返回 null） |
| [`script/no-next-tick`](#script-no-next-tick) | [错误示例](#script-no-next-tick-bad) · [正确示例](#script-no-next-tick-good) | 禁止面向 Vapor 的组件使用 nextTick() |
| [`script/no-options-api`](#script-no-options-api) | [错误示例](#script-no-options-api-bad) · [正确示例](#script-no-options-api-good) | 禁止 Vapor 模式中的 Options API 模式 |
| [`vapor/no-inline-template`](#vapor-no-inline-template) | [错误示例](#vapor-no-inline-template-bad) · [正确示例](#vapor-no-inline-template-good) | 禁止已弃用的 inline-template 属性 |
| [`vapor/no-vue-lifecycle-events`](#vapor-no-vue-lifecycle-events) | [错误示例](#vapor-no-vue-lifecycle-events-bad) · [正确示例](#vapor-no-vue-lifecycle-events-good) | 禁止 @vue:xxx 元素生命周期事件（Vapor 不支持） |
| [`vapor/prefer-static-class`](#vapor-prefer-static-class) | [错误示例](#vapor-prefer-static-class-bad) · [正确示例](#vapor-prefer-static-class-good) | 字符串字面量优先使用静态 class，而不是动态 class 绑定 |
| [`vapor/require-vapor-attribute`](#vapor-require-vapor-attribute) | [错误示例](#vapor-require-vapor-attribute-bad) · [正确示例](#vapor-require-vapor-attribute-good) | 建议为 script setup 添加 vapor 属性 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `script/no-get-current-instance`

禁止在 Vapor 模式使用 getCurrentInstance()（返回 null）

[错误示例](#script-no-get-current-instance-bad) · [正确示例](#script-no-get-current-instance-good)

默认严重程度: `error`  
预设: `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: 面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-get-current-instance-bad"></span>

**错误示例**

带 Vapor 标记的 setup 导入并调用 `getCurrentInstance`，依赖了此规则针对 Vapor 组件禁止的实例 API。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**正确示例**

`inject("app-config")` 获取显式提供的配置，无需导入或调用 `getCurrentInstance`。

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [全部规则](all.md)

### `script/no-next-tick`

禁止面向 Vapor 的组件使用 nextTick()

[错误示例](#script-no-next-tick-bad) · [正确示例](#script-no-next-tick-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: 面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-next-tick-bad"></span>

**错误示例**

面向 Vapor 的组件导入并等待 `nextTick`，引入此迁移规则拒绝的 DOM 刷新调度依赖。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**正确示例**

通过 `useTemplateRef` 获取输入框，并在 `onMounted` 时聚焦。显式挂载边界替代示例中的 `nextTick` 依赖。

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [全部规则](all.md)

### `script/no-options-api`

禁止 Vapor 模式中的 Options API 模式

[错误示例](#script-no-options-api-bad) · [正确示例](#script-no-options-api-good)

默认严重程度: `error`  
预设: `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: 面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-options-api-bad"></span>

**错误示例**

默认导出对象声明了 Options API 的 `data()`，这是此规则禁止的组件选项形式。

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

<span id="script-no-options-api-good"></span>

**正确示例**

组件状态改为 Vapor `<script setup>` 中的 Composition API `ref`，移除 Options API 对象及其 `data` 选项。

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [全部规则](all.md)

### `vapor/no-inline-template`

禁止已弃用的 inline-template 属性

[错误示例](#vapor-no-inline-template-bad) · [正确示例](#vapor-no-inline-template-good)

默认严重程度: `error`  
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
        "vapor/no-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-inline-template-bad"></span>

**错误示例**

LegacyCard 为子标记使用 inline-template 属性。

```vue annotate="remove:2,3"
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

<span id="vapor-no-inline-template-good"></span>

**正确示例**

标记通过默认插槽传入，而非内联模板。

```vue annotate="add:2,3,4,5"
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) · [全部规则](all.md)

### `vapor/no-vue-lifecycle-events`

禁止 @vue:xxx 元素生命周期事件（Vapor 不支持）

[错误示例](#vapor-no-vue-lifecycle-events-bad) · [正确示例](#vapor-no-vue-lifecycle-events-good)

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
        "vapor/no-vue-lifecycle-events": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-vue-lifecycle-events-bad"></span>

**错误示例**

输入框使用了 @vue:mounted 模板生命周期事件。

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

<span id="vapor-no-vue-lifecycle-events-good"></span>

**正确示例**

onMounted 通过受支持的脚本生命周期钩子访问具名模板引用，并聚焦输入框。

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) · [全部规则](all.md)

### `vapor/prefer-static-class`

字符串字面量优先使用静态 class，而不是动态 class 绑定

[错误示例](#vapor-prefer-static-class-bad) · [正确示例](#vapor-prefer-static-class-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
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
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-prefer-static-class-bad"></span>

**错误示例**

class 绑定对常量字符串求值，但类名并不会改变。

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

<span id="vapor-prefer-static-class-good"></span>

**正确示例**

静态 class 属性表达相同的 panel 类名，无需绑定。

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) · [全部规则](all.md)

### `vapor/require-vapor-attribute`

建议为 script setup 添加 vapor 属性

[错误示例](#vapor-require-vapor-attribute-bad) · [正确示例](#vapor-require-vapor-attribute-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 尚未在 SFC lint 中实现  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

当前支持情况: `no-sfc-finding`

此规则是具有空回调的占位项。添加 vapor 会选择 Vapor 编译；当前 linter 不会因缺少该属性而报告此目录 ID。

**可配置的 ID（当前没有 SFC 诊断）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-require-vapor-attribute-bad"></span>

**错误示例**

script setup 块缺少 Vapor 编译属性。这是预期规范：当前为空的规则回调不会诊断它。

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

<span id="vapor-require-vapor-attribute-good"></span>

**正确示例**

添加 vapor 会选择 Vapor 编译。这展示预期修复，并不表示当前 linter 会报告此目录规则。

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

正确示例展示预期规范；当前 SFC 流程不会为任一示例生成这条规则的诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) · [全部规则](all.md)
