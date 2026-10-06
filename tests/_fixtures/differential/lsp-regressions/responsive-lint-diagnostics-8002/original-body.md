## Version

- `vize` 0.432.0 (npm, `node_modules/.bin/vize lsp --stdio`), serverInfo `vize-maestro 0.432.0`
- `vue` 3.5.41, Node 26.6.0, macOS (arm64)

## Minimal reproduction

`tsconfig.json`: `moduleResolution: "Bundler"`, `strict: true`, `include: ["src/**/*.ts", "src/**/*.vue"]`.

`vize.config.json`:

```json
{
  "linter": { "preset": "opinionated" },
  "languageServer": { "lint": true, "typecheck": true, "editor": true }
}
```

`src/App.vue`:

```vue
<script setup lang="ts">
import { ref } from "vue";

const count = ref(0);
</script>

<template>
  <img src="/logo.png" alt="" />
  <span>{{ count }}</span>
</template>
```

## Steps

`vize lsp --stdio`, `initialize` with `initializationOptions` `{ "lint": true, "typecheck": <true|false>, "editor": true }`, `didOpen`, wait until diagnostics settle, then three times: `didChange` that removes `alt=""` (→ `a11y/alt-text`), wait for the diagnostic, `didChange` back. Time from `didChange` to the `publishDiagnostics` that carries `a11y/alt-text`.

## Actual

```
## typecheck: true
  edit 1: first publish 1485 ms, a11y/alt-text published after 1485 ms
  edit 2: first publish 1144 ms, a11y/alt-text published after 1144 ms
  edit 3: first publish 927 ms, a11y/alt-text published after 927 ms
  server log (last edit):
    03:07:00.093826Z vize_maestro::ide::diagnostics::native: sync diagnostics count: 1
    03:07:00.981214Z vize_maestro::ide::diagnostics::native: corsa diagnostics count: 0
    03:07:00.981241Z vize_maestro::server::diagnostic_publishing: sending collected diagnostics for <App.vue> version 6

## typecheck: false
  edit 1: first publish 205 ms, a11y/alt-text published after 205 ms
  edit 2: first publish 51 ms, a11y/alt-text published after 51 ms
  edit 3: first publish 50 ms, a11y/alt-text published after 50 ms
```

The lint result is ready (`sync diagnostics count: 1`) about 0.9 s before it is sent; it waits for `corsa diagnostics count` and goes out in the same notification.

`didOpen` does not wait — with the warning already in the file, the server publishes twice (lint first, then the merged set):

```
[typecheck: true]  didOpen: a11y/alt-text published after 351 ms; publishes: 351ms:1 2850ms:1
[typecheck: false] didOpen: a11y/alt-text published after 235 ms; publishes: 235ms:1
```

so the two-phase publish exists, only the edit path skips it. The wait is the length of the type pass, so it grows with the project: in a larger app (~290 SFCs, ~850 `.ts`, a 1,080-line page component; shared machine under load) the same kind of edit (an `<img>` without `alt` plus a `v-for` without `key`) showed up **61 s** after `didChange` (`sync diagnostics count: 3` at 02:57:47, published at 02:58:46). When the type pass times out (`typecheck-timed-out`, default bound 60 s), lint results are held for the whole timeout.

## Expected

Diagnostics that do not need the checker (template/script parse errors, Patina lint, `component-required-props`, ecosystem) are published as soon as they are computed, and the type diagnostics are merged in by a second `publishDiagnostics` when Corsa answers (the server already re-publishes for later versions, so the client handles successive notifications fine). To avoid type squiggles blinking out on every keystroke, the early publish can carry the previous version's type diagnostics until the new ones arrive. Alternatively, support pull diagnostics (`textDocument/diagnostic`) with separate result ids, but publishing the sync set first is the smaller change.

## Why

Lint feedback is the fast path of the editor loop; with `typecheck` on it becomes as slow as the slowest type check. ESLint and Vue - Official run as separate servers, so their lint squiggles never wait for `tsserver`; users switching to a single Vize server would see lint get slower. The parse-error path already publishes immediately (`collect_async: Corsa diagnostics skipped after parser error` → sent at once), so the plumbing for an early publish exists.
