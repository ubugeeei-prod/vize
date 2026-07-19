---
title: 跨档规则
---

<!-- Generated translation; source: rules/cross-file.md -->

# 跨档规则

跨文件诊断由`vize lint --cross-file`发出。他们使用
`vize:croquis/cf/*`诊断代码是因为它们分析的是项目图，而非孤立的图
中尉。这些检查是当前需要跨档文件信息的Patina规则的公开表层。
当密钥为
与`InjectionKey<T>`一同宣告。

下面的每个示例都写成了一个小型多文件夹具。跨文件部分是关系：
组件导入、模板使用、提供/注入键，或从一个文件移动的反应式值
变成另一个。报告本地线路的规则，如`v-for`内的ID，仍会在
这种形状是因为诊断信号在同一项目图传递过程中发射。

## `vize:croquis/cf/unmatched-inject`

报告一个`inject()`，其密钥无法匹配到分析中的可达`provide()`
分量图。

缺点：

```ts
// keys/theme.ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

```vue
<!-- App.vue -->
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

好：

```vue
<!-- App.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

## `vize:croquis/cf/unused-provide`

报告一个图中可达但没有匹配注入器的`provide()`。

缺点：

```vue
<!-- App.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

```vue
<!-- Dashboard.vue -->
<template>
  <h1>Dashboard</h1>
</template>
```

好：

```vue
<!-- App.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

```vue
<!-- Dashboard.vue -->
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

## `vize:croquis/cf/provide-without-symbol`

报告`provide()`使用字符串键的呼叫。符号在文件中保持一个密钥身份，且
避免无关的提供者与注射者之间出现意外匹配。

缺点：

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

好：

```ts
// keys/theme.ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

## `vize:croquis/cf/inject-without-symbol`

报告`inject()`使用字符串键的呼叫。

缺点：

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

好：

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

## `vize:croquis/cf/non-reactive-provide`

报告提供的数值是纯快照，而非反应式值。更喜欢`ref()`或者
`computed()`，另一个文件中的消费者会观察到提供者的更新。

缺点：

```ts
// keys/theme.ts
export const ThemeKey = Symbol("theme");
```

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = { color: "blue" };
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

```vue
<!-- ThemeLabel.vue -->
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

好：

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

好：

```vue
<!-- ThemeProvider.vue -->
<script setup lang="ts">
import { computed, provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const color = ref("blue");
const theme = computed(() => ({ color: color.value }));
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

## `vize:croquis/cf/duplicate-id`

报告在分析的组件图中重复使用静态ID。规则报告了当两名时
不同的组件可以一起渲染，产生相同的DOM ID。

缺点：

```vue
<!-- CheckoutForm.vue -->
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

```vue
<!-- ShippingAddress.vue -->
<template>
  <label for="postal-code">Shipping postal code</label>
  <input id="postal-code" />
</template>
```

```vue
<!-- BillingAddress.vue -->
<template>
  <label for="postal-code">Billing postal code</label>
  <input id="postal-code" />
</template>
```

好：

```vue
<!-- ShippingAddress.vue -->
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Shipping postal code</label>
  <input :id="postalCodeId" />
</template>
```

```vue
<!-- BillingAddress.vue -->
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Billing postal code</label>
  <input :id="postalCodeId" />
</template>
```

## `vize:croquis/cf/non-unique-id`

报告重复模板作用域内的静态ID。有问题的界线是局部的，但规则是有效的
在图通点内，它还会检查文件间的重复ID。

缺点：

```vue
<!-- ResultsList.vue -->
<template>
  <article v-for="result in results" :key="result.id">
    <h2 id="result-title">{{ result.title }}</h2>
  </article>
</template>
```

好：

```vue
<!-- ResultsList.vue -->
<template>
  <article v-for="result in results" :key="result.id">
    <h2 :id="`result-${result.id}-title`">{{ result.title }}</h2>
  </article>
</template>
```

## `vize:croquis/cf/spread-breaks-reactivity`

报告对象会在快照反应状态穿越组件边界后传播。

缺点：

```vue
<!-- UserPage.vue -->
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
const props = defineProps<{ user: { name: string; role: string } }>();
const copiedUser = { ...props.user };
</script>
```

好：

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string; role: string } }>();
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/reassignment-breaks-reactivity`

报告反应式引用，状态跨越文件边界后被替换为纯值。

缺点：

```vue
<!-- UserPage.vue -->
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
let user = toRef(props, "user");

user = props.user;
</script>
```

好：

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/value-extraction-breaks-reactivity`

报告反应值，被复制到长寿命的素装订中。直接反应式螺旋桨
允许结构解构;问题在于如何将这种解构的结合映射到另一个平面上
束缚。

缺点：

```vue
<!-- UserPage.vue -->
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
const { item } = defineProps<{ item: { name: string } }>();
const itemSnapshot = item;
</script>
```

好：

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { computed } from "vue";

const { item } = defineProps<{ item: { name: string } }>();
const itemView = computed(() => item);
</script>
```

## `vize:croquis/cf/destructuring-breaks-reactivity`

报告中，Vue反应道具未覆盖的反应物体被结构化
变形。

缺点：

```vue
<!-- UserPage.vue -->
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const { item } = props;
</script>
```

好：

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

## `vize:croquis/cf/hydration-risk`

报告服务器和客户端之间渲染可能不同的数值。图表有助于说明问题
从路由或父组件到渲染非确定性值的组件。

缺点：

```vue
<!-- App.vue -->
<script setup lang="ts">
import ClockBadge from "./ClockBadge.vue";
</script>

<template>
  <ClockBadge />
</template>
```

```vue
<!-- ClockBadge.vue -->
<template>
  <time>{{ new Date().toLocaleString() }}</time>
</template>
```

好：

```vue
<!-- ClockBadge.vue -->
<script setup lang="ts">
const renderedAt = useState("rendered-at", () => new Date().toISOString());
</script>

<template>
  <time :datetime="renderedAt">{{ renderedAt }}</time>
</template>
```

## `vize:croquis/cf/async-boundary`

报告的异步反应工作可能超过读取状态，除非有清理记录。

缺点：

```vue
<!-- SearchPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

```vue
<!-- SearchResults.vue -->
<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value) => {
    result.value = await load(value);
  },
);
</script>
```

好：

```vue
<!-- SearchResults.vue -->
<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

## `vize:croquis/cf/watcheffect-async`

报告`watchEffect`回调，混合了依赖收集和异步工作。使用显式
用`watch()`源，这样无效请求可以取消过时的请求。

缺点：

```vue
<!-- SearchPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

```vue
<!-- SearchResults.vue -->
<script setup lang="ts">
import { ref, watchEffect } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watchEffect(async () => {
  result.value = await load(props.query);
});
</script>
```

好：

```vue
<!-- SearchResults.vue -->
<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

## `vize:croquis/cf/injected-async-mutation-race`

报告注入状态的异步突变可能与提供者或兄弟注射者竞速。设
提供者拥有共享变异，或将显式事件/动作反馈给该变异。

缺点：

```ts
// keys/store.ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

```vue
<!-- StoreProvider.vue -->
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);
</script>

<template>
  <CountLoader />
  <CountSummary />
</template>
```

```vue
<!-- CountLoader.vue -->
<script setup lang="ts">
import { inject, ref, watch } from "vue";
import { StoreKey } from "./keys/store";

const store = inject(StoreKey)!;
const query = ref("");

watch(query, async (value) => {
  store.count = await loadCount(value);
});
</script>
```

好：

```vue
<!-- StoreProvider.vue -->
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);

function applyLoadedCount(count: number) {
  store.count = count;
}
</script>

<template>
  <CountLoader @loaded="applyLoadedCount" />
  <CountSummary />
</template>
```

```vue
<!-- CountLoader.vue -->
<script setup lang="ts">
import { ref, watch } from "vue";

const emit = defineEmits<{ loaded: [count: number] }>();
const query = ref("");

watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;

  onCleanup(() => {
    active = false;
    controller.abort();
  });

  const count = await loadCount(value, { signal: controller.signal });
  if (active) emit("loaded", count);
});
</script>
```

## 实施方向

跨文件引擎被有意以规则形式文档化，尽管它使用
今天的诊断代码。未来的工作可以在需要时将更多铜绿规则引入这一层
导入、组件关系或项目范围的符号身份，以准确解释问题。
