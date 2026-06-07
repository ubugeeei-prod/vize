---
title: Testing & Feedback
---

# Testing & Feedback

Vize is in its **Real World Testing** phase: the focus is correctness, and real-world projects are
the test suite. This page is for testers — how to inspect what Vize does, where to look, how to
report findings, how to measure performance, and how to offer your project as a test bed.

## Inspect with the Playground

The playground ships an **inspector** that shows, side by side, the official Vue SFC compiler
output, Vize's compiler output, the generated Virtual TS, the VIR, and a cross-file graph for local
batches. It is the fastest way to see exactly where Vize agrees or disagrees with Vue for a given
`.vue` file.

- Open it at <https://vizejs.dev/play/?tab=inspector>.
- See the [Compiler Inspector](./compiler-inspector.md) guide for what each surface means.

A playground inspector link makes an excellent bug reproduction.

## Read the Test Cases

Vize is tested heavily and in many different ways — compiler fixtures compared against the official
Vue compiler, type-check parity against `vue-tsc`, lint and formatter snapshots, SSR codegen
snapshots, fuzz harnesses, and real-world application fixtures. Before filing a report, it often
helps to skim the existing cases:

- Compiler and SFC parity fixtures and snapshots under `tests/` and each crate's `src/snapshots/`.
- Real-world application fixtures under `tests/_fixtures/` (for example Elk, Misskey, Nuxt UI,
  Reka UI, and VOICEVOX) that drive E2E and VRT.

If a case is missing or a result looks wrong, that is exactly the kind of feedback this phase wants.

## Report Findings

- **Plain text is fine.** A clear description of what you did, what you expected, and what happened
  is already valuable.
- **If you can, attach a minimal reproduction** to a GitHub Issue — the smallest `.vue` file (or
  small project) that still shows the problem. A playground inspector link works great.
- Bug reports, reproductions, benchmark results, and compatibility findings all help. See the
  [Contributing](../contributing.md) guide and
  [Support](https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md).

## Measure Performance

Vize has a built-in **profiling mode**, so you can measure where time goes instead of guessing.

- In local development, the toolchain exposes profiling across the parser, compiler, analysis, and
  type-check phases.
- The CLI has it too: `vize check --profile` runs the check through **vize_curator** and prints a
  per-phase profiling report. Use it to capture and share performance numbers from your own
  codebase.

## Offer Your Project as a Test Bed

Real, sizable codebases find the bugs that synthetic examples never will. **Where the license
allows it, a project can be added to Vize's fixtures and become an E2E / VRT target**, so future
regressions are caught automatically.

If you maintain (or know of) a Vue application, library, framework, or tool that may be used this
way, please let us know — open an issue or reach out. The bigger and more real the codebase, the
more useful the signal.
