---
title: Cross-File Analyzer Rules
---

# Cross-File Analyzer Rules

Cross-file diagnostics are emitted by `vize lint --cross-file`. They use
`vize:croquis/cf/*` diagnostic codes because they analyze a project graph rather than one isolated
SFC. These checks are the current public surface for Patina rules that need cross-file information.
Provider and injector value type mismatches are left to TypeScript diagnostics when the key is
declared with `InjectionKey<T>`.

Each example below is written as a tiny multi-file fixture. The cross-file part is the relationship:
component imports, template usage, provide/inject keys, or reactive values that move from one file
into another. Rules that report a local line, such as an ID inside `v-for`, are still documented in
that shape because the diagnostic is emitted during the same project-graph pass.

## `vize:croquis/cf/unmatched-inject`

Reports an `inject()` whose key cannot be matched to a reachable `provide()` in the analyzed
component graph.

Bad:

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

Good:

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

Reports a `provide()` that is reachable in the graph but has no matching injector.

Bad:

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

Good:

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

Reports `provide()` calls that use string keys. Symbols preserve one key identity across files and
avoid accidental matches between unrelated providers and injectors.

Bad:

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

Good:

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

Reports `inject()` calls that use string keys.

Bad:

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

Good:

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

Reports provided values that are plain snapshots instead of reactive values. Prefer `ref()` or
`computed()` so consumers in another file observe updates from the provider.

Bad:

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

Good:

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

Good:

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

Reports duplicate static IDs across the analyzed component graph. The analyzer reports this when two
different components can be rendered together and produce the same DOM ID.

Bad:

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

Good:

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

Reports static IDs inside repeated template scopes. The problematic line is local, but the rule runs
inside the graph pass that also checks duplicate IDs across files.

Bad:

```vue
<!-- ResultsList.vue -->
<template>
  <article v-for="result in results" :key="result.id">
    <h2 id="result-title">{{ result.title }}</h2>
  </article>
</template>
```

Good:

```vue
<!-- ResultsList.vue -->
<template>
  <article v-for="result in results" :key="result.id">
    <h2 :id="`result-${result.id}-title`">{{ result.title }}</h2>
  </article>
</template>
```

## `vize:croquis/cf/spread-breaks-reactivity`

Reports object spreads that snapshot reactive state after it crosses a component boundary.

Bad:

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

Good:

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string; role: string } }>();
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/reassignment-breaks-reactivity`

Reports reactive references that are replaced with plain values after state crosses a file boundary.

Bad:

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

Good:

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
const user = toRef(props, "user");
</script>
```

## `vize:croquis/cf/value-extraction-breaks-reactivity`

Reports a reactive value that is copied into a long-lived plain binding. Direct reactive props
destructure is allowed; the problem is assigning that destructured binding into another plain
binding.

Bad:

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

Good:

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { computed } from "vue";

const { item } = defineProps<{ item: { name: string } }>();
const itemView = computed(() => item);
</script>
```

## `vize:croquis/cf/destructuring-breaks-reactivity`

Reports destructuring of reactive objects that are not covered by Vue's reactive props destructure
transform.

Bad:

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

Good:

```vue
<!-- UserSummary.vue -->
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

## `vize:croquis/cf/hydration-risk`

Reports values that can render differently between the server and the client. The graph helps point
from the route or parent component to the component that renders the nondeterministic value.

Bad:

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

Good:

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

Reports async reactive work that can outlive the state it reads unless cleanup is registered.

Bad:

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

Good:

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

Reports `watchEffect` callbacks that mix dependency collection with async work. Use an explicit
source with `watch()` so invalidation can cancel stale requests.

Bad:

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

Good:

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

Reports async mutations to injected state that can race with the provider or sibling injectors. Let
the provider own the shared mutation, or pass an explicit event/action back to it.

Bad:

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

Good:

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

## Analyzer Direction

The cross-file analyzer is intentionally shaped like rule documentation, even though it uses
diagnostic codes today. Future work can promote more Patina rules into this layer when they need
imports, component relationships, or project-wide symbol identity to explain a bug accurately.
