import assert from "node:assert/strict";
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { createTypecheckToolVariants } from "../../tools/benchmarks/scripts/compare-tools-typecheck.mjs";
import { runVizeCheck } from "../../tools/benchmarks/scripts/check-gate.mjs";
import { runCommand as runPrCommand } from "../../tools/benchmarks/scripts/compare-pr.ts";
import { runCheck as runWorkspaceCheck } from "../../tools/benchmarks/scripts/check-workspace.ts";
import { withEmptyUserCache } from "../../tools/benchmarks/scripts/empty-user-cache.ts";

void test("each call gets its own empty cache, removed afterwards even when the run throws", () => {
  const seen: string[] = [];
  for (let run = 0; run < 2; run++) {
    withEmptyUserCache(({ XDG_CACHE_HOME }) => {
      assert.deepEqual(readdirSync(XDG_CACHE_HOME), []);
      writeFileSync(join(XDG_CACHE_HOME, "tsconfig.tsbuildinfo"), "{}");
      seen.push(XDG_CACHE_HOME);
    });
  }
  assert.notEqual(seen[0], seen[1]);
  assert.ok(seen.every((dir) => !existsSync(dir)));
  let leaked = "";
  assert.throws(
    () =>
      withEmptyUserCache(({ XDG_CACHE_HOME }) => {
        leaked = XDG_CACHE_HOME;
        throw new Error("run failed");
      }),
    /run failed/u,
  );
  assert.ok(leaked && !existsSync(leaked));
});

void test("every published vize check lane starts from an empty cache, so no run replays the last", () => {
  const caches: { dir: string; empty: boolean }[] = [];
  const lanes = createTypecheckToolVariants({
    fileCount: 1,
    vizeBin: "vize",
    corsaPath: "tsgo",
    resolveWorkspaceBin: (name: string) => name,
    cpus: 4,
    runCommand: (_binary: string, _args: string[], options: { env?: Record<string, string> }) => {
      const dir = options.env?.XDG_CACHE_HOME;
      if (dir) {
        caches.push({ dir, empty: readdirSync(dir).length === 0 });
        // What one run leaves behind must not reach the next.
        writeFileSync(join(dir, "tsconfig.tsbuildinfo"), "{}");
      }
      return {};
    },
  }).filter((lane: { tool: string }) => lane.tool === "vize check");
  assert.ok(lanes.length >= 2);
  for (const lane of lanes) {
    lane.run("/project");
    lane.run("/project");
  }
  assert.equal(caches.length, lanes.length * 2);
  assert.equal(new Set(caches.map((cache) => cache.dir)).size, caches.length);
  assert.ok(caches.every((cache) => cache.empty && !existsSync(cache.dir)));
});

void test("the check gate, the PR lanes and the workspace A/B also check from an empty cache", () => {
  const project = mkdtempSync(join(tmpdir(), "vize-cache-handoff-"));
  const parentCache = mkdtempSync(join(tmpdir(), "vize-parent-cache-"));
  const previous = process.env.XDG_CACHE_HOME;
  try {
    writeFileSync(join(parentCache, "tsconfig.tsbuildinfo"), "stale");
    process.env.XDG_CACHE_HOME = parentCache;
    writeFileSync(join(project, "package.json"), '{"type":"commonjs"}');
    writeFileSync(
      join(project, "check"),
      `
const fs = require("node:fs");
const path = require("node:path");
const cache = process.env.XDG_CACHE_HOME;
fs.appendFileSync(path.join(__dirname, "commands.jsonl"), JSON.stringify({
  cache, empty: cache !== undefined && fs.readdirSync(cache).length === 0,
  threads: process.env.RAYON_NUM_THREADS, bench: process.env.VIZE_BENCH,
}) + "\\n");
if (cache) fs.writeFileSync(path.join(cache, "tsconfig.tsbuildinfo"), "checked");
console.log(JSON.stringify({ files: [], errorCount: 0, warningCount: 0 }));
console.error("Type checked 1 files in 1s (collect: 0.1s, imports: 0.2s, gen: 0.3s, corsa: 0.4s)");
`,
    );
    for (let run = 0; run < 2; run++) {
      const gate = runVizeCheck(process.execPath, "tsgo", project, [], { RAYON_NUM_THREADS: "3" });
      assert.equal(gate.status, 0);
      assert.deepEqual(gate.report, { files: [], errorCount: 0, warningCount: 0 });
      runPrCommand(process.execPath, ["check"], {
        cwd: project,
        path: process.env.PATH ?? "",
        runs: 1,
        warmups: 0,
        thresholdPercent: 5,
        allowNonZeroExit: false,
        threads: 2,
      });
      assert.deepEqual(runWorkspaceCheck(process.execPath, project), {
        total: 1,
        collect: 0.1,
        imports: 0.2,
        gen: 0.3,
        corsa: 0.4,
      });
    }
    const records = readFileSync(join(project, "commands.jsonl"), "utf8")
      .trim()
      .split("\n")
      .map(
        (line) =>
          JSON.parse(line) as { cache: string; empty: boolean; threads?: string; bench?: string },
      );
    assert.equal(records.length, 6);
    assert.equal(new Set(records.map((row) => row.cache)).size, 6);
    assert.ok(
      records.every((row) => row.empty && row.cache !== parentCache && !existsSync(row.cache)),
    );
    assert.equal(records[0]?.threads, "3");
    assert.equal(records[1]?.threads, "2");
    assert.equal(records[0]?.bench, "1");
    assert.equal(records[1]?.bench, "1");
    assert.equal(readFileSync(join(parentCache, "tsconfig.tsbuildinfo"), "utf8"), "stale");
  } finally {
    if (previous === undefined) delete process.env.XDG_CACHE_HOME;
    else process.env.XDG_CACHE_HOME = previous;
    rmSync(project, { recursive: true, force: true });
    rmSync(parentCache, { recursive: true, force: true });
  }
});
