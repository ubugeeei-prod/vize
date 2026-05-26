---
title: Compiler Inspector
---

# Compiler Inspector

The playground inspector compares the official Vue SFC compiler output with Vize's compiler output
for the same `.vue` source. Use it when a report, fixture, or pull request needs a concrete parity
artifact instead of a prose description of the mismatch.

Open the inspector from the playground:

```bash
https://vizejs.dev/play/?tab=inspector
```

The inspector runs both compilers in the browser:

- `@vue/compiler-sfc` for the reference output
- Vize WASM for the Vize output
- DOM or SSR target selection
- Optional custom renderer and Vue parser quirk toggles
- Full output tabs for both compilers
- A unified diff tab with Vue-only and Vize-only lines
- A permalink and prefilled pull request link

## CLI Payloads

Use `vize inspector` when the repro already exists in a local project. A single file produces a
playground URL by default:

```bash
vize inspector src/App.vue
```

Directories and globs create batch payloads. The playground opens the batch and lets you switch
between files.

```bash
vize inspector src/components
vize inspector "src/**/*.vue" --target ssr
```

For large batches, emit JSON instead of a long URL:

```bash
vize inspector "src/**/*.vue" --format json --output inspector-payload.json
```

Useful options:

| Option                | Description                                    |
| --------------------- | ---------------------------------------------- |
| `--target dom`        | Compare DOM compiler output                    |
| `--target ssr`        | Compare SSR compiler output                    |
| `--custom-renderer`   | Enable custom renderer mode in the playground  |
| `--vue-parser-quirks` | Enable Vue parser compatibility quirks in Vize |
| `--max-files <n>`     | Limit the number of files in a batch payload   |
| `--playground-url`    | Override the playground URL used for links     |

## PR Workflow

When opening a compiler parity PR, include the inspector permalink in the PR body and add the
minimal fixture or snapshot that makes the diff reviewable in CI. The prefilled PR link is a
starting point; after pushing your branch, replace the compare head if GitHub asks for it.

The useful PR evidence is:

- The inspector permalink
- The selected target and options
- The minimized `.vue` fixture or snapshot diff
- The reason the Vize output should match or intentionally differ from Vue
- The local verification command that covers the touched compiler surface
