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
