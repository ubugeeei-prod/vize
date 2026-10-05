## Summary

In a Vite+ project, `vp run typecheck` with a relative `typecheck.tsconfig` checks no files at all and exits 0:

```
No .vue, .ts, .tsx, .mts, .cts, .js, .jsx, .mjs, .cjs, .d.ts, .d.mts, or .d.cts files found matching inputs: []
```

`vize check --tsconfig <same file>` in the same project finds the errors. Worked in 0.429.1, where the task checked the whole project.

Probably the same cause as #7307: the runner writes the merged config to the OS temp directory (`runNative()` → `os.tmpdir()/.vize-vp-<uuid>.json`) and `relocateTaskConfig()` makes `basePath`, `ignores` and `entries` absolute, but not `typeChecker.tsconfig`. The native checker then resolves `"tsconfig.app.json"` next to the temp file, finds nothing, and reports success.

## Reproduction

`package.json`: `vite-plus@0.2.7`, `@vizejs/vite-plugin@0.432.0`, `vize@0.432.0`, `vue@3.5.42`, `typescript@5.9.3`

`vite.config.mts`

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: false,
  typecheck: { tsconfig: "tsconfig.app.json" }
});
```

`tsconfig.app.json`

```json
{
  "compilerOptions": { "target": "ESNext", "module": "ESNext", "moduleResolution": "Bundler", "strict": true, "skipLibCheck": true, "noEmit": true },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
```

`src/Counter.vue`

```vue
<script setup lang="ts">
const count: number = "not a number";
</script>

<template>
  <p>{{ count }}</p>
</template>
```

```
$ vp run typecheck; echo "exit=$?"
No .vue, .ts, .tsx, .mts, .cts, .js, .jsx, .mjs, .cjs, .d.ts, .d.mts, or .d.cts files found matching inputs: []
exit=0

$ vize check --tsconfig tsconfig.app.json
  error:2:7 [TS2322] Type 'string' is not assignable to type 'number'. (source: const count: number = "not a number";)
✗ Type checked 1 files in 2.47s
  1 error(s)
```

## Expected

- `vp run typecheck` checks the files of `tsconfig.app.json` (resolved against the project root, like `ignores` / `entries` since #7307) and fails with the TS2322.
- Independently of the path bug: when the configured tsconfig cannot be found, or matches no files, `vize check` should fail (or at least warn loudly) instead of exiting 0.

## Why it matters

The type check task is what CI runs. After upgrading from 0.429.1 the job stays green while checking nothing, so type errors land unnoticed. In a ~1,700-file project the task went from checking every file to "No … files found" with exit 0, while `vize check` on the same tsconfig reports 11 errors.

## Environment

- vize 0.432.0, @vizejs/vite-plugin 0.432.0, vite-plus 0.2.7
- node 26.8.1, macOS arm64
