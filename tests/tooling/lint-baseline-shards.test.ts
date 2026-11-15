import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  BaselineShardAccumulator,
  DEFAULT_BASELINE_SHARD_LIMITS,
  baselineAuthoritySha256,
  mergeBaselineShardOutputs,
  partitionBaselineFiles,
  type BaselineShardLimits,
} from "../../tools/support/fixtures/lint_divergence/baseline_shards.ts";
import {
  BaselineArtifactStage,
  removeStagedBaselineArtifacts,
} from "../../tools/support/fixtures/lint_divergence/baseline_stage.ts";
import { runEslintBaseline } from "../../tools/support/fixtures/lint_divergence/runtime.ts";
import { canonicalJson, type Json } from "../../tools/support/fixtures/lint_divergence/value.ts";

const root = path.resolve(import.meta.dirname, "../..");
const limits: BaselineShardLimits = {
  maxFileCount: 2,
  maxOutputBytes: 8 * 1024 * 1024,
  maxSourceBytes: 32,
};

test("the production baseline has an explicit bounded memory envelope", () => {
  assert.deepEqual(DEFAULT_BASELINE_SHARD_LIMITS, {
    maxFileCount: 32,
    maxOutputBytes: 256 * 1024 * 1024,
    maxSourceBytes: 8 * 1024 * 1024,
  });
});

function withCorpus<T>(sources: Record<string, string>, callback: (cwd: string) => T): T {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lint-baseline-shards-"));
  try {
    for (const [filename, source] of Object.entries(sources))
      fs.writeFileSync(path.join(cwd, filename), source);
    return callback(cwd);
  } finally {
    fs.rmSync(cwd, { recursive: true, force: true });
  }
}

function packet(cwd: string, files: string[], version = "10.9.2"): Json {
  const results = files.map((file) => ({
    filePath: path.join(cwd, file),
    messages: [{ ruleId: "vue/no-v-html", line: 1, column: 1 }],
  }));
  return {
    version,
    runtimeVersions: { eslint: "9.39.2", "eslint-plugin-vue": version },
    rawResults: results,
    results,
    droppedConfigMessageCount: 0,
  };
}

test("file and authored-byte envelopes deterministically partition a multi-shard corpus", () => {
  withCorpus(
    {
      "a.vue": "a".repeat(20),
      "b.vue": "b".repeat(12),
      "c.vue": "c".repeat(20),
      "d.vue": "d".repeat(4),
      "e.vue": "e".repeat(4),
    },
    (cwd) => {
      const files = ["a.vue", "b.vue", "c.vue", "d.vue", "e.vue"];
      assert.deepEqual(partitionBaselineFiles(cwd, files, limits), [
        { files: ["a.vue", "b.vue"], sourceBytes: 32 },
        { files: ["c.vue", "d.vue"], sourceBytes: 24 },
        { files: ["e.vue"], sourceBytes: 4 },
      ]);
      assert.throws(
        () => partitionBaselineFiles(cwd, ["b.vue", "a.vue"], limits),
        /unique and byte-ordered/u,
      );
      assert.throws(
        () => partitionBaselineFiles(cwd, ["a.vue", "a.vue"], limits),
        /unique and byte-ordered/u,
      );
    },
  );
});

test("one oversized authored file fails closed before ESLint starts", () => {
  withCorpus({ "large.vue": "x".repeat(33) }, (cwd) => {
    assert.throws(
      () => partitionBaselineFiles(cwd, ["large.vue"], limits),
      /above the 32-byte shard envelope/u,
    );
  });
});

test("aggregation preserves complete packets and exact global order", () => {
  const cwd = path.join(root, "virtual-baseline-corpus"),
    files = ["a.vue", "b.vue", "c.vue"],
    shards = [
      { files: files.slice(0, 2), output: packet(cwd, files.slice(0, 2)) },
      { files: files.slice(2), output: packet(cwd, files.slice(2)) },
    ],
    merged = mergeBaselineShardOutputs(cwd, files, shards);
  assert.deepEqual(
    merged.rawResults.map((result) => (result as { filePath: string }).filePath),
    files.map((file) => path.join(cwd, file)),
  );
  assert.equal(merged.rawResults.length, files.length);
  assert.equal(merged.results.length, files.length);
  assert.equal(merged.rawPacketCount, files.length);
  assert.equal(merged.retainedPacketCount, files.length);
  assert.match(merged.rawPacketSha256, /^[0-9a-f]{64}$/u);
  assert.equal(merged.rawPacketSha256, merged.retainedPacketSha256);
  assert.equal(merged.version, "10.9.2");
  assert.deepEqual(merged.runtimeVersions, {
    eslint: "9.39.2",
    "eslint-plugin-vue": "10.9.2",
  });
});

test("aggregation refuses missing, duplicate, reordered and runtime-drifted packets", () => {
  const cwd = path.join(root, "virtual-baseline-corpus"),
    files = ["a.vue", "b.vue", "c.vue"];
  assert.throws(
    () =>
      mergeBaselineShardOutputs(cwd, files, [
        { files: files.slice(0, 2), output: packet(cwd, files.slice(0, 2)) },
      ]),
    /coverage reported 2 files, expected 3/u,
  );
  assert.throws(
    () =>
      mergeBaselineShardOutputs(cwd, files, [
        { files: files.slice(0, 2), output: packet(cwd, files.slice(0, 2)) },
        { files: ["b.vue"], output: packet(cwd, ["b.vue"]) },
      ]),
    /input coverage/u,
  );
  assert.throws(
    () =>
      mergeBaselineShardOutputs(cwd, files, [
        { files: ["a.vue", "b.vue"], output: packet(cwd, ["b.vue", "a.vue"]) },
      ]),
    /rawResults file 0/u,
  );
  assert.throws(
    () =>
      mergeBaselineShardOutputs(cwd, files, [
        { files: files.slice(0, 2), output: packet(cwd, files.slice(0, 2)) },
        { files: files.slice(2), output: packet(cwd, files.slice(2), "10.9.3") },
      ]),
    /runtime identity differs/u,
  );
});

test("a failed aggregate cannot be finished into partial public evidence", () => {
  const cwd = path.join(root, "virtual-baseline-corpus"),
    accumulator = new BaselineShardAccumulator(cwd, ["a.vue", "b.vue"]);
  accumulator.add({ files: ["a.vue"], output: packet(cwd, ["a.vue"]) });
  assert.throws(() => accumulator.finish(), /coverage reported 1 files, expected 2/u);
});

test("file and rule authority is content-addressed", () => {
  withCorpus({ "a.vue": "<template />\n", "b.vue": "<script setup />\n" }, (cwd) => {
    const files = ["a.vue", "b.vue"],
      rules = { "vue/no-v-html": "warn" },
      original = baselineAuthoritySha256(cwd, files, rules);
    assert.equal(original, baselineAuthoritySha256(cwd, files, rules));
    assert.notEqual(original, baselineAuthoritySha256(cwd, files, { ...rules, other: "warn" }));
    fs.appendFileSync(path.join(cwd, "b.vue"), "<!-- changed -->\n");
    assert.notEqual(original, baselineAuthoritySha256(cwd, files, rules));
  });
});

test("private staging materializes the exact canonical artifact atomically", () => {
  withCorpus({}, (outputDir) => {
    const filename = path.join(outputDir, "project-lint-divergence.json"),
      rawResults: Json[] = [
        { filePath: "/fixture/a.vue", messages: [], source: "<template />\n" },
        {
          filePath: "/fixture/b.vue",
          messages: [{ ruleId: "vue/no-v-html", message: "Use textContent" }],
          source: '<div v-html="x" />\n',
        },
      ],
      artifact = {
        schema: "vize.fixtureLintDivergenceRun",
        version: 2,
        project: "stage-test",
        raw: { eslintResults: rawResults, patinaFindings: [], patinaResults: [] },
      };
    const stage = new BaselineArtifactStage(outputDir);
    try {
      stage.append(rawResults.slice(0, 1));
      stage.append(rawResults.slice(1));
      stage.writeArtifact(filename, {
        ...artifact,
        raw: { eslintResults: [], patinaFindings: [], patinaResults: [] },
      });
    } finally {
      stage.cleanup();
    }
    assert.equal(fs.readFileSync(filename, "utf8"), `${canonicalJson(artifact, true)}\n`);
    assert.equal(
      fs.readdirSync(outputDir).some((entry) => entry.startsWith(".lint-divergence-stage-")),
      false,
    );
  });
});

test("stale private stages are removed without touching public reports", () => {
  withCorpus({}, (outputDir) => {
    const publicReport = path.join(outputDir, "project-lint-divergence.json"),
      stale = fs.mkdtempSync(path.join(outputDir, ".lint-divergence-stage-"));
    fs.writeFileSync(publicReport, "public\n");
    fs.writeFileSync(path.join(stale, "partial"), "private\n");
    removeStagedBaselineArtifacts(outputDir);
    assert.equal(fs.readFileSync(publicReport, "utf8"), "public\n");
    assert.equal(fs.existsSync(stale), false);
  });
});

test("bounded and unbounded ESLint packets are byte-identical", () => {
  withCorpus(
    Object.fromEntries(
      Array.from({ length: 5 }, (_, index) => [
        `${index}.vue`,
        `<template><div v-html="value${index}" /></template>\n`,
      ]),
    ),
    (cwd) => {
      const files = ["0.vue", "1.vue", "2.vue", "3.vue", "4.vue"],
        rules = { "vue/no-v-html": "warn" } as const,
        unbounded = runEslintBaseline(root, cwd, files, rules, null),
        bounded = runEslintBaseline(root, cwd, files, rules, {
          maxFileCount: 2,
          maxOutputBytes: 8 * 1024 * 1024,
          maxSourceBytes: 1024,
        });
      assert.equal(
        canonicalJson(bounded as unknown as Json),
        canonicalJson(unbounded as unknown as Json),
      );
    },
  );
});
