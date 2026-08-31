import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { DEFAULT_TASKS } from "../../tools/benchmarks/scripts/compare-tools.mjs";
import {
  DEFAULT_MUSEA_FILE_COUNT,
  buildMuseaSurface,
} from "../../tools/benchmarks/scripts/compare-tools-musea.mjs";
import { MUSEA_CORPUS_FILE_COUNT } from "../../tools/benchmarks/scripts/musea-corpus.mjs";
import { assertMuseaArtifactsUnchanged } from "../../tools/benchmarks/scripts/musea-artifacts.mjs";
import { resolveMuseaArtifacts } from "../../tools/benchmarks/scripts/musea-stages.mjs";
import { testAndBenchmarkTasks } from "../../tools/config/vite-plus/tasks/test-benchmark.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const dispatcherPath = path.join(root, "tools/commands/benchmarks/dispatch.rs");

const taskShape = (value: unknown) => value as { cache: boolean; command: string };

function withTempRoot(fn: (root: string) => void): void {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-musea-"));
  try {
    fn(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

function writeCompleteBuild(directory: string): void {
  const pluginDist = path.join(directory, "npm", "builder", "vite-musea", "dist");
  const nuxtDist = path.join(directory, "npm", "framework", "musea-nuxt", "dist");
  const nativeDir = path.join(directory, "npm", "native");
  fs.mkdirSync(path.join(pluginDist, "chunks"), { recursive: true });
  fs.mkdirSync(nuxtDist, { recursive: true });
  fs.mkdirSync(nativeDir, { recursive: true });
  fs.writeFileSync(path.join(pluginDist, "index.mjs"), 'import "./chunks/runtime.mjs";\n');
  fs.writeFileSync(path.join(pluginDist, "chunks", "runtime.mjs"), "export const runtime = 1;\n");
  fs.writeFileSync(path.join(nuxtDist, "index.mjs"), "export const nuxt = 1;\n");
  fs.writeFileSync(path.join(nativeDir, "vize-vitrine.fixture.node"), "native-test-fixture");
  fs.writeFileSync(
    path.join(nativeDir, "index.js"),
    'module.exports = require("./native-binding");\n',
  );
  fs.writeFileSync(path.join(nativeDir, "native-binding.js"), "module.exports = {};\n");
  fs.writeFileSync(
    path.join(nativeDir, "native-targets.js"),
    'module.exports = { nativeTargets: () => ["fixture"] };\n',
  );
  fs.writeFileSync(path.join(nativeDir, "package.json"), '{"name":"@vizejs/native"}\n');
}

const runDispatcher = (args: string[], env: NodeJS.ProcessEnv = process.env) =>
  spawnSync("rust-script", [dispatcherPath, ...args], { cwd: root, encoding: "utf8", env });

function withStubbedNode(fn: (env: NodeJS.ProcessEnv, logPath: string) => void): void {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-dispatch-"));
  const bin = path.join(directory, "bin");
  const logPath = path.join(directory, "node-args.txt");
  const node = path.join(bin, "node");
  const script =
    '#!/usr/bin/env bash\nprintf "%s\\n" "$@" > "$VIZE_BENCH_STUB_LOG"\nexit "${VIZE_BENCH_STUB_EXIT:-0}"\n';
  const pathEnv = `${bin}${path.delimiter}${process.env.PATH ?? ""}`;
  fs.mkdirSync(bin);
  fs.writeFileSync(node, script);
  fs.chmodSync(node, 0o755);
  try {
    fn({ ...process.env, PATH: pathEnv, VIZE_BENCH_STUB_LOG: logPath }, logPath);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

test("bench:musea is registered as a no-cache benchmark lane", () => {
  assert.equal(taskShape(testAndBenchmarkTasks["bench:musea"]).cache, false);
});

test("bench:all runs the Musea lane alongside the other benchmark lanes", () => {
  assert.equal(
    taskShape(testAndBenchmarkTasks["bench:all"]).command,
    ["bench", "bench:lint", "bench:fmt", "bench:check", "bench:vite", "bench:musea"]
      .map((task) => `vp run --workspace-root ${task}`)
      .join(" && "),
  );
});

test("the Rust Script bench dispatcher selects scripts and forwards args", () => {
  withStubbedNode((env, logPath) => {
    const result = runDispatcher(["musea", "--runs", "3", "--label", "gallery"], env);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(
      fs.readFileSync(logPath, "utf8"),
      "tools/benchmarks/scripts/musea.mjs\n--runs\n3\n--label\ngallery\n",
    );

    const failed = runDispatcher(["vite"], { ...env, VIZE_BENCH_STUB_EXIT: "17" });
    assert.equal(failed.status, 17);
  });

  const usage = runDispatcher([]);
  assert.equal(usage.status, 1);
  assert.match(usage.stderr, /Usage: rust-script tools\/commands\/benchmarks\/dispatch\.rs/);
  assert.match(usage.stderr, /<run\|generate\|lint\|fmt\|check\|vite\|musea\|compare-tools>/);
});

test("the tool comparison runs the Musea surface by default", () => {
  assert.deepEqual(DEFAULT_TASKS, [
    "compile",
    "large",
    "lint",
    "fmt",
    "check",
    "vite",
    "nuxt",
    "musea",
  ]);
  assert.equal(DEFAULT_MUSEA_FILE_COUNT, MUSEA_CORPUS_FILE_COUNT);
});

test("the Musea surface publishes the exact six-stage schema without a ratio", () => {
  // The repo has one drift gate: benchmark.yml compares a lane against the PR
  // base and, since #3586, against a pinned historical commit on a schedule.
  // A published lane that invented a ratio or carried its own stored baseline
  // would be a second, unreconciled gate. This surface names no baseline
  // variant, so `createSurface` reports the refusal instead of a number.
  const surface = buildMuseaSurface({
    fileCount: 240,
    variantCount: 720,
    bytes: 1_000,
    artifacts: {
      museaPlugin: { sha256: "plugin-hash", pinned: true, source: "/source/plugin" },
      native: { sha256: "native-hash", pinned: true, source: "/source/native" },
    },
    stages: [
      ["musea-options", "options: preserve configured Rollup inputs", 1, "build hooks", 0.1],
      ["musea-build-start", "buildStart: scan + parse art files", 240, "art files", 400],
      ["musea-load", "load: generate art modules", 240, "art files", 80],
      ["musea-transform", "transform: TS to JS on generated modules", 240, "art files", 60],
      ["musea-nuxt-virtual", "musea-nuxt: resolve Nuxt mock specifiers", 2160, "resolutions", 0.25],
      [
        "musea-plugin-total",
        "whole plugin: config + options + buildStart + load + transform",
        240,
        "art files",
        800,
      ],
    ].map(([id, label, units, unitLabel, medianMs], index) => ({
      id,
      label,
      units,
      unitLabel,
      medianMs,
      msPerUnit: Number((Number(medianMs) / Number(units)).toFixed(6)),
      runs: [medianMs, medianMs, medianMs],
      digest: `digest-${index}`,
    })),
  });

  assert.equal(surface.id, "musea");
  assert.equal(surface.label, "Musea plugin hooks (art gallery build)");
  assert.equal(surface.baselineId, null);
  assert.equal(surface.vizeSingleId, null);
  assert.equal(surface.vizeMaxId, "musea-plugin-total");
  assert.equal(surface.primarySpeedup, null);
  assert.equal(surface.speedupStatus, "unavailable");
  assert.equal(surface.engineClassRanking, null);
  assert.equal(surface.files, 240);
  assert.equal(surface.variantCount, 720);
  assert.deepEqual(surface.artifacts, {
    museaPlugin: { sha256: "plugin-hash", pinned: true },
    native: { sha256: "native-hash", pinned: true },
  });
  assert.deepEqual(
    surface.variants.map((variant) => ({
      id: variant.id,
      label: variant.label,
      units: variant.units,
      unitLabel: variant.unitLabel,
      medianMs: variant.medianMs,
      msPerUnit: variant.msPerUnit,
      runs: variant.runs,
      digest: variant.digest,
      throughput: variant.throughput,
    })),
    [
      {
        id: "musea-options",
        label: "options: preserve configured Rollup inputs",
        units: 1,
        unitLabel: "build hooks",
        medianMs: 0.1,
        msPerUnit: 0.1,
        runs: [0.1, 0.1, 0.1],
        digest: "digest-0",
        throughput: "10.0k build hooks/s",
      },
      {
        id: "musea-build-start",
        label: "buildStart: scan + parse art files",
        units: 240,
        unitLabel: "art files",
        medianMs: 400,
        msPerUnit: 1.666667,
        runs: [400, 400, 400],
        digest: "digest-1",
        throughput: "600 art files/s",
      },
      {
        id: "musea-load",
        label: "load: generate art modules",
        units: 240,
        unitLabel: "art files",
        medianMs: 80,
        msPerUnit: 0.333333,
        runs: [80, 80, 80],
        digest: "digest-2",
        throughput: "3.0k art files/s",
      },
      {
        id: "musea-transform",
        label: "transform: TS to JS on generated modules",
        units: 240,
        unitLabel: "art files",
        medianMs: 60,
        msPerUnit: 0.25,
        runs: [60, 60, 60],
        digest: "digest-3",
        throughput: "4.0k art files/s",
      },
      {
        id: "musea-nuxt-virtual",
        label: "musea-nuxt: resolve Nuxt mock specifiers",
        units: 2160,
        unitLabel: "resolutions",
        medianMs: 0.25,
        msPerUnit: 0.000116,
        runs: [0.25, 0.25, 0.25],
        digest: "digest-4",
        throughput: "8.6M resolutions/s",
      },
      {
        id: "musea-plugin-total",
        label: "whole plugin: config + options + buildStart + load + transform",
        units: 240,
        unitLabel: "art files",
        medianMs: 800,
        msPerUnit: 3.333333,
        runs: [800, 800, 800],
        digest: "digest-5",
        throughput: "300 art files/s",
      },
    ],
  );
});

test("a missing native binding names the task that produces it", () => {
  // Without this check the run dies inside the plugin with napi-rs's generic
  // "Cannot find native binding" text, which blames npm's optional-dependency
  // bug instead of naming the build task the lane actually needs.
  withTempRoot((directory) => {
    for (const relative of [
      ["npm", "builder", "vite-musea", "dist"],
      ["npm", "framework", "musea-nuxt", "dist"],
    ]) {
      fs.mkdirSync(path.join(directory, ...relative), { recursive: true });
      fs.writeFileSync(path.join(directory, ...relative, "index.mjs"), "export {};\n");
    }

    assert.throws(
      () => resolveMuseaArtifacts(directory),
      (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.equal(
          error.message,
          `@vizejs/native must have exactly one local binding in ${path.join(directory, "npm", "native")}, found 0. Run vp run --workspace-root build:native:test first.`,
        );
        return true;
      },
    );
  });
});

test("a missing plugin build names the task that produces it", () => {
  // Asserted against a temporary root, never the checkout: this suite runs in a
  // clean CI checkout where no package has been built, and a test that read the
  // real dist directory would pass or fail on build state instead of wiring.
  withTempRoot((directory) => {
    assert.throws(
      () => resolveMuseaArtifacts(directory),
      (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.equal(
          error.message,
          `@vizejs/vite-plugin-musea build not found: ${path.join(directory, "npm", "builder", "vite-musea", "dist", "index.mjs")}. Run vp run --workspace-root build:nuxt-stack first.`,
        );
        return true;
      },
    );
  });
});

test("the lane pins and hashes imported dist chunks, not just package entries", () => {
  withTempRoot((directory) => {
    writeCompleteBuild(directory);
    const artifacts = resolveMuseaArtifacts(directory);
    const chunk = artifacts["museaPlugin:chunks/runtime.mjs"];

    assert.ok(chunk, "the imported runtime chunk must be part of the artifact manifest");
    assert.notEqual(chunk.measuredPath, chunk.source);
    assert.equal(
      fs.readFileSync(chunk.measuredPath, "utf8"),
      fs.readFileSync(chunk.source, "utf8"),
    );
    assert.equal(artifacts.native.pinned, true);
    assert.equal(artifacts["nativeLoader:index.js"].pinned, true);
    assert.match(
      artifacts.native.measuredPath,
      /vite-musea[/\\]node_modules[/\\]\.cache[/\\]vize-musea-benchmark[/\\][a-f0-9]{64}[/\\]package[/\\]node_modules[/\\]@vizejs[/\\]native[/\\]vize-vitrine\.fixture\.node$/,
    );
    assert.doesNotThrow(() => assertMuseaArtifactsUnchanged(artifacts));
  });
});

test("the lane refuses source or measured-copy mutation of an imported chunk", () => {
  withTempRoot((directory) => {
    writeCompleteBuild(directory);
    const sourceArtifacts = resolveMuseaArtifacts(directory);
    const sourceChunk = sourceArtifacts["museaPlugin:chunks/runtime.mjs"];
    fs.writeFileSync(sourceChunk.source, "export const runtime = 2;\n");
    assert.throws(
      () => assertMuseaArtifactsUnchanged(sourceArtifacts),
      /museaPlugin:chunks\/runtime\.mjs source changed during the run/,
    );

    writeCompleteBuild(directory);
    const measuredArtifacts = resolveMuseaArtifacts(directory);
    const measuredChunk = measuredArtifacts["museaPlugin:chunks/runtime.mjs"];
    fs.writeFileSync(measuredChunk.measuredPath, "export const runtime = 3;\n");
    assert.throws(
      () => assertMuseaArtifactsUnchanged(measuredArtifacts),
      /museaPlugin:chunks\/runtime\.mjs measured copy changed during the run/,
    );
  });
});
