## Summary

With `--cross-file`, `vize:croquis/cf/browser-api-ssr` is reported for a browser API used in `<script setup>`, but the position points into the template, or past the end of the file:

- `WindowWidth.vue`: `window` is at **2:15**, the diagnostic is at **6:15** (inside `<template>`).
- `PageTitle.vue` (11 lines): `document` is at **6:35**, the diagnostic is at **12:1**, past the last line.

The script-relative offset seems to be added to the start of the `<template>` block instead of the `<script>` block. 2:15 → 6:15 is the same offset counted from the template content, and when the offset is larger than the template content, the position runs past the end of the file. Editors and CI annotations then point at unrelated template lines or at nothing. In a real project, the reported lines landed inside `<style>` blocks. The other cross-file diagnostics seen there (`croquis/cf/uncaught-error`) also pointed at unrelated lines, such as comments, so the mapping may be shared.

## Environment

- `vize` 0.432.0 (npm)
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir -p repro-cross-file-loc && cd repro-cross-file-loc
cat > WindowWidth.vue <<'VUE'
<script setup lang="ts">
const width = window.innerWidth;
</script>

<template>
  <p>{{ width }}</p>
</template>
VUE
cat > PageTitle.vue <<'VUE'
<script setup lang="ts">
import { ref, watch } from "vue";

const lang = ref("en");
const title = ref("");
watch(lang, () => { title.value = document.title; });
</script>

<template>
  <p>{{ lang }} {{ title }}</p>
</template>
VUE
wc -l WindowWidth.vue PageTitle.vue
npx vize@0.432.0 lint --no-config --preset opinionated --cross-file -f plain --help-level none WindowWidth.vue PageTitle.vue | grep cross-file
```

## Actual

```text
      7 WindowWidth.vue
     11 PageTitle.vue
  PageTitle.vue:12:1 warning cross-file vize:croquis/cf/browser-api-ssr: Browser API used in potentially SSR context
  WindowWidth.vue:6:15 warning cross-file vize:croquis/cf/browser-api-ssr: Browser API used in potentially SSR context
```

## Expected

`WindowWidth.vue:2:15` (the `window` reference) and `PageTitle.vue:6:35` (the `document` reference), the positions in the original file. Whether `PageTitle.vue` should be reported at all is a separate question: a non-immediate `watch` callback never runs during SSR.
