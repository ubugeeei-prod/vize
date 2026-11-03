import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import { createTypecheckToolVariants } from "../../tools/benchmarks/scripts/compare-tools-typecheck.mjs";
import { withEmptyUserCache } from "../../tools/benchmarks/scripts/empty-user-cache.ts";
import { root } from "./support/github-workflows.ts";

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
  for (const file of ["check-gate.mjs", "compare-pr.ts", "check-workspace.ts"]) {
    const source = readFileSync(join(root, "tools/benchmarks/scripts", file), "utf8");
    assert.match(source, /withEmptyUserCache\(\(cache\) =>/u, file);
    assert.match(source, /\.\.\.cache\b/u, file);
  }
});
