---
title: "vize:croquis/cf/duplicate-id"
---

# `vize:croquis/cf/duplicate-id`

多个组件使用同一个元素 id。

| 默认严重程度 | 自动修复 | 选项 |
| --- | --- | --- |
| warning | 无；请检查相关文件并应用修复 | 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度 |

适用范围: 已分析的组件图，以及下方说明的受支持事实

## 共享项目文件

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./CheckoutForm.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## 错误示例

可达的配送与账单组件都渲染 `id="postal-code"`，使其标签共享不明确的文档目标。

`CheckoutForm.vue`

```vue
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

`ShippingAddress.vue`

```vue
<template>
  <label for="postal-code">Shipping postal code</label>
  <input id="postal-code" />
</template>
```

`BillingAddress.vue`

```vue
<template>
  <label for="postal-code">Billing postal code</label>
  <input id="postal-code" />
</template>
```

## 正确示例

每个组件调用 `useId()`，将自身的值同时绑定给标签和输入框，在不重复字面量 ID 的情况下保留关联。

`CheckoutForm.vue`

```vue
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

`ShippingAddress.vue`

```vue
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Shipping postal code</label>
  <input :id="postalCodeId" />
</template>
```

`BillingAddress.vue`

```vue
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Billing postal code</label>
  <input :id="postalCodeId" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "croquis/cf/duplicate-id": "warn" },
  },
});
```

```sh
vp run lint
```

## 相关页面

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[跨文件规则索引](../cross-file.md)
