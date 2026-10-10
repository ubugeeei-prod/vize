---
title: "生态系统规则"
---

# 生态系统规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。


| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](#ecosystem-nuxt-prefer-nuxt-link) | [错误示例](#ecosystem-nuxt-prefer-nuxt-link-bad) · [正确示例](#ecosystem-nuxt-prefer-nuxt-link-good) | 应用内部链接优先使用 NuxtLink |
| [`ecosystem/pinia-prefer-store-to-refs`](#ecosystem-pinia-prefer-store-to-refs) | [错误示例](#ecosystem-pinia-prefer-store-to-refs-bad) · [正确示例](#ecosystem-pinia-prefer-store-to-refs-good) | 解构 Pinia store 时优先使用 storeToRefs() |
| [`ecosystem/router-link-require-to`](#ecosystem-router-link-require-to) | [错误示例](#ecosystem-router-link-require-to-bad) · [正确示例](#ecosystem-router-link-require-to-good) | 要求 RouterLink 和 NuxtLink 组件具有 `to` 目标 |
| [`ecosystem/void-link-require-href`](#ecosystem-void-link-require-href) | [错误示例](#ecosystem-void-link-require-href-bad) · [正确示例](#ecosystem-void-link-require-href-good) | 要求 Void Vue Link 组件具有 `href` |
| [`ecosystem/void-link-valid-method`](#ecosystem-void-link-valid-method) | [错误示例](#ecosystem-void-link-valid-method-bad) · [正确示例](#ecosystem-void-link-valid-method-good) | 验证 Void Vue Link 的静态 method prop |
| [`ecosystem/vue-i18n-no-missing-key`](#ecosystem-vue-i18n-no-missing-key) | [错误示例](#ecosystem-vue-i18n-no-missing-key-bad) · [正确示例](#ecosystem-vue-i18n-no-missing-key-good) | 报告本地 SFC 消息中不存在的静态 vue-i18n 键 |
| [`ecosystem/vue-router-prefer-named-link`](#ecosystem-vue-router-prefer-named-link) | [错误示例](#ecosystem-vue-router-prefer-named-link-bad) · [正确示例](#ecosystem-vue-router-prefer-named-link-good) | RouterLink 优先使用具名路由对象，而不是静态路径字符串 |
| [`ecosystem/vue-router-prefer-named-push`](#ecosystem-vue-router-prefer-named-push) | [错误示例](#ecosystem-vue-router-prefer-named-push-bad) · [正确示例](#ecosystem-vue-router-prefer-named-push-good) | Vue Router 编程式导航优先使用具名路由对象 |
| [`ecosystem/vue-test-utils-no-html-snapshot`](#ecosystem-vue-test-utils-no-html-snapshot) | [错误示例](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [正确示例](#ecosystem-vue-test-utils-no-html-snapshot-good) | 避免在 Vue Test Utils 测试中对 wrapper.html() 生成快照 |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [错误示例](#nuxt-no-nuxt-config-test-key-bad) · [正确示例](#nuxt-no-nuxt-config-test-key-good) | 禁止在 Nuxt 配置中设置 `test` 键 |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [错误示例](#nuxt-no-page-meta-runtime-values-bad) · [正确示例](#nuxt-no-page-meta-runtime-values-good) | 禁止在 `definePageMeta` 的立即求值层使用运行时上下文值；该层在构建时被提取到独立代码块，并在组件 setup 之前运行 |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [错误示例](#nuxt-nuxt-config-keys-order-bad) · [正确示例](#nuxt-nuxt-config-keys-order-good) | 优先使用推荐的 Nuxt 配置属性顺序 |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [错误示例](#nuxt-prefer-import-meta-bad) · [正确示例](#nuxt-prefer-import-meta-good) | 优先使用 `import.meta.*`，而不是 `process.*` |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `ecosystem/nuxt-prefer-nuxt-link`

应用内部链接优先使用 NuxtLink

[错误示例](#ecosystem-nuxt-prefer-nuxt-link-bad) · [正确示例](#ecosystem-nuxt-prefer-nuxt-link-good)

默认严重程度: `warning`  
预设: `nuxt`  
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
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

**错误示例**

Nuxt 应用中的内部设置目标使用了普通锚点。

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

**正确示例**

NuxtLink 通过 Nuxt 路由处理相同的内部目标。

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) · [全部规则](all.md)

### `ecosystem/pinia-prefer-store-to-refs`

解构 Pinia store 时优先使用 storeToRefs()

[错误示例](#ecosystem-pinia-prefer-store-to-refs-bad) · [正确示例](#ecosystem-pinia-prefer-store-to-refs-good)

默认严重程度: `warning`  
预设: `ecosystem`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

**错误示例**

直接从 store 解构 `name`，使该值脱离了响应式 store 访问。

```vue annotate="remove:2"
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

**正确示例**

store 保持完整，storeToRefs 为 name 创建响应式引用。

```vue annotate="add:2,3"
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) · [全部规则](all.md)

### `ecosystem/router-link-require-to`

要求 RouterLink 和 NuxtLink 组件具有 `to` 目标

[错误示例](#ecosystem-router-link-require-to-bad) · [正确示例](#ecosystem-router-link-require-to-good)

默认严重程度: `error`  
预设: `ecosystem`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

SFC 的唯一根链接可能从父组件属性继承目标。此示例使用嵌套链接，必须显式提供目标。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/router-link-require-to": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-router-link-require-to-bad"></span>

**错误示例**

嵌套 RouterLink 没有 `to` 目标，不能依赖根节点的属性透传。

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

<span id="ecosystem-router-link-require-to-good"></span>

**正确示例**

`to="/settings"` 显式提供嵌套链接目标。

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [全部规则](all.md)

### `ecosystem/void-link-require-href`

要求 Void Vue Link 组件具有 `href`

[错误示例](#ecosystem-void-link-require-href-bad) · [正确示例](#ecosystem-void-link-require-href-good)

默认严重程度: `error`  
预设: `ecosystem`  
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
        "ecosystem/void-link-require-href": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-require-href-bad"></span>

**错误示例**

从 @void/vue 导入的 Link 省略了 href 目标。

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

<span id="ecosystem-void-link-require-href-good"></span>

**正确示例**

同一导入的 Link 通过 href 接收设置目标。

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) · [全部规则](all.md)

### `ecosystem/void-link-valid-method`

验证 Void Vue Link 的静态 method prop

[错误示例](#ecosystem-void-link-valid-method-bad) · [正确示例](#ecosystem-void-link-valid-method-good)

默认严重程度: `warning`  
预设: `ecosystem`  
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
        "ecosystem/void-link-valid-method": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-valid-method-bad"></span>

**错误示例**

DELETE 操作请求了预取，但预取适用于导航请求。

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

<span id="ecosystem-void-link-valid-method-good"></span>

**正确示例**

移除 prefetch，保留 DELETE 操作，不再预取该非 GET 请求。

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) · [全部规则](all.md)

### `ecosystem/vue-i18n-no-missing-key`

报告本地 SFC 消息中不存在的静态 vue-i18n 键

[错误示例](#ecosystem-vue-i18n-no-missing-key-bad) · [正确示例](#ecosystem-vue-i18n-no-missing-key-good)

默认严重程度: `warning`  
预设: `ecosystem`  
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
        "ecosystem/vue-i18n-no-missing-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

**错误示例**

模板请求 auth.missing，但本地英文消息只声明了 auth.login。

```vue annotate="remove:1"
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

**正确示例**

模板请求本地消息中已存在的 auth.login 键。

```vue annotate="add:1"
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) · [全部规则](all.md)

### `ecosystem/vue-router-prefer-named-link`

RouterLink 优先使用具名路由对象，而不是静态路径字符串

[错误示例](#ecosystem-vue-router-prefer-named-link-bad) · [正确示例](#ecosystem-vue-router-prefer-named-link-good)

默认严重程度: `warning`  
预设: `ecosystem`  
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
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

**错误示例**

RouterLink 的目标是路径字面量，而不是具名路由。

```vue annotate="remove:2"
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

**正确示例**

绑定的路由对象通过 settings 路由名称标识目标。

```vue annotate="add:2"
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) · [全部规则](all.md)

### `ecosystem/vue-router-prefer-named-push`

Vue Router 编程式导航优先使用具名路由对象

[错误示例](#ecosystem-vue-router-prefer-named-push-bad) · [正确示例](#ecosystem-vue-router-prefer-named-push-good)

默认严重程度: `warning`  
预设: `ecosystem`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-push": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

**错误示例**

router.push 接收与当前 URL 写法绑定的路径字符串。

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

**正确示例**

router.push 接收具有稳定 settings 名称的路由对象。

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) · [全部规则](all.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

避免在 Vue Test Utils 测试中对 wrapper.html() 生成快照

[错误示例](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [正确示例](#ecosystem-vue-test-utils-no-html-snapshot-good)

默认严重程度: `warning`  
预设: `ecosystem`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

**错误示例**

断言对整个 wrapper HTML 生成快照，而不是检查预期行为。

```vue annotate="remove:2"
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

**正确示例**

断言检查渲染文本是否包含 Saved。

```vue annotate="add:2"
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) · [全部规则](all.md)

### `nuxt/no-nuxt-config-test-key`

禁止在 Nuxt 配置中设置 `test` 键

[错误示例](#nuxt-no-nuxt-config-test-key-bad) · [正确示例](#nuxt-no-nuxt-config-test-key-good)

默认严重程度: `error`  
预设: `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: Nuxt 配置文件（nuxt.config.ts）  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

**错误示例**

导出的 Nuxt 配置将标识符键 `test` 设为布尔值 `true`，这是此规则拒绝的旧配置形式。

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

<span id="nuxt-no-nuxt-config-test-key-good"></span>

**正确示例**

空配置移除了该布尔 `test` 属性。此示例并不禁止测试配置对象。

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) · [全部规则](all.md)

### `nuxt/no-page-meta-runtime-values`

禁止在 `definePageMeta` 的立即求值层使用运行时上下文值；该层在构建时被提取到独立代码块，并在组件 setup 之前运行

[错误示例](#nuxt-no-page-meta-runtime-values-bad) · [正确示例](#nuxt-no-page-meta-runtime-values-good)

默认严重程度: `error`  
预设: `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-page-meta-runtime-values": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

**错误示例**

构造 `definePageMeta` 对象时立即执行 `useRoute()`，但宏会将元数据提升到 setup 运行时上下文之外。

```vue annotate="remove:2"
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

<span id="nuxt-no-page-meta-runtime-values-good"></span>

**正确示例**

`validate` 接收回调，因此 `useRoute().params.id` 访问延迟到回调运行时。规则区分延迟执行的函数体和立即求值的元数据值。

```vue annotate="add:2"
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) · [全部规则](all.md)

### `nuxt/nuxt-config-keys-order`

优先使用推荐的 Nuxt 配置属性顺序

[错误示例](#nuxt-nuxt-config-keys-order-bad) · [正确示例](#nuxt-nuxt-config-keys-order-good)

默认严重程度: `error`  
预设: `nuxt`  
自动修复: 适用于已支持的诊断  
适用范围: Nuxt 配置文件（nuxt.config.ts）  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-nuxt-config-keys-order-bad"></span>

**错误示例**

配置将 `ssr` 放在 `modules` 之前，与规则推荐的 Nuxt 配置键顺序相反。

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

<span id="nuxt-nuxt-config-keys-order-good"></span>

**正确示例**

将 `modules` 放在 `ssr` 之前，在保留两个值的同时满足规定顺序；修复改变布局，不改变选项含义。

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) · [全部规则](all.md)

### `nuxt/prefer-import-meta`

优先使用 `import.meta.*`，而不是 `process.*`

[错误示例](#nuxt-prefer-import-meta-bad) · [正确示例](#nuxt-prefer-import-meta-good)

默认严重程度: `error`  
预设: `nuxt`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-prefer-import-meta-bad"></span>

**错误示例**

`process.client` 使用旧 Nuxt 环境标志，规则要求将其迁移到 `import.meta`。

```vue annotate="remove:2"
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

<span id="nuxt-prefer-import-meta-good"></span>

**正确示例**

`import.meta.client` 通过替代环境标志显式保留仅浏览器执行的分支。

```vue annotate="add:2"
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [全部规则](all.md)
