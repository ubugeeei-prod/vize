# Oxlint + Patina Example

This is the smallest runnable example of Vize Patina executed through Oxlint's JS Plugin system.

## Prerequisites

This example uses the locally built Patina plugin bundle and `@vizejs/native`. The `pnpm lint` scripts rebuild them automatically. If you want to build them up front from the repository root:

```bash
pnpm install
pnpm --filter @vizejs/native build
pnpm --filter @vizejs/oxlint-plugin-patina build
```

## Run

```bash
pnpm -C examples/oxlint-patina lint
```

This command intentionally exits non-zero because it includes `src/HasPatinaErrors.vue`.
It mixes:

- Oxlint core diagnostics from `no-console`
- Patina diagnostics from the JS plugin bridge

If you want to check `no-unused-vars` specifically:

```bash
pnpm -C examples/oxlint-patina lint:unused-vars-probe
```

Current observed behavior in this repository: that command reports `0` findings on `.vue` even though `src/UnusedVarProbe.vue` contains an unused binding. I verified separately that the same `no-unused-vars` rule does fire on a plain `.ts` file, so this is an Oxlint/Vue behavior to be aware of rather than a Patina bridge conflict.

If you only want the success path:

```bash
pnpm -C examples/oxlint-patina lint:clean
```

If you want JSON output from the same command:

```bash
pnpm -C examples/oxlint-patina lint:json
```

If you want the raw direct CLI path, that also works once the plugin has been built:

```bash
cd examples/oxlint-patina
pnpm exec oxlint -c .oxlintrc.json src
```

## Notes

- Oxlint's built-in `vue` plugin is enabled through `"plugins": ["vue"]`.
- Oxlint's built-in `no-console` rule is enabled so the example shows native Oxlint output mixed with Patina output in one run.
- A dedicated `lint:unused-vars-probe` command is included because `no-unused-vars` currently does not emit on the example `.vue` SFC, even without the Patina plugin.
- The Patina JS plugin is loaded from `../../npm/oxlint-plugin-patina/dist/index.js`.
- The plugin starts with a single-rule native Patina run and only upgrades to a shared full-file pass when multiple Patina rules are enabled for the same file.
- Patina help text is normalized to plain text so terminal output stays readable even without Markdown rendering.
- Because of the current Oxlint JS Plugin limitation, the example `.vue` files include `<script setup>`.
- `src/HasPatinaErrors.vue` is the failing mixed-output sample, `src/Clean.vue` is the clean sample, and `src/UnusedVarProbe.vue` is the `no-unused-vars` probe.
