---
title: Cross-File Analyzer Rules
---

# Cross-File Analyzer Rules

Cross-file diagnostics are emitted by `vize lint --cross-file`. They use
`vize:croquis/cf/*` diagnostic codes because they analyze a project graph rather than one isolated
SFC. These checks are the current public surface for Patina rules that need cross-file information.
Provider and injector value type mismatches are left to TypeScript diagnostics when the key is
declared with `InjectionKey<T>`.

## `vize:croquis/cf/unmatched-inject`

Reports an `inject()` whose key cannot be matched to a reachable `provide()`.

Bad:

```vue
<!-- Child.vue -->
<script setup lang="ts">
import { ThemeKey } from "./keys";

const theme = inject(ThemeKey);
</script>
```

Good:

```vue
<!-- Parent.vue -->
<script setup lang="ts">
import { ThemeKey } from "./keys";

provide(ThemeKey, theme);
</script>

<template>
  <Child />
</template>
```

## `vize:croquis/cf/unused-provide`

Reports a `provide()` that has no matching injector in the analyzed graph.

Bad:

```vue
<script setup lang="ts">
provide(ThemeKey, theme);
</script>
```

Good:

```vue
<!-- Child.vue -->
<script setup lang="ts">
const theme = inject(ThemeKey);
</script>
```

## `vize:croquis/cf/provide-without-symbol`

Reports `provide()` calls that use string keys. Symbols preserve type identity across files.

Bad:

```vue
<script setup lang="ts">
provide("theme", theme);
</script>
```

Good:

```ts
// keys.ts
export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

```vue
<script setup lang="ts">
provide(ThemeKey, theme);
</script>
```

## `vize:croquis/cf/inject-without-symbol`

Reports `inject()` calls that use string keys.

Bad:

```vue
<script setup lang="ts">
const theme = inject("theme");
</script>
```

Good:

```vue
<script setup lang="ts">
import { ThemeKey } from "./keys";

const theme = inject(ThemeKey);
</script>
```

## `vize:croquis/cf/non-reactive-provide`

Reports provided values that are plain snapshots instead of reactive values.

Bad:

```vue
<script setup lang="ts">
const theme = { color: "blue" };

provide(ThemeKey, theme);
</script>
```

Good:

```vue
<script setup lang="ts">
const theme = ref({ color: "blue" });

provide(ThemeKey, theme);
</script>
```

Good:

```vue
<script setup lang="ts">
const color = ref("blue");
const theme = computed(() => ({ color: color.value }));

provide(ThemeKey, theme);
</script>
```

## `vize:croquis/cf/duplicate-id`

Reports duplicate static IDs across the analyzed component graph.

Bad:

```vue
<!-- SearchBox.vue -->
<template>
  <input id="search" />
</template>
```

```vue
<!-- HeaderSearch.vue -->
<template>
  <input id="search" />
</template>
```

Good:

```vue
<script setup lang="ts">
const searchId = useId();
</script>

<template>
  <input :id="searchId" />
</template>
```

## `vize:croquis/cf/non-unique-id`

Reports IDs inside repeated template scopes.

Bad:

```vue
<template>
  <div v-for="item in items" id="row" :key="item.id">
    {{ item.name }}
  </div>
</template>
```

Good:

```vue
<template>
  <div v-for="item in items" :id="`row-${item.id}`" :key="item.id">
    {{ item.name }}
  </div>
</template>
```

## `vize:croquis/cf/spread-breaks-reactivity`

Reports object spreads that snapshot reactive state before passing it to another component or flow.

Bad:

```vue
<script setup lang="ts">
const props = defineProps<{ user: User }>();
const copied = { ...props.user };
</script>
```

Good:

```vue
<script setup lang="ts">
const props = defineProps<{ user: User }>();
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/reassignment-breaks-reactivity`

Reports reactive references that are reassigned to plain values.

Bad:

```vue
<script setup lang="ts">
let user = toRef(props, "user");

user = props.user;
</script>
```

Good:

```vue
<script setup lang="ts">
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/value-extraction-breaks-reactivity`

Reports a reactive value that is copied into a long-lived plain binding.

Bad:

```vue
<script setup lang="ts">
const count = injectedCount.value;
const { item } = defineProps<{ item: Item }>();
const item2 = item;
</script>

<template>
  <p>{{ count }}</p>
</template>
```

Good:

```vue
<script setup lang="ts">
const count = injectedCount;
const { item } = defineProps<{ item: Item }>();
const item2 = computed(() => item);
</script>

<template>
  <p>{{ count }}</p>
</template>
```

## `vize:croquis/cf/destructuring-breaks-reactivity`

Reports destructuring of reactive objects that are not covered by Vue's reactive props destructure
transform.

Bad:

```vue
<script setup lang="ts">
const state = reactive({ item });
const { item: localItem } = state;
</script>
```

Good:

```vue
<script setup lang="ts">
const state = reactive({ item });
const localItem = toRef(state, "item");
</script>
```

## `vize:croquis/cf/hydration-risk`

Reports values that can render differently between the server and the client.

Bad:

```vue
<template>
  <p>{{ new Date().toLocaleString() }}</p>
</template>
```

Good:

```vue
<script setup lang="ts">
const renderedAt = useState("rendered-at", () => new Date().toISOString());
</script>

<template>
  <time :datetime="renderedAt">{{ renderedAt }}</time>
</template>
```

## `vize:croquis/cf/async-boundary`

Reports async reactive work that can outlive the state it reads unless cleanup is registered.

Bad:

```vue
<script setup lang="ts">
watch(query, async () => {
  result.value = await load(query.value);
});
</script>
```

Good:

```vue
<script setup lang="ts">
watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;

  onCleanup(() => {
    active = false;
    controller.abort();
  });

  const next = await load(value, { signal: controller.signal });
  if (active) result.value = next;
});
</script>
```

## `vize:croquis/cf/watcheffect-async`

Reports `watchEffect` callbacks that mix dependency collection with async work.

Bad:

```vue
<script setup lang="ts">
watchEffect(async () => {
  result.value = await load(query.value);
});
</script>
```

Good:

```vue
<script setup lang="ts">
watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;

  onCleanup(() => {
    active = false;
    controller.abort();
  });

  const next = await load(value, { signal: controller.signal });
  if (active) result.value = next;
});
</script>
```

## `vize:croquis/cf/injected-async-mutation-race`

Reports async mutations to injected state that can race with the provider or sibling injectors.

Bad:

```vue
<script setup lang="ts">
const store = inject(StoreKey)!;

watch(query, async () => {
  store.count = await loadCount(query.value);
});
</script>
```

Good:

```vue
<script setup lang="ts">
const emit = defineEmits<{ loaded: [count: number] }>();

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

## Analyzer Direction

The cross-file analyzer is intentionally shaped like rule documentation, even though it uses
diagnostic codes today. Future work can promote more Patina rules into this layer when they need
imports, component relationships, or project-wide symbol identity to explain a bug accurately.
