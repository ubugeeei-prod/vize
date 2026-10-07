### Area

Linter, `type/no-unsafe-template-binding` (in the `opinionated` and `nuxt` presets)

### Version

`vize` 0.432.0

### Minimal reproduction

`package.json`: `vue@3.5`, `typescript@5`

`tsconfig.json`

```json
{
  "compilerOptions": { "target": "ESNext", "module": "ESNext", "moduleResolution": "Bundler", "strict": true, "jsx": "preserve", "skipLibCheck": true },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
```

`src/consts.ts`

```ts
export const PREFIX = "pre";
export const isReady = () => true;
```

`src/Child.vue`

```vue
<script setup lang="ts">
defineProps<{ label?: string; disabled?: boolean }>();
defineEmits<{ play: [] }>();
</script>

<template>
  <button type="button" :disabled="disabled">{{ label }}</button>
</template>
```

`src/App.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";
import { PREFIX, isReady } from "./consts";

const count = ref(0);
const localOk = () => true;
const emit = defineEmits<{ play: [] }>();
</script>

<template>
  <div>
    <Child :disabled="!isReady()" />
    <Child :disabled="isReady()" />
    <Child :disabled="!localOk()" />
    <Child :disabled="localOk()" />
    <Child :label="PREFIX" />
    <Child @play="() => emit('play')" />
    <Child @play="emit('play')" />
    <Child @play="$emit('play')" />
    <Child @play="() => count++" />
  </div>
</template>
```

`vize.config.json`

```json
{ "linter": { "rules": { "type/no-unsafe-template-binding": "warn" } } }
```

```sh
vize lint src            # same result with --type-aware
vize check               # ✓ No type errors found
```

### Actual

```
App.vue:13:23 Template binding resolves to an unsafe `any` or `unknown` type        :disabled="!isReady()"
App.vue:14:23 Template binding resolves to an unsafe `any` or `unknown` type        :disabled="isReady()"
App.vue:15:23 Template binding resolves to an unsafe `any` or `unknown` type        :disabled="!localOk()"
App.vue:16:23 Template binding resolves to an unsafe `any` or `unknown` type        :disabled="localOk()"
App.vue:18:25 Template event handler calls a value with an unsafe `any` or `unknown` type   @play="() => emit('play')"
App.vue:21:19 Template event handler resolves to an unsafe `any` or `unknown` type          @play="() => count++"
```

- Any call of a setup binding (imported or local, `() => true`) in a prop binding is reported, even under `!`, whose result is always `boolean`.
- An arrow handler calling the typed `emit` from `defineEmits` is reported as "calls a value with an unsafe type", while the inline statement `emit('play')` is not.
- An arrow handler `() => count++` is reported as resolving to an unsafe type.
- `:label="PREFIX"`, `emit('play')` and `$emit('play')` are not reported, and `vize check` sees the correct types for all of these.

On a real-world codebase with the `opinionated` preset this rule dominates the output, and most reports are plain calls and arrow handlers like the ones above.

### Expected

No diagnostics: every expression above has a concrete type (`boolean`, `() => void`, `() => number`). The probe seems to resolve the type at the wrong node for call expressions and arrow functions (the virtual TS has `void (!isReady()); // VBind` and `const __vize_handler_… = (() => emit('play'));`, both well-typed).

### Environment

- OS: macOS 26.4.1
- Architecture: arm64
- Node.js: 26.8.1
- Package manager: npm 11.19.0
- Framework: vue 3.5.x, typescript 5.9.3
