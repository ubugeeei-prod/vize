---
title: "vize:croquis/cf/unhandled-event"
---

# `vize:croquis/cf/unhandled-event`

子组件发出的事件没有父组件处理。

默认严重程度: info  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

## 共享项目文件

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## 错误示例

`Child.vue` 发出 `save`，直属包装组件却没有监听；组件事件不会自动穿过包装组件冒泡。

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

## 正确示例

`Wrapper.vue` 为直接子组件附加 `save` 监听器。空回调展示此规则认可的处理形式，并不是完整保存实现。

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[跨文件规则索引](../cross-file.md)
