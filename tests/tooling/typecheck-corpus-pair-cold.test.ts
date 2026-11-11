import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test, type TestContext } from "node:test";

import {
  type ColdLane,
  type ColdRunRecord,
  dominantPart,
  statMs,
  summarizeCold,
} from "../../tools/benchmarks/scripts/typecheck-corpus-pair-cold.ts";
import { renderCold } from "../../tools/benchmarks/scripts/typecheck-corpus-pair-cold-report.ts";
import {
  CORPORA,
  selectCorpora,
} from "../../tools/benchmarks/scripts/typecheck-corpus-pair-corpora.ts";
import { hostArgs } from "../../tools/benchmarks/scripts/typecheck-corpus-pair-host.ts";
import {
  RSS_TAG,
  environTag,
  parseProcStatus,
  startRssSampler,
  treeRss,
} from "../../tools/benchmarks/scripts/typecheck-corpus-pair-rss.ts";
import { hasMutationSet } from "../../tools/benchmarks/scripts/typecheck-corpus-mutations.ts";
import { root } from "./support/github-workflows.ts";

function scratch(t: TestContext): string {
  const dir = realpathSync(mkdtempSync(join(tmpdir(), "typecheck-corpus-cold-")));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  return dir;
}

void test("the pair checks four pinned corpora cold and plants every one", () => {
  assert.deepEqual(
    CORPORA.map((corpus) => [corpus.id, corpus.sfcs]),
    [
      ["misskey", 583],
      ["n8n", 1113],
      ["nuxt-ui", 187],
      ["reka-ui", 455],
    ],
  );
  for (const corpus of CORPORA) {
    assert.ok(corpus.source.startsWith(corpus.submodule), corpus.id);
    // No run reads or writes a tsbuildinfo.
    assert.equal(corpus.tsconfig.compilerOptions.incremental, false, corpus.id);
  }
  const gitmodules = readFileSync(join(root, ".gitmodules"), "utf8");
  for (const corpus of CORPORA) assert.ok(gitmodules.includes(`path = ${corpus.submodule}\n`));
  assert.deepEqual(
    selectCorpora("").map((corpus) => corpus.id),
    CORPORA.map((corpus) => corpus.id),
  );
  assert.deepEqual(
    selectCorpora("reka-ui, misskey").map((corpus) => corpus.id),
    ["misskey", "reka-ui"],
  );
  assert.throws(() => selectCorpora("misskey,vuetify"), /unknown corpus: vuetify/u);
  assert.deepEqual(
    CORPORA.map((corpus) => hasMutationSet(corpus.id)),
    [true, true, true, true],
  );
  assert.deepEqual(hostArgs({ id: "tsrs-default" }, "/p", "t.json"), [
    "--noEmit",
    "--runExternalCode",
    "--extendedDiagnostics",
    "--pretty",
    "false",
    "-p",
    "/p/t.json",
  ]);
});

void test("/proc status and environ parse into the tree's host and mapper peaks", () => {
  const status = (name: string, ppid: number, rss?: number, hwm?: number) =>
    [
      `Name:\t${name}`,
      "State:\tS (sleeping)",
      `PPid:\t${ppid}`,
      ...(rss == null ? [] : [`VmHWM:\t${hwm} kB`, `VmRSS:\t${rss} kB`]),
    ].join("\n");
  assert.deepEqual(parseProcStatus(status("tsrs", 10, 900, 1200)), {
    name: "tsrs",
    ppid: 10,
    rssKb: 900,
    hwmKb: 1200,
  });
  // A zombie has no memory lines.
  assert.equal(parseProcStatus(status("node", 10)), null);
  assert.equal(parseProcStatus(""), null);
  assert.equal(environTag(`PATH=/bin\0${RSS_TAG}=cold-7\0HOME=/h\0`), "cold-7");
  assert.equal(environTag("PATH=/bin\0"), null);
  assert.deepEqual(
    treeRss({
      root: 1,
      tag: "cold-7",
      samples: 40,
      treePeakKb: 2000,
      procs: [
        { pid: 1, ppid: 0, name: "time", hwmKb: 2 },
        { pid: 2, ppid: 1, name: "time", hwmKb: 2 },
        { pid: 3, ppid: 2, name: "tsrs", hwmKb: 1500 },
        { pid: 4, ppid: 3, name: "node", hwmKb: 300 },
        { pid: 5, ppid: 4, name: "node", hwmKb: 100 },
      ],
    }),
    {
      samples: 40,
      treePeakKb: 2000,
      hostKb: 1500,
      hostName: "tsrs",
      mapperKb: 400,
      mappers: 2,
    },
  );
});

void test(
  "the sampler records a tagged tree's summed peak and its child",
  { skip: process.platform !== "linux" },
  (t) => {
    const dir = scratch(t);
    const sampler = startRssSampler(join(dir, "rss.jsonl"));
    t.after(sampler.stop);
    const hold = "const b = Buffer.alloc(64 << 20, 1); setTimeout(() => b.length, 400);";
    const parent = `require("node:child_process").spawnSync(process.execPath, ["-e", ${JSON.stringify(hold)}])`;
    const result = spawnSync(process.execPath, ["-e", parent], {
      env: { ...process.env, [RSS_TAG]: "sampled" },
    });
    assert.equal(result.status, 0);
    const tree = sampler.take("sampled");
    assert.ok(tree, "no record for the tagged tree");
    // Node 24 names its main thread, and so /proc/<pid>/status, "MainThread".
    assert.ok(["node", "MainThread"].includes(tree.hostName), tree.hostName);
    assert.equal(tree.mappers, 1);
    assert.ok(tree.mapperKb > 64 * 1024, `child peak ${tree.mapperKb} kB`);
    assert.ok(tree.treePeakKb >= tree.mapperKb, `tree ${tree.treePeakKb} kB`);
  },
);

const run = (wallMs: number, stats: Record<string, string>, rssKb = 1024): ColdRunRecord => ({
  wallMs,
  cpuMs: 2000,
  time: { wallS: wallMs / 1000, userS: 1, sysS: 1, maxRssKb: 4 * 1024, status: 2 },
  status: 2,
  stats: Object.entries(stats),
  fingerprint: "f",
  rss: {
    samples: 9,
    treePeakKb: rssKb * 5,
    hostKb: rssKb * 4,
    hostName: "tsrs",
    mapperKb: rssKb,
    mappers: 1,
  },
  cacheFiles: 0,
});

void test("cold lanes split wall into host phases and what lies outside its Total", () => {
  assert.equal(statMs([["Check time", "0.148s"]], "Check time"), 148);
  assert.equal(statMs([["Check time", "0.148s"]], "Bind time"), null);
  const stats = (check: string) => ({
    "Config time": "0.002s",
    "Parse time": "0.200s",
    "Content mapper request wait time": "0.170s",
    "Bind time": "0.000s",
    "Check time": check,
    "Total time": "0.360s",
  });
  const lane: ColdLane = {
    corpus: "misskey",
    host: "tsrs",
    variant: "vize-head",
    runs: [run(420, stats("0.150s")), run(400, stats("0.140s")), run(480, stats("0.160s"))],
  };
  const summary = summarizeCold(lane);
  assert.equal(summary.wallMs, 420);
  assert.equal(summary.outsideTotalMs, 60);
  assert.equal(summary.checkMs, 150);
  assert.equal(summary.treeMb, 5);
  assert.equal(summary.mapperMb, 1);
  assert.equal(summary.largestMb, 4);
  // Parse is 200 ms with 170 ms of mapper wait inside it.
  assert.deepEqual(dominantPart(summary), { part: "mapper wait", ms: 170 });
  const floor: ColdLane = { ...lane, variant: "floor-absolute", runs: [run(300, stats("0.010s"))] };
  const markdown = renderCold({
    runs: 3,
    runner: "local",
    corpora: [{ id: "misskey", label: "misskey frontend", sfcs: 583 }],
    lanes: [floor, lane],
    startup: [{ host: "tsrs", wallMs: [40, 42, 41] }],
    replies: { "misskey-tsrs-vize-head": { replyBytes: 2 * 1048576, spans: 1500, directives: 7 } },
    notes: ["misskey / tsrs --fast: zero-wait replay diagnostics IDENTICAL to Vize head"],
  });
  assert.match(
    markdown,
    /\| misskey frontend \| tsrs 0\.20 --fast \| 583 \| 420 \| ≤80 \| 60 \| 170 \| 30 \| 150 \| 5 \| mapper wait \(170 ms\) \| 300 \| n\/a \| n\/a \|/u,
  );
  assert.match(markdown, /- tsrs 0\.20 --fast: 41 ms/u);
  // Vize head sorts before the floors of the same host.
  assert.ok(
    markdown.indexOf("| tsrs 0.20 --fast | Vize head |") <
      markdown.indexOf("| tsrs 0.20 --fast | floor: absolute"),
  );
  assert.match(markdown, /\| 2\.0 \| 1,500 \| 7 \| 0 \|/u);
  assert.match(markdown, /IDENTICAL to Vize head/u);
});
