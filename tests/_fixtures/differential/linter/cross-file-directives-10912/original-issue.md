## Versions

`vize` **0.593.0** (npm), Node 26.6.0, macOS arm64.

## Reproduction

Repro directory: `repro/xf-disable-directive/` (3 SFCs, no config).

`src/Editor.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";

const { initial } = defineProps<{ initial: { name: string } }>();

// eslint-disable-next-line croquis/cf/spread-breaks-reactivity -- a draft copy of the prop
const draft = reactive({ ...initial });
</script>

<template>
  <p>{{ draft.name }}</p>
</template>
```

`src/PageA.vue`

```vue
<script setup lang="ts">
import Editor from "./Editor.vue";
</script>

<template>
  <div id="panel">
    <Editor :initial="{ name: 'a' }" />
  </div>
</template>
```

`src/PageB.vue`

```vue
<template>
  <!-- eslint-disable-next-line croquis/cf/duplicate-id -->
  <div id="panel">b</div>
</template>
```

```
$ vize lint -f plain --no-config --cross-file "src/**/*.vue"
  src/Editor.vue:7:26 error cross-file vize:croquis/cf/spread-breaks-reactivity: Spreading 'initial' creates a non-reactive copy
  src/PageB.vue:3:8 warning cross-file vize:croquis/cf/duplicate-id: Element ID 'panel' is used in 2 different locations across files
```

The same happens with every name I tried in the directive: `cross-file`, `vize:croquis/cf/spread-breaks-reactivity`
(the printed id), `croquis/cf/spread-breaks-reactivity` (the id `linter.rules` accepts), the bare rule name, and with
`vize-disable-next-line` / `oxlint-disable-next-line` / a file-level `/* eslint-disable */`. The directives work for
the ordinary rules in the same files.

`linter.rules: { "croquis/cf/spread-breaks-reactivity": "off" }` does turn the rule off (and `entries[].linter.rules`
turns it off per file), so the rule id is known — only the directives are not applied to cross-file diagnostics.

## Expected

`eslint-disable-next-line croquis/cf/<rule>` above the reported line suppresses that cross-file diagnostic, like it does
for the other rules (and `--report-unused-disable-directives` reports it when it suppresses nothing).

## Why it matters

Several cross-file rules report code that is correct on purpose, and the only way to keep `--max-warnings 0` is to turn
the rule off for the whole project or for whole files:

- `spread-breaks-reactivity` / `value-extraction-breaks-reactivity`: a prop copied into local state as the initial
  value of a form draft (re-synced when the form opens) is exactly what the message describes, and it is intended.
- `duplicate-id`: two pages that are never rendered together.
- `unmatched-inject` "Conditionally Defaulted Inject": an `inject(key, default)` whose default is the intended answer
  outside the provider.

A line-level opt-out (with a reason) keeps the rule on for everything else.
