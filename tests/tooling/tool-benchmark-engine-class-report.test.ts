import assert from "node:assert/strict";
import { test } from "node:test";

import { buildFairnessNotes } from "../../tools/benchmarks/scripts/benchmark-notes.mjs";
import { createSurface } from "../../tools/benchmarks/scripts/compare-tools-report.mjs";
import { renderMarkdown } from "../../tools/benchmarks/scripts/compare-tools.mjs";
import { checkSurfaceInput } from "./support/benchmark/check-surface.ts";

test("the type-check summary ranks engine classes beside the published ratio", () => {
  const surface = createSurface(checkSurfaceInput());
  const markdown = renderMarkdown({
    schemaVersion: 1,
    kind: "tool-comparison",
    generatedAt: "2026-06-01T00:00:00.000Z",
    commit: { sha: "0123456789abcdef", ref: "main", repository: "o/r", runUrl: "" },
    runner: {
      label: "local",
      blacksmithMaxSpec: "",
      cpuCount: 8,
      cpuModel: "test cpu",
      platform: "linux",
      arch: "x64",
      osRelease: "6.0.0",
      node: "v24.0.0",
    },
    versions: {
      vize: "vize 0.303.0",
      tsgo: "7.0.0-dev",
      vueTsc: "3.2.0",
      verterTsc: "verter-tsc 0.0.1-beta.3",
      golar: "golar 0.1.10",
      typescript: "5.9.0",
      vue: "3.6.0",
      eslint: "9.0.0",
      prettier: "3.4.0",
      node: "v24.0.0",
    },
    binaries: {
      vize: "d".repeat(64),
      tsgo: "e".repeat(64),
      vueTsc: null,
      verterTsc: "f".repeat(64),
      golar: "g".repeat(64),
    },
    backend: {
      engine: "tsgo-native",
      corsaPath: "/repo/node_modules/.bin/tsgo",
      corsaVersion: "7.0.0-dev",
      ready: true,
      reason: null,
    },
    input: {
      dir: "/tmp/bench",
      fileCount: 500,
      totalBytes: 1_000_000,
      checkFileCount: 500,
      viteFileCount: 0,
      nuxtFileCount: 0,
      museaFileCount: 0,
      largeBlocks: 0,
      largeSfcBytes: 0,
    },
    settings: { runs: 1, warmups: 1, tasks: ["check"] },
    commands: { workflowDispatch: "dispatch", generate: "generate", benchmark: "benchmark" },
    fairness: ["only note"],
    surfaces: [surface],
  });

  assert.deepEqual(markdown.split("\n"), [
    "## Tool Benchmark",
    "",
    "Measured: 2026-06-01T00:00:00.000Z",
    "Commit: `0123456789ab`",
    "Runner: `local` (8 logical CPU, test cpu)",
    "Input: 500 generated SFC files (976.6 KB). Median of 1 measured run(s) after 1 warmup run(s).",
    "Versions: vize `vize 0.303.0` · tsgo `7.0.0-dev` · vue-tsc `3.2.0` (typescript `5.9.0`) · verter-tsc `verter-tsc 0.0.1-beta.3` · Golar `golar 0.1.10` · vue `3.6.0` · eslint `9.0.0` · prettier `3.4.0` · node `v24.0.0`",
    `Binaries (sha256): vize \`${"d".repeat(64)}\` tsgo \`${"e".repeat(64)}\` vueTsc n/a verterTsc \`${"f".repeat(64)}\` golar \`${"g".repeat(64)}\``,
    "Backend: native TypeScript engine ready at `/repo/node_modules/.bin/tsgo`. Planted-diagnostic gating for the type-check rows lives in tools/benchmarks/scripts/check-gate.mjs (.github/workflows/check-bench.yml).",
    "",
    "| Surface | Files | Existing tool | Existing median | Vize 1T | Vize max | Speedup |",
    "| --- | ---: | --- | ---: | ---: | ---: | ---: |",
    "| Type check | 500 | vue-tsc | 8.00s | 2.00s | 500.0ms | 16.0x |",
    "",
    "#### Type check — engine classes ranked separately",
    "",
    "| Engine class | Row | Median | Relative to fastest in class |",
    "| --- | --- | ---: | ---: |",
    "| JS TypeScript engine (tsc) | vue-tsc | 8.00s | 1.00x |",
    "| native TypeScript engine (tsgo) | Vize check (max) | 500.0ms | 1.00x |",
    "| native TypeScript engine (tsgo) | verter-tsc | 1.00s | 2.00x |",
    "| native TypeScript engine (tsgo) | Golar typecheck | 1.50s | 3.00x |",
    "| native TypeScript engine (tsgo) | Vize check (1T) | 2.00s | 4.00x |",
    "| native TypeScript engine (tsgo) | Golar (lint+check) | 2.50s | 5.00x |",
    "",
    "The Type check ratio compares Vize with vue-tsc, the checker Vue projects run today. vue-tsc drives the JavaScript TypeScript compiler while Vize drives native tsgo, so that ratio is the whole toolchain and not the Vue layer alone; the per-engine-class rows above isolate the Vue layer by ranking each class against its own fastest row. Diagnostic coverage can differ between tools; no ratio here is an accuracy-parity claim.",
    "",
    "Fairness notes:",
    "- only note",
    "",
    "Commands:",
    "",
    "```sh",
    "dispatch",
    "generate",
    "benchmark",
    "```",
    "",
    "<details>",
    "<summary>Variant details and raw run times</summary>",
    "",
    "### Type check",
    "",
    "| Variant | Median | Throughput | Raw measured runs |",
    "| --- | ---: | ---: | --- |",
    "| vue-tsc | 8.00s | 62.5 files/s | 8.00s |",
    "| verter-tsc | 1.00s | 500.0 files/s | 1.00s |",
    "| Golar typecheck | 1.50s | 333.3 files/s | 1.50s |",
    "| Golar (lint+check) | 2.50s | 200.0 files/s | 2.50s |",
    "| Vize check (1T) | 2.00s | 250.0 files/s | 2.00s |",
    "| Vize check (max) | 500.0ms | 1.0k files/s | 500.0ms |",
    "",
    "</details>",
    "",
    "",
  ]);
});

test("the fairness notes name the incumbent the published ratio is measured against", () => {
  assert.deepEqual(
    buildFairnessNotes(500).filter((note) => note.startsWith("Type-check rows")),
    [
      "Type-check rows publish their speedup against vue-tsc, the type checker Vue projects actually run. That ratio spans two TypeScript engines — vue-tsc runs the JavaScript compiler while Vize check runs native tsgo (Corsa) — so it measures the whole toolchain a reader would replace, not the Vue layer alone, and part of it is TypeScript's Go rewrite rather than anything Vize does. The per-engine-class ranking published beside it isolates the Vue layer by rating each row against the fastest row of its own engine, which is where the same-engine native checkers (verter-tsc, Golar) appear; they are reference rows and never the headline comparison, because almost nobody runs them. tools/benchmarks/scripts/check-gate.mjs publishes the same per-engine-class split with planted-diagnostic gating.",
    ],
  );
});
