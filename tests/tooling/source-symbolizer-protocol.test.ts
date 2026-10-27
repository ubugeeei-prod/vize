import assert from "node:assert/strict";
import { test } from "node:test";
import {
  parseFlatSourceReport,
  rustSourceCensus,
  sentinelStyle,
} from "../../tools/benchmarks/scripts/source-symbolizer-protocol.ts";

void test("Linux perf sentinel compatibility rejects LLVM symbol-name unknown output", () => {
  assert.equal(sentinelStyle("0x0000000000000000\n??\n??:0\n"), "gnu");
  assert.equal(sentinelStyle(",\n"), "llvm-echo");
  assert.equal(sentinelStyle("??\n??:0\n"), null);
  assert.equal(sentinelStyle(""), null);
});
void test("source census uses exact self samples, retains tiny and unresolved rows, and separates inclusive cost", () => {
  const report = [
    "# Children\tSelf\tSamples\tPeriod\tShared Object\tSymbol\tSource:Line",
    "60.00%\t0.03%\t1\t10000000\tvize\t[.] f\t/crates/vize/a.rs:7",
    "50.00%\t0.07%\t2\t20000000\tvize\t[.] g\t??:0",
    "70.00%\t20.00%\t10\t100000000\tlibc.so\t[.] c\t/lib/c.c:12",
  ].join("\n");
  const parsed = parseFlatSourceReport(report);
  assert.equal(parsed.qualifiedFormat, true);
  assert.equal(parsed.rows[0].childrenPercent, 60);
  assert.equal(parsed.rows[0].selfPercent, 0.03);
  const census = rustSourceCensus(parsed.rows, "vize", 13);
  assert.equal(census.qualified, true);
  assert.equal(census.allDsoCountsMatch, true);
  assert.equal(census.allSelfSamples, 13);
  assert.deepEqual(census.allDsos, [
    { dso: "vize", samples: 3, resolvedSourceSamples: 1, unresolvedSourceSamples: 2 },
    { dso: "libc.so", samples: 10, resolvedSourceSamples: 10, unresolvedSourceSamples: 0 },
  ]);
  assert.equal(census.targetSamples, 3);
  assert.equal(census.rustSourceSamples, 1);
  assert.equal(census.unresolvedTargetSamples, 2);
  assert.equal(rustSourceCensus(parsed.rows, "other", 13).qualified, false);
  assert.equal(rustSourceCensus(parsed.rows, "vize", 1).qualified, false);
  assert.equal(rustSourceCensus(parsed.rows, "vize", 13, "different.rs").qualified, false);
});
void test("abbreviated counts and foreign C source cannot qualify missing Rust attribution", () => {
  const parsed = parseFlatSourceReport(
    "# Overhead\tSamples\tShared Object\tSymbol\tSource:Line\n1.00%\t1K\tvize\tf\ta.rs:3",
  );
  assert.equal(parsed.rows[0].samples, null);
  assert.equal(rustSourceCensus(parsed.rows, "vize", 1000).qualified, false);
  assert.equal(parseFlatSourceReport("Samples 1K\n??:0").qualifiedFormat, false);
  const foreign = parseFlatSourceReport(
    "# Overhead\tSamples\tShared Object\tSymbol\tSource:Line\n1.00%\t3\tvize\tf\ta.rs:0\n1.00%\t3\tlibc\tf\ta.c:8",
  );
  assert.equal(rustSourceCensus(foreign.rows, "vize", 6).qualified, false);
});

void test("bounded packets require every unique authored error before every repaired file", async () => {
  const { verifyStagePacket } =
    await import("../../tools/benchmarks/scripts/source-symbolizer-stages.ts");
  const project = "/tmp/original-stage",
    names = ["Binding0.vue", "Binding1.vue"];
  const diagnostic = "error:2:7 [TS2322] Type 'string' is not assignable to type 'number'.";
  const packet = {
    fileCount: 2,
    errorCount: 2,
    warningCount: 0,
    files: names.map((file) => ({ file, diagnostics: [diagnostic] })),
  };
  assert.equal(verifyStagePacket(packet, project, names, true).files.length, 2);
  const repaired = {
    ...packet,
    errorCount: 0,
    files: names.map((file) => ({ file, diagnostics: [] })),
  };
  assert.equal(verifyStagePacket(repaired, project, names, false).files.length, 2);
  for (const bad of [
    { ...packet, files: packet.files.slice(0, 1) },
    { ...packet, files: [packet.files[0], packet.files[0]] },
    { ...packet, files: [{ file: "Binding0.vue", diagnostics: [] }, packet.files[1]] },
    { ...packet, files: [packet.files[0], { file: "elsewhere.vue", diagnostics: [diagnostic] }] },
    { ...packet, errorCount: 0 },
  ])
    assert.throws(() => verifyStagePacket(bad, project, names, true));
  assert.throws(() => verifyStagePacket(packet, project, names, false));
});

void test(
  "Linux adapter changes only a standalone comma and passes decoder output unchanged",
  { skip: process.platform !== "linux" },
  async () => {
    const { numericSentinelScript } =
      await import("../../tools/benchmarks/scripts/source-symbolizer-render.ts");
    const { spawnSync } = await import("node:child_process");
    const input = ",\n0x1234\nabcd\nprefix,suffix\n, \n,\n";
    const result = spawnSync("/bin/sh", ["-c", numericSentinelScript("/bin/cat")], {
      input,
      encoding: "utf8",
    });
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout, "0\n0x1234\nabcd\nprefix,suffix\n, \n0\n");
  },
);

void test("source links bind the compiled SHA and reject outside or unresolved paths", async () => {
  const { frozenSourceLocations } =
    await import("../../tools/benchmarks/scripts/source-symbolizer-protocol.ts");
  const sha = "d16307e5eebffa43277a4fd9e5a471b5312256cc";
  const row = (source: string) => ({
    dso: "vize",
    symbol: "function",
    source,
    selfPercent: 1,
    childrenPercent: null,
    samples: 1,
    period: "1",
  });
  const locations = frozenSourceLocations(
    [
      row("/build/vize/crates/check.rs:12"),
      row("/rustc/compiler/library/core.rs:44"),
      row("/build/vize/../outside.rs:1"),
      row("unresolved+0x12"),
    ],
    "/build/vize",
    sha,
  );
  assert.equal(
    locations[0].url,
    "https://github.com/ubugeeei-prod/vize/blob/" + sha + "/crates/check.rs#L12",
  );
  assert.deepEqual(
    locations.slice(1).map((entry) => entry.url),
    [null, null, null],
  );
  assert.throws(() => frozenSourceLocations([], "/build/vize", "main"));
});
