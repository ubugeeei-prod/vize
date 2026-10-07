## Summary

With `--type-aware`, `type/no-unsafe-template-binding` reports `v-on` handlers whose types are fully known. `vize check` finds no type errors in the same files. Two messages appear:

- "Template event handler resolves to an unsafe `any` or `unknown` type" for handlers that assign (`selected = []`, `() => (selected = [])`) on native elements and components, and for an inline call (`reset()`) on a component.
- "Template event handler calls a value with an unsafe `any` or `unknown` type" for an arrow function that calls a typed function (`() => reset()`, `() => emit('retry')`) on a component. The same `() => emit('retry')` on a native `<button>` is not reported.

The callees (`reset`, a typed `defineEmits` `emit`) and the assignment target (`Ref<string[]>`) are all typed. In an app where `() => emit('…')` and `() => (x = …)` are common handler styles, this gives dozens of warnings per screen that can't be fixed.

This is not the `paths` / sibling-`<script>` / ambient-augmentation resolution problem of #7206, #7245 and #7215: the repro has no imports other than `vue` and a local SFC.

## Environment

- `vize` 0.432.0 (npm), `vue` 3.5.43, `typescript` 6
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1
- Also reproduces with `vue` 3.6.0-rc.10 and `typescript` 7.0.2

## Reproduction

```sh
mkdir -p repro-unsafe-handler && cd repro-unsafe-handler
npm init -y > /dev/null
npm install -D vize@0.432.0 vue@3.5.43 typescript@6 > /dev/null 2>&1
cat > tsconfig.json <<'JSON'
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["*.vue"]
}
JSON
cat > vize.config.json <<'JSON'
{
  "linter": {
    "preset": "incremental",
    "typeAware": true,
    "rules": { "type/no-unsafe-template-binding": "warn" }
  },
  "typeChecker": { "tsconfig": "tsconfig.json" }
}
JSON
cat > ChildButton.vue <<'VUE'
<script setup lang="ts">
defineProps<{ label: string }>();
defineEmits<{ click: [event: MouseEvent] }>();
</script>

<template>
  <button type="button" @click="$emit('click', $event)">{{ label }}</button>
</template>
VUE
cat > ParentPanel.vue <<'VUE'
<script setup lang="ts">
import { ref } from "vue";
import ChildButton from "./ChildButton.vue";

const emit = defineEmits<{ retry: [] }>();
const selected = ref<string[]>([]);
function reset(): void {
  selected.value = [];
}
</script>

<template>
  <button type="button" @click="emit('retry')">a</button>
  <button type="button" @click="() => emit('retry')">b</button>
  <button type="button" @click="selected = []">c</button>
  <ChildButton label="d" @click="reset" />
  <ChildButton label="e" @click="reset()" />
  <button type="button" @click="() => (selected = [])">f</button>
  <ChildButton label="g" @click="() => (selected = [])" />
  <ChildButton label="h" @click="() => reset()" />
  <ChildButton label="i" @click="() => emit('retry')" />
</template>
VUE
npx vize lint -f plain --help-level none ParentPanel.vue
npx vize check
```

## Actual

```text
ParentPanel.vue:15:33 warning type/no-unsafe-template-binding Template event handler resolves to an unsafe `any` or `unknown` type
ParentPanel.vue:17:34 warning type/no-unsafe-template-binding Template event handler resolves to an unsafe `any` or `unknown` type
ParentPanel.vue:18:33 warning type/no-unsafe-template-binding Template event handler resolves to an unsafe `any` or `unknown` type
ParentPanel.vue:19:34 warning type/no-unsafe-template-binding Template event handler resolves to an unsafe `any` or `unknown` type
ParentPanel.vue:20:40 warning type/no-unsafe-template-binding Template event handler calls a value with an unsafe `any` or `unknown` type
ParentPanel.vue:21:40 warning type/no-unsafe-template-binding Template event handler calls a value with an unsafe `any` or `unknown` type
```

So handlers c, e, f, g, h and i are reported, and a, b and d are not. `vize check` reports `No type errors found!`.

## Expected

No diagnostics. Every handler, assignment target and callee in `ParentPanel.vue` has a concrete type, and wrapping a call in an arrow function or writing it on a component instead of a native element doesn't change that.
