## Summary

`vize:croquis/cf/browser-api-ssr` ("Browser API used in potentially SSR context") is reported for code that never runs during SSR:

1. **A function used only as an event handler** (`@click="goHome"` → `window.location.assign("/")`). Event handlers never run on the server.
2. **A non-immediate `watch` callback**, even when the access is wrapped in `if (!import.meta.env.SSR) { … }`. A non-immediate `watch` callback doesn't run during SSR, and the explicit guard is ignored.

The same guard *is* honored in a `watchEffect` callback (`if (import.meta.env.SSR) return;`, not reported) and at the top level of `<script setup>`, so the analysis is inconsistent between these scopes. The suggestion also only mentions `import.meta.client`, which is Nuxt-specific. A plain Vite SSR/SSG app uses `import.meta.env.SSR`.

The positions in the output are also wrong. That is a separate issue, #7907 (positions resolved against `<template>`).

## Environment

- `vize` 0.432.0 (npm)
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir -p repro-browser-api-ssr && cd repro-browser-api-ssr
cat > HomeButton.vue <<'VUE'
<script setup lang="ts">
function goHome(): void {
  window.location.assign("/");
}
</script>

<template>
  <button type="button" @click="goHome">Home</button>
</template>
VUE
cat > GuardedWatch.vue <<'VUE'
<script setup lang="ts">
import { ref, watch } from "vue";

const lang = ref("en");
const title = ref("");
watch(lang, () => {
  if (!import.meta.env.SSR) {
    title.value = document.title;
  }
});
</script>

<template>
  <p>{{ lang }} {{ title }}</p>
</template>
VUE
cat > GuardedEffect.vue <<'VUE'
<script setup lang="ts">
import { ref, watchEffect } from "vue";

const lang = ref("en");
watchEffect(() => {
  if (import.meta.env.SSR) return;
  document.documentElement.lang = lang.value;
});
</script>

<template>
  <p>{{ lang }}</p>
</template>
VUE
npx vize@0.432.0 lint --no-config --preset opinionated --cross-file -f plain HomeButton.vue GuardedWatch.vue GuardedEffect.vue | grep -A5 'cross-file'
```

## Actual

```text
  GuardedWatch.vue:16:1 warning cross-file vize:croquis/cf/browser-api-ssr: Browser API used in potentially SSR context
      Suggestion: Wrap in onMounted() or use import.meta.client check
  HomeButton.vue:8:29 warning cross-file vize:croquis/cf/browser-api-ssr: Browser API used in potentially SSR context
      Suggestion: Wrap in onMounted() or use import.meta.client check
```

`GuardedEffect.vue` is not reported.

## Expected

No diagnostics for any of the three files:

- browser APIs inside functions that are only referenced as `v-on` handlers (or only called from them / from `onMounted`) are client-only;
- non-immediate `watch` callbacks don't run during SSR, and an `import.meta.env.SSR` / `import.meta.client` / `typeof window` guard inside any callback should be honored the same way it is in `watchEffect`;
- the suggestion should mention `import.meta.env.SSR` (Vite) as well as `import.meta.client` (Nuxt).
