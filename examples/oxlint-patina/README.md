# Oxlint + Patina Example

This is the smallest runnable example of Vize Patina executed through Oxlint's JS Plugin system.

## Prerequisites

This example uses the local built plugin bundle and `@vizejs/native`. `pnpm lint` / `pnpm lint:json` automatically build missing artifacts, but if you want to build them up front from the repository root:

```bash
pnpm install
pnpm -C npm/vize-native build
pnpm -C npm/oxlint-plugin-patina build
```

## Run

```bash
pnpm -C examples/oxlint-patina lint
```

This command intentionally exits non-zero because it includes `src/HasPatinaErrors.vue`.
It mixes:

- Oxlint core diagnostics from `no-console`
- Patina diagnostics from the JS plugin bridge

`pnpm lint` uses a small JSON-based formatter so the terminal output shows meaningful Vue source snippets instead of Oxlint's misleading fallback script frame for unmappable template diagnostics.

If you want to check `no-unused-vars` specifically:

```bash
pnpm -C examples/oxlint-patina lint:unused-vars-probe
```

Current observed behavior in this repository: that command reports `0` findings on `.vue` even though `src/UnusedVarProbe.vue` contains an unused binding. I verified separately that the same `no-unused-vars` rule does fire on a plain `.ts` file, so this is an Oxlint/Vue behavior to be aware of rather than a Patina bridge conflict.

If you only want the success path:

```bash
pnpm -C examples/oxlint-patina lint:clean
```

If you want JSON output:

```bash
pnpm -C examples/oxlint-patina lint:json
```

If you want the raw Oxlint text formatter for comparison:

```bash
pnpm -C examples/oxlint-patina lint:raw
```

## Notes

- Oxlint's built-in `vue` plugin is enabled through `"plugins": ["vue"]`.
- Oxlint's built-in `no-console` rule is enabled so the example shows native Oxlint output mixed with Patina output in one run.
- A dedicated `lint:unused-vars-probe` command is included because `no-unused-vars` currently does not emit on the example `.vue` SFC, even without the Patina plugin.
- The Patina JS plugin is loaded directly from `../../npm/oxlint-plugin-patina/dist/index.js`.
- `pnpm lint` renders from Oxlint JSON to avoid the confusing fallback code frame that Oxlint JS plugins produce for Vue template/style diagnostics outside the extracted script range.
- The plugin starts with a single-rule native Patina run and only upgrades to a shared full-file pass when multiple Patina rules are enabled for the same file.
- Patina help text is normalized to plain text so terminal output stays readable even without Markdown rendering.
- The helper script only builds dependencies when the native binary or plugin bundle is missing, so repeat example runs stay fast.
- Because of the current Oxlint JS Plugin limitation, the example `.vue` files include `<script setup>`.
- `src/HasPatinaErrors.vue` is the failing mixed-output sample, `src/Clean.vue` is the clean sample, and `src/UnusedVarProbe.vue` is the `no-unused-vars` probe.
