---
title: Testing & Feedback
---

# Testing & Feedback

Use this guide when a Vue project produces an unexpected compiler result, a type error or lint
warning on correct code, or a slowdown. A small reproduction and the command you ran let us
investigate the same behavior.

| You want to look into                     | Start here                                                         |
| ----------------------------------------- | ------------------------------------------------------------------ |
| A template or generated-code difference   | [Compare output in the Playground](#inspect-with-the-playground)   |
| A type error, lint, or editor problem     | [Write down the steps and diagnostic](#report-findings)            |
| Slow type checking                        | [Measure where the time goes](#measure-performance)                |
| Continuous testing on your own project    | [Offer your project as a test bed](#offer-your-project-as-a-test-bed) |

## Inspect with the Playground

Open the [Playground compiler inspector](https://vizejs.dev/play/?tab=inspector) and paste the
`.vue` file that shows the problem. Compare the official Vue SFC compiler output with Vize's output
while you remove code that is not related to the problem. After each removal, check that the
problem is still there.

For a state-update problem, for example, a component that keeps only the state and the event is a
good starting point. Keep the expression or directive that triggers the problem, and describe the
difference you see:

```vue
<script setup lang="ts">
import { ref } from "vue";

const count = ref(0);
</script>

<template>
  <button type="button" @click="count++">{{ count }}</button>
</template>
```

Keep every file a parent/child problem needs. Once the reproduction is small, copy the permalink;
it shares the same input and settings. The [Compiler Inspector guide](./compiler-inspector.md)
covers DOM/SSR/Vapor targets, reading Virtual TS and VIR, and loading files from a local project.

## Report Findings

The [Support guide](https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md) says where to report.
Copy this template into your report. If you do not know an item, say so:

```text
Vize / Vue version:
OS, Node.js, and package manager versions:
Reproduction URL or inspector permalink:
Command you ran, or the editor action:
Expected result:
Actual result (including the diagnostic or output):
```

For a type error or lint problem, include the full diagnostic and the code it points at. If the
problem only happens in an editor, add the editor and extension versions and the steps from opening
the file to seeing the problem. When several files are needed, prepare a small project where
installing dependencies and running the same command reproduces it.

For a speed problem, add the measurements described below. The
[Contributing guide](../contributing.md) explains the development setup and pull request steps if
you want to fix it yourself.

## Read the Test Cases

Existing cases are useful when you want to compare a problem with a similar example or contribute
a regression test. Compiler fixtures compare output against the official Vue compiler, type-check
cases compare diagnostics against `vue-tsc`, and lint, formatter, and SSR snapshots record expected
results. Fuzz tests and real-world application fixtures cover additional inputs and workflows.

- Compiler and SFC parity fixtures and snapshots under `tests/` and each crate's `src/snapshots/`.
- Real-world application fixtures under `tests/_fixtures/` (for example Elk, Misskey, Nuxt UI,
  Reka UI, and VOICEVOX) that drive end-to-end and visual regression tests.

If you find a missing case, include the example in your report. It can become a regression test
that protects the fix.

## Measure Performance

When type checking is slow, run this in the affected project:

```bash
vize check --profile
```

Share the command and its complete output, together with the Vize version, the size of the project,
and the observed run time. Note whether only the first run is slow or every run is.

For a repeatable comparison, follow the [type-check performance guide](./check-performance.md).
Local development tools also expose profiling for parsing, compilation, analysis, and type checks.

## Offer Your Project as a Test Bed

If you maintain a Vue application, library, framework, or tool, you can propose it as a test
fixture. Include a link to the project, its license, the commands to install and run it, the
workflow Vize should check, and the result you expect.

With permission and a suitable license, the project can become an end-to-end or visual regression
test. Running it again after changes helps catch the same problem before it reaches other users.

Vize runs both `check` and `lint` against the pinned source of [PrimeVue](https://github.com/primefaces/primevue)
and [OpenVue](https://github.com/openvi-foundation/openvue). OpenVue is pinned to its MIT-licensed
`1.0.0` release so every run uses the same component, icon, form, showcase, and Volt sources.
