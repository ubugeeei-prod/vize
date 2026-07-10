---
title: Type & Script Rules
---

# Type & Script Rules

Type rules use the TypeScript checker when semantic information is needed. Vize reads the same
project shape that TypeScript reads from `tsconfig.json`, so shared ambient names should come from
`compilerOptions.types`, project references, or declaration files.

Script rules are Patina rules for Composition API and Vapor-oriented code. They focus on patterns
that are hard to compile efficiently or hard to reason about in Vapor mode.

Type-aware linting is opt-in. Enable it with `linter.typeAware: true`, `vize lint --type-aware`, or
by explicitly enabling a `type/*` rule. `type/no-reactivity-loss` can be enabled directly with
`vize lint --strict-reactivity`. If Corsa cannot be started, Patina reports `type/corsa-runtime` and
skips the checker-backed rule pass instead of silently dropping the configured rules.

`--type-aware` uses the same Corsa executable resolution as `vize check`; configure
`typeChecker.corsaPath` when the project needs an explicit `tsgo` or Corsa binary. Defaults stay
zero-cost: Patina does not parse SFCs for checker-backed linting or start Corsa unless the flag,
`linter.typeAware`, or an explicitly enabled `type/*` rule opts in.

```ts
export default defineConfig({
  linter: { typeAware: true },
});
```

## `type/require-typed-props`

Requires `defineProps` to be typed instead of using a runtime array declaration.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
const props = defineProps(["label", "count"]);
</script>
```

Good:

```vue
<script setup lang="ts">
const props = defineProps<{
  label: string;
  count: number;
}>();
</script>
```

## `type/require-typed-emits`

Requires `defineEmits` to describe the emitted event payloads.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
const emit = defineEmits(["save"]);

emit("save", form.value);
</script>
```

Good:

```vue
<script setup lang="ts">
const emit = defineEmits<{
  save: [payload: FormValue];
}>();

emit("save", form.value);
</script>
```

## `type/no-unsafe-template-binding`

Reports template bindings that resolve to unsafe values such as `any`. The rule is checker-backed,
so it follows imported types and project configuration.

Default severity: `warning`  
Presets: `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
const payload: any = await loadPayload();
</script>

<template>
  <p>{{ payload.title }}</p>
</template>
```

Good:

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

Reports promises that are created but not awaited, returned, or intentionally handled.
The check covers both `<script>` and template expressions.

Default severity: `warning`  
Presets: `nuxt`, `opinionated`

Bad:

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

Good:

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

Reports plain snapshots of reactive values that are used across flows. The rule also runs when
`vize lint --strict-reactivity` is enabled.

Default severity: `warning`  
Presets: `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const item = props.item;
</script>
```

Good:

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

## Checker Configuration

The type-aware rules do not need a separate Vize `globals` field for TypeScript names. Prefer
TypeScript-native configuration:

Bad:

```ts
export default {
  globals: ["definePageMeta", "process"],
};
```

Good:

```json
{
  "compilerOptions": {
    "types": ["node", "nuxt/app"]
  }
}
```

## `script/no-options-api`

Reports Options API component definitions in Vapor-oriented presets.

Default severity: `error`  
Presets: `nuxt`, `opinionated`

Bad:

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

Good:

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## `script/no-next-tick`

Reports `nextTick()` in Vapor-oriented components. Prefer direct refs, lifecycle hooks, or state
flow that does not depend on the next DOM flush.

Default severity: `error`  
Presets: `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts" vapor>
await nextTick();
input.value?.focus();
</script>
```

Good:

```vue
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>
```

## `script/no-get-current-instance`

Reports `getCurrentInstance()` in Vapor-oriented components. It reaches into runtime internals that
Vapor cannot safely optimize.

Default severity: `error`  
Presets: `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts" vapor>
const instance = getCurrentInstance();
const app = instance?.appContext.app;
</script>
```

Good:

```vue
<script setup lang="ts" vapor>
const appConfig = useAppConfig();
</script>
```
