import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import {
  type CheckSample,
  attributeStrace,
  attributionTable,
  classifyPath,
  markdown,
  median,
  processTotals,
  summarizeCell,
  tracedPath,
} from "../../tools/benchmarks/scripts/typecheck-cold-pair-report.ts";
import { catalogVersion } from "../../tools/benchmarks/scripts/typecheck-cold-pair.ts";

const sample = (wallMs: number, outputSha = "a", exitCode = 1): CheckSample => ({
  wallMs,
  cpuSeconds: wallMs / 100,
  peakMiB: 400,
  vizeHwmMiB: 300,
  childHwmMiB: 900,
  exitCode,
  outputBytes: 10_000,
  outputSha,
  mirrorFiles: 1_050,
  mirrorBytes: 4_000_000,
});

const cell = (head: CheckSample[], allowOutputChange = false) =>
  summarizeCell({
    label: "linux-ext4",
    project: "vue-1000",
    builds: ["base", "head"],
    samples: new Map([
      ["base", [sample(2000), sample(2200), sample(2100)]],
      ["head", head],
    ]),
    allowOutputChange,
  });

test("medians ignore input order", () => {
  assert.equal(median([3, 1, 2]), 2);
  assert.equal(median([4, 1, 3, 2]), 2.5);
  assert.ok(Number.isNaN(median([])));
});

test("identical diagnostics publish medians and signed head deltas", () => {
  const result = cell([sample(1000), sample(1100), sample(1050)]);
  assert.equal(result.failure, undefined);
  assert.equal(result.outputsIdentical, true);
  const report = markdown("linux-ext4", [result]);
  assert.match(report, /\| head \| 1050 \(-50\.0%\) \(1000–1100\)/u);
  assert.match(report, /\| base \| 2100 \(2000–2200\)/u);
});

test("different diagnostics or exit codes publish no timing unless allowed", () => {
  const changed = [sample(1000, "b"), sample(1100, "b"), sample(1050, "b")];
  const refused = cell(changed);
  assert.equal(refused.failure, "builds reported different diagnostics");
  assert.doesNotMatch(markdown("x", [refused]), /\| head \|/u);
  assert.equal(cell(changed, true).failure, undefined);
  const exit = cell([sample(1000, "a", 0), sample(1100, "a", 0), sample(1050, "a", 0)]);
  assert.equal(exit.failure, "builds reported different diagnostics");
  const unstable = cell([sample(1000), sample(1100, "c"), sample(1050)], true);
  assert.equal(unstable.failure, "head reported runs differently");
});

test("catalog pins come from the workspace file", () => {
  const workspace = readFileSync(new URL("../../pnpm-workspace.yaml", import.meta.url), "utf8");
  assert.match(catalogVersion(workspace, "vue-beta", "vue"), /^\d+\.\d+\.\d+/u);
  assert.match(
    catalogVersion(workspace, "corsa-runtime", "@typescript/typescript-linux-x64"),
    /^\d+\.\d+\.\d+/u,
  );
  assert.throws(() => catalogVersion(workspace, "vue-beta", "missing"), /no missing pin/u);
});

const roots = {
  mirror: "/cache/run",
  modules: "/p/node_modules",
  project: "/p",
  checker: "/tools/checker",
};

test("paths resolve against descriptors and dirfds", () => {
  assert.equal(tracedPath("read", '3</p/a.vue>, "<template>"..., 9'), "/p/a.vue");
  assert.equal(tracedPath("openat", 'AT_FDCWD</p>, "src/a.vue", O_RDONLY'), "/p/src/a.vue");
  assert.equal(
    tracedPath("newfstatat", 'AT_FDCWD</p>, "/cache/run/x.ts", 0x1, 0'),
    "/cache/run/x.ts",
  );
  assert.equal(tracedPath("readlink", '"/p", 0x7ff, 1023'), "/p");
  assert.equal(classifyPath("/p/node_modules/vue/x.d.ts", roots), "node_modules");
  assert.equal(classifyPath("/cache/run/canon/a.ts", roots), "mirror");
  assert.equal(classifyPath("/tools/checker/lib/tsc", roots), "checker-install");
  assert.equal(classifyPath("/proc/self/maps", roots), "procfs");
  assert.equal(classifyPath(undefined, roots), "fd");
});

test("strace lines attribute to the process each pid last executed", () => {
  const log = [
    '100 execve("/bin/vize", ["vize"], 0x0 /* 3 vars */) = 0',
    '100 openat(AT_FDCWD</p>, "src/a.vue", O_RDONLY|O_CLOEXEC) = 3</p/src/a.vue>',
    '100 newfstatat(AT_FDCWD</p>, "/cache/run/x.ts", 0x1, 0) = -1 ENOENT (No such file or directory)',
    "100 clone3({flags=CLONE_VM|CLONE_VFORK}, 88 <unfinished ...>",
    "100 <... clone3 resumed>) = 101",
    '101 execve("/tools/checker/lib/tsc", ["tsc"], 0x1 /* 3 vars */) = 0',
    '101 openat(AT_FDCWD</cache/run>, "/p/node_modules/vue/index.d.ts", O_RDONLY <unfinished ...>',
    '100 readlink("/p", 0x7ff, 1023) = -1 EINVAL (Invalid argument)',
    "101 <... openat resumed>) = 5</p/node_modules/vue/index.d.ts>",
  ].join("\n");
  const counts = attributeStrace(log, roots);
  assert.equal(counts.get("vize\topenat\tsource\tok"), 1);
  assert.equal(counts.get("vize\tnewfstatat\tmirror\tENOENT"), 1);
  assert.equal(counts.get("tsc\topenat\tnode_modules\tok"), 1);
  assert.equal(counts.get("vize\treadlink\tsource\tEINVAL"), 1);
  assert.equal(
    [...counts.keys()].some((key) => key.includes("clone3")),
    false,
  );
  const table = attributionTable([
    ["base", counts],
    ["head", new Map([["vize\topenat\tsource\tok", 3]])],
  ]);
  assert.match(table, /\| vize \| openat \| source \| 1 \(0\) \| 3 \(0\) \|/u);
  assert.match(table, /\| vize \| newfstatat \| mirror \| 1 \(1\) \| 0 \(0\) \|/u);
  assert.match(processTotals([["base", counts]]), /\| tsc \| 2 \|/u);
});

test("a Windows checkout's CRLF workspace file still yields its pins", () => {
  const workspace = readFileSync(new URL("../../pnpm-workspace.yaml", import.meta.url), "utf8");
  const crlf = workspace.replaceAll("\n", "\r\n");
  assert.equal(
    catalogVersion(crlf, "vue-beta", "vue"),
    catalogVersion(workspace, "vue-beta", "vue"),
  );
});

test("a thread seen before its creator's clone returns joins that process", () => {
  const log = [
    '100 execve("/bin/vize", ["vize"], 0x0 /* 3 vars */) = 0',
    "100 clone3({flags=CLONE_VM|CLONE_THREAD}, 88 <unfinished ...>",
    '102 openat(AT_FDCWD</p>, "src/b.vue", O_RDONLY) = 4</p/src/b.vue>',
    "100 <... clone3 resumed>) = 102",
    '103 openat(AT_FDCWD</p>, "src/c.vue", O_RDONLY) = 5</p/src/c.vue>',
  ].join("\n");
  const counts = attributeStrace(log, roots);
  assert.equal(counts.get("vize\topenat\tsource\tok"), 1);
  assert.equal(counts.get("unknown\topenat\tsource\tok"), 1);
});
