---
title: 类型与脚本规则
---

<!-- Generated translation; source: rules/type-and-script.md -->

# 类型与文字规则

类型规则在需要语义信息时使用 TypeScript 检查器。Vize的读数也是一样的
TypeScript 从`tsconfig.json`读取的项目形状，因此共享的环境名称应来自
`compilerOptions.types`、项目参考或声明文件。

脚本规则是用于合成API和面向蒸汽代码的Patina规则。他们关注的是模式
这些问题难以高效编译，或者在蒸汽模式下难以理清。

类型感知的 linting 是自愿选择的。通过`linter.typeAware: true`、`vize lint --type-aware`或
通过明确启用`type/*`规则。`type/no-reactivity-loss`可以直接通过以下方式启用
`vize lint --strict-reactivity`。如果科尔萨无法启动，帕蒂纳报告`type/corsa-runtime`
跳过了跳过有检查器的规则传递，而不是悄无声息地放弃配置的规则。

`--type-aware` 使用与 `vize check` 相同的 Corsa 可执行解析;配置
`typeChecker.corsaPath`项目需要明确的`tsgo`或Corsa二进制时。违约保持
零成本：Patina 不会解析 SFC 以进行棋子背衬 linting，也不会启动 Corsa，除非旗帜，
`linter.typeAware`，或者明确启用的`type/*`规则选择加入。

```ts
export default defineConfig({
  linter: { typeAware: true },
});
```

## `type/require-typed-props`

需要用类型化`defineProps`而不是运行时数组声明。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<script setup lang="ts">
const props = defineProps(["label", "count"]);
</script>
```

好：

```vue
<script setup lang="ts">
const props = defineProps<{
  label: string;
  count: number;
}>();
</script>
```

## `type/require-typed-emits`

需要`defineEmits`描述已发射的事件有效载荷。

默认严重程度：`warning`
预设：`happy-path`，`nuxt`，`opinionated`

缺点：

```vue
<script setup lang="ts">
const emit = defineEmits(["save"]);

emit("save", form.value);
</script>
```

好：

```vue
<script setup lang="ts">
const emit = defineEmits<{
  save: [payload: FormValue];
}>();

emit("save", form.value);
</script>
```

## `type/no-unsafe-template-binding`

报告模板绑定，解析为不安全值，如`any`。该规则有棋子支持，
所以它遵循导入的类型和项目配置。

默认严重程度：`warning`
预设：`nuxt`，`opinionated`

缺点：

```vue
<script setup lang="ts">
const payload: any = await loadPayload();
</script>

<template>
  <p>{{ payload.title }}</p>
</template>
```

好：

```vue
<script setup lang="ts">
type Payload = { title: string };

const payload = await loadPayload<Payload>();
</script>

<template>
  <p>{{ payload.title }}</p>
</template>
```

## `type/no-floating-promises`

报告承诺是创造但未被等待、未退回或有意处理的。
该检查涵盖`<script>`和模板表达式。

默认严重程度：`warning`
预设：`nuxt`，`opinionated`

缺点：

```vue
<script setup lang="ts">
function submit() {
  saveForm(form.value);
}
</script>

<template>
  <button @click="saveForm(form)">Save</button>
  <p>{{ loadPreview() }}</p>
</template>
```

好：

```vue
<script setup lang="ts">
type Preview = { title: string };

async function submit() {
  await saveForm(form.value);
}

const preview = ref<Preview | null>(null);

async function loadPreviewIntoState() {
  preview.value = await loadPreview();
}
</script>

<template>
  <button @click="void submit()">Save</button>
  <button @click="void loadPreviewIntoState()">Preview</button>
  <PreviewPanel v-if="preview" :preview="preview" />
</template>
```

## `type/no-reactivity-loss`

报告跨流使用的反应式值的纯快照。该规则还适用于
`vize lint --strict-reactivity`已启用。

默认严重程度：`warning`
预设：`nuxt`，`opinionated`

缺点：

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const item = props.item;
</script>
```

好：

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

## 检查器配置

类型感知规则不需要为TypeScript名称单独设置一个Vize `globals`字段。更喜欢
TypeScript原生配置：

缺点：

```ts
export default {
  globals: ["definePageMeta", "process"],
};
```

好：

```json
{
  "compilerOptions": {
    "types": ["node", "nuxt/app"]
  }
}
```

## `script/no-options-api`

报告面向蒸汽预设的选项 API 组件定义。

默认严重程度：`error`
预设：`nuxt`，`opinionated`

缺点：

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

好：

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## `script/no-next-tick`

报告以蒸汽为导向的部件`nextTick()`。更倾向于直接引用、生命周期钩子或状态
流量不依赖于下一次DOM冲洗。

默认严重程度：`error`
预设：`nuxt`，`opinionated`

缺点：

```vue
<script setup lang="ts" vapor>
await nextTick();
input.value?.focus();
</script>
```

好：

```vue
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>
```

## `script/no-get-current-instance`

报告以蒸汽为导向的部件`getCurrentInstance()`。它深入运行时内部
蒸汽无法安全地进行优化。

默认严重程度：`error`
预设：`nuxt`，`opinionated`

缺点：

```vue
<script setup lang="ts" vapor>
const instance = getCurrentInstance();
const app = instance?.appContext.app;
</script>
```

好：

```vue
<script setup lang="ts" vapor>
const appConfig = useAppConfig();
</script>
```
