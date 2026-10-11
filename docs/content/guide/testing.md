---
title: Testing & Feedback
---

# Testing & Feedback

Use this guide when a Vue project produces an unexpected compiler result, type error, lint warning,
or slowdown. A small reproduction and the command you ran help us investigate the same behavior.

Start with the affected `.vue` file. The [Compiler Inspector guide](./compiler-inspector.md) shows
how to open it and compare its output. Describe what you expected and share the reproduction.
You can also contribute performance measurements or offer a project for regression testing.

## Inspect with the Playground

Open the [Playground compiler inspector](https://vizejs.dev/play/?tab=inspector) and use the smallest
`.vue` file that still shows the problem. Compare the official Vue SFC compiler output with Vize's
output, then copy the permalink so another person can open the same example.

The inspector also shows generated Virtual TS, VIR, and a cross-file graph for local batches. See
the [Compiler Inspector guide](./compiler-inspector.md) for these views, DOM/SSR target selection,
and loading files from a local project.

## Report Findings

Describe what you ran, what happened, and what you expected. Plain text is welcome. Include the
Vize and Vue versions, the exact command or editor action, and any relevant error message.

Attach the inspector permalink or a small project that still reproduces the problem. For a
performance report, include the measurement and profiling output described below. The
[Contributing guide](../contributing.md) explains how to contribute a fix, and the
[Support guide](https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md) explains where to ask for
help or report a problem.

## Read the Test Cases

Existing cases are useful when you want to compare a problem with a similar example or contribute
a regression test. Compiler fixtures compare output against the official Vue compiler, type-check
cases compare diagnostics against `vue-tsc`, and lint, formatter, and SSR snapshots record expected
results. Fuzz tests and real-world application fixtures cover additional inputs and workflows.

- Compiler and SFC parity fixtures and snapshots under `tests/` and each crate's `src/snapshots/`.
- Real-world application fixtures under `tests/_fixtures/` (for example Elk, Misskey, Nuxt UI,
  Reka UI, and VOICEVOX) that drive E2E and VRT.

If you find a missing case, include the example in your report. It can become a regression test
that protects the fix.

## Measure Performance

Run `vize check --profile` in the project that feels slow. It prints a report of the time spent in
each phase. Share the complete command and report, the Vize version, and the size of the project
alongside the observed run time.

For a repeatable comparison, follow the [type-check performance measurement guide](./check-performance.md).
Local development tools also expose profiling for parsing, compilation, analysis, and type checks.

## Offer Your Project as a Test Bed

If you maintain a Vue application, library, framework, or tool, you can propose it as a test
fixture. Include a link to the project, its license, and the commands needed to install and run it.
Explain which workflow Vize should check.

With permission and a suitable license, the project can become an end-to-end or visual regression
test. Running it again after changes helps catch the same problem before it reaches other users.
