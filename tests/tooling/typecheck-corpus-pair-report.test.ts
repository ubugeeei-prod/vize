import assert from "node:assert/strict";
import { test } from "node:test";

import type { Diagnostic } from "../../tools/benchmarks/scripts/typecheck-corpus-pair-parse.ts";
import {
  type LaneResult,
  type PairResults,
  percentDelta,
  renderPairSummary,
  summarizePair,
} from "../../tools/benchmarks/scripts/typecheck-corpus-pair-report.ts";

function lane(
  host: string,
  variant: LaneResult["variant"],
  cpuMs: number[],
  diagnostics: Diagnostic[],
  fingerprint = `${host}-${diagnostics.length}`,
): LaneResult {
  const stats: [string, string][] = [
    ["Types", "100"],
    ["Instantiations", "1000"],
    ["Check time", "1.500s"],
  ];
  return {
    corpus: "misskey",
    corpusLabel: "misskey frontend",
    sfcs: 583,
    host,
    variant,
    probe: { files: 583, transforms: 583, processes: 1, bytes: 2048, fingerprint },
    diagnostics,
    runs: cpuMs.map((ms) => ({
      wallMs: ms / 4,
      cpuMs: ms,
      time: { wallS: 1, userS: 1, sysS: 0, maxRssKb: 1024 * 512, status: 2 },
      status: 2,
      stats,
      fingerprint,
    })),
  };
}

void test("the summary tables medians per lane and reports diagnostic identity", () => {
  const d = (line: string): Diagnostic => ["src/a.vue", "error", line, "1", "TS2322", "x"];
  const results: PairResults = {
    schemaVersion: 1,
    base: { sha: "a".repeat(40) },
    head: { sha: "b".repeat(40) },
    runs: 3,
    measurement: {
      runner: "blacksmith-32vcpu-ubuntu-2404",
      packages: {
        typescript: "7.1.0-beta",
        tsrs: "0.20.1-ts7.1.0-beta",
        "@vue/content-mapper": "4.0.0-alpha.3",
        vue: "3.5.44",
        "@types/node": "26.6.6",
        "vue-tsc": "3.3.11",
        "vue-tsc typescript": "6.0.3",
      },
    },
    references: [
      {
        corpus: "misskey",
        tool: "vue-tsc",
        cpuMs: 30_000,
        maxRssKb: 1024 * 900,
        fingerprint: "c".repeat(64),
        diagnostics: [d("1"), d("4")],
      },
    ],
    lanes: [
      lane("tsc", "vize-base", [10_000, 12_000, 11_000], [d("1")]),
      lane("tsc", "vize-head", [9_000, 9_900, 9_500], [d("1")]),
      lane("tsc", "vue", [20_000, 20_000, 20_000], [d("1"), d("2")]),
      lane("tsrs", "vize-base", [4_000, 4_000, 4_000], [d("1")]),
      lane("tsrs", "vize-head", [4_000, 4_000, 4_000], [d("1"), d("3")]),
      lane("tsrs", "vue", [5_000, 5_000, 5_000], [d("1")]),
    ],
  };
  const summary = summarizePair(results);
  assert.deepEqual(
    summary.identity.map((entry) => [entry.host, entry.status, entry.added.length]),
    [
      ["tsc", "IDENTICAL", 0],
      ["tsrs", "CHANGED", 1],
    ],
  );
  assert.deepEqual(summary.unstable, []);
  // vize-base is compared to head above, so only head and Vue's mapper face the reference.
  assert.deepEqual(
    summary.agreement.map((e) => [e.host, e.variant, e.shared, e.laneOnly, e.referenceOnly]),
    [
      ["tsc", "vize-head", 1, 0, 1],
      ["tsc", "vue", 1, 1, 1],
      ["tsrs", "vize-head", 1, 1, 1],
      ["tsrs", "vue", 1, 0, 1],
    ],
  );
  const markdown = renderPairSummary(results, summary);
  assert.match(markdown, /### Agreement with vue-tsc 3\.3\.11 \(typescript 6\.0\.3\)/u);
  assert.match(markdown, /\| misskey \| tsc \| vue \| 2 \| 1 \| 1 \| 1 \|/u);
  assert.match(
    markdown,
    /- misskey \/ vue-tsc: 2 diagnostics, CPU 30\.00 s, max RSS 900 MB, sha256 `cccccccccccc`/u,
  );
  assert.match(
    markdown,
    /\| misskey frontend \(583 SFC\) \| tsc Vize base \| tsc Vize head \| Δ \| tsc Vue \| tsrs Vize base/u,
  );
  assert.match(
    markdown,
    /\| CPU s \(user\+sys, process tree\) \| 11\.00 \| 9\.50 \| −13\.6% \| 20\.00 \| 4\.00 \| 4\.00 \| 0\.0% \| 5\.00 \|/u,
  );
  // The fastest run shows when a median sits on a different mode than its pair.
  assert.match(
    markdown,
    /\| CPU s, fastest run \| 10\.00 \| 9\.00 \| −10\.0% \| 20\.00 \| 4\.00 \| 4\.00 \| 0\.0% \| 5\.00 \|/u,
  );
  assert.match(markdown, /\| Diagnostics \| 1 \| 1 \| IDENTICAL \| 2 \| 1 \| 2 \| \+1 −0 \| 1 \|/u);
  assert.match(markdown, /misskey \/ tsrs: \*\*CHANGED\*\*, base 1 → head 2 \(\+1 −0\)/u);
  assert.match(markdown, /\+ `src\/a\.vue\(3,1\): error TS2322: x`/u);

  const drifting = lane("tsc", "vize-head", [1, 1], [d("1")]);
  (drifting.runs[1] as { fingerprint: string }).fingerprint = "other";
  assert.deepEqual(
    summarizePair({
      ...results,
      lanes: [...results.lanes.slice(0, 1), drifting, ...results.lanes.slice(2)],
    }).unstable,
    ["misskey-tsc-vize-head"],
  );
  assert.equal(percentDelta(0, 0), "0.0%");
  assert.equal(percentDelta(2, 3), "+50.0%");
});

void test("single-checker counts sit beside the default-checker medians", () => {
  const results: PairResults = {
    schemaVersion: 1,
    base: { sha: "a".repeat(40) },
    head: { sha: "b".repeat(40) },
    runs: 1,
    measurement: { runner: "local", packages: {} },
    references: [],
    lanes: [
      lane("tsc", "vize-base", [1], []),
      lane("tsc", "vize-head", [1], []),
      lane("tsc", "vue", [1], []),
    ],
    singleChecker: [
      {
        corpus: "misskey",
        host: "tsc",
        variant: "vize-base",
        types: 80,
        instantiations: 800,
        fingerprint: "tsc-0",
      },
      {
        corpus: "misskey",
        host: "tsc",
        variant: "vize-head",
        types: 76,
        instantiations: 800,
        fingerprint: "other",
      },
    ],
  };
  const markdown = renderPairSummary(results, summarizePair(results));
  assert.match(markdown, /### Partition-free counts \(`tsc --checkers 1`\)/u);
  assert.match(markdown, /\| misskey \| Types \| 100 \| 100 \| 0\.0% \| 80 \| 76 \| −5\.0% \|/u);
  assert.match(
    markdown,
    /\| misskey \| Instantiations \| 1,000 \| 1,000 \| 0\.0% \| 800 \| 800 \| 0\.0% \|/u,
  );
  assert.match(markdown, /- misskey \/ vize-head: diagnostics differ at one checker/u);
  assert.doesNotMatch(markdown, /vize-base: diagnostics differ/u);
  assert.doesNotMatch(
    renderPairSummary({ ...results, singleChecker: undefined }, summarizePair(results)),
    /Partition-free/u,
  );
});
