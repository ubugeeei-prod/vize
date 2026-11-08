# Diff

`Diff` renders dependency-free text differences in unified or side-by-side form. It includes hunk
headers, aligned old/new line numbers, and token-level emphasis inside changed line pairs.

```vue
<script setup lang="ts">
import { Diff } from "@vizejs/fresco";
</script>

<template>
  <Diff
    :old-text="previousSource"
    :new-text="nextSource"
    view="split"
    :context="3"
    old-label="before.ts"
    new-label="after.ts"
  />
</template>
```

## Large files

Line matching uses a Myers shortest-edit script and does not add a syntax or diffing dependency to
the core package. Set `viewport-start`, `viewport-size`, and `overscan` when the caller owns a large
scrollable document. Fresco still computes the line edit script, but mounts only the bounded row
window, so Vue node and native render-node counts scale with the viewport instead of file length.

`context` controls how many unchanged rows surround each hunk. Set it to `0` for a compact change
list. `show-line-numbers="false"` removes both gutters when horizontal space is constrained.
