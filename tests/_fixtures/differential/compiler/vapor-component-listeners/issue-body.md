## Summary

Two bugs in how Vapor output passes `v-on` listeners to a **component**:

1. **Arrow function with TypeScript annotations.** `@ping="(a: number) => log.push(a)"` is not recognized as a function expression and is wrapped as an inline statement: `onPing: () => (($event) => ((a) => …))`. Emitting `ping` calls the wrapper, which only creates the arrow function, so the handler never runs. The same handler without annotations (`(a) => …`) works, and VDOM output handles both.
2. **Kebab-case listener names.** `@update-thing="…"` is emitted as the prop key `"onUpdate-thing"`. A child that declares and emits the camelCase event (`defineEmits<{ updateThing: [] }>()`, `emit('updateThing')`) looks up `onUpdateThing` and never finds it. Vue's compilers normalize component listener names (`onUpdateThing`).

Same output from `@vizejs/native` `compileSfc(src, { vapor: true })`.

## Environment

- `vize` 0.432.0 (npm), `vue` / `@vue/compiler-sfc` 3.6.0-rc.10 for comparison
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir repro && cd repro
cat > App.vue <<'VUE'
<script setup vapor lang="ts">
import { ref } from "vue";
import Child from "./Child.vue";
const log = ref<string[]>([]);
</script>

<template>
  <Child
    @typed="(a: number) => log.push('typed ' + a)"
    @plain="(a) => log.push('plain ' + a)"
    @update-thing="log.push('kebab')"
  />
  <p>{{ log }}</p>
</template>
VUE
cat > Child.vue <<'VUE'
<script setup vapor lang="ts">
const emit = defineEmits<{ typed: [a: number]; plain: [a: number]; updateThing: [] }>();
</script>

<template>
  <button @click="emit('typed', 1); emit('plain', 2); emit('updateThing')">go</button>
</template>
VUE
npx vize@0.432.0 build --no-config -o out App.vue && grep -n -E '"?on[A-Z]' out/App.js
```

## Actual

```js
onTyped: () => (($event) => ((a) => _ctx.log.push("typed " + a))),
onPlain: () => ((a) => _ctx.log.push("plain " + a)),
"onUpdate-thing": () => (($event) => _ctx.log.push("kebab"))
```

Mounted, clicking "go" logs only `["plain 2"]`.

## Expected

`["typed 1", "plain 2", "kebab"]`. `@vue/compiler-sfc` emits `onTyped` / `onPlain` as the arrow functions themselves and the third key as `onUpdateThing`.
