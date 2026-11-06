---
title: "vize:croquis/cf/reassignment-breaks-reactivity"
---

# `vize:croquis/cf/reassignment-breaks-reactivity`

重新赋值响应式绑定，将其替换为普通值。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| error | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

## 共享项目文件

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## 错误示例

子组件创建 prop ref 后用 `props.user` 覆盖变量，丢弃该 ref 联系。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
let user = toRef(props, "user");

user = props.user;
</script>
```

## 正确示例

将 `toRef` 保留在 `const` 绑定中，移除替换它的重新赋值。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
const user = toRef(props, "user");
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "croquis/cf/reassignment-breaks-reactivity": "warn" },
  },
});
```

```sh
vp run lint
```

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[跨文件规则索引](../cross-file.md)
