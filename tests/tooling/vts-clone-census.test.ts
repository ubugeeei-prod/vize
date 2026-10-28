import assert from "node:assert/strict";
import test from "node:test";
import {
  censusFixture,
  assertPacket,
  packetDiagnostics,
} from "../../tools/benchmarks/scripts/vts-clone-census-fixture.ts";
import {
  censusProfile,
  type Profile,
} from "../../tools/benchmarks/scripts/vts-clone-census-profile.ts";

void test("tiny census graph has every feature and UTF16/CRLF-negative token", () => {
  for (const sfcs of [1, 10, 100]) {
    const valid = censusFixture(sfcs, false),
      invalid = censusFixture(sfcs, true);
    assert.equal(valid.expected.length, 0);
    assert.equal(invalid.expected.length, sfcs + 5);
    assert.equal(Object.keys(valid.files).filter((f) => f.endsWith(".vue")).length, sfcs);
    assert.ok(Object.keys(valid.files).some((f) => f.endsWith(".tsx")));
    const config = JSON.parse(valid.files["tsconfig.json"]);
    assert.equal(config.compilerOptions.incremental, false);
    assert.equal(config.files.filter((f: string) => f.endsWith(".vue")).length, sfcs);
    for (const diagnostic of invalid.expected) {
      const line = invalid.files[diagnostic.file].split("\n")[diagnostic.line - 1];
      const token = line.slice(diagnostic.column - 1);
      assert.ok(
        token.startsWith(diagnostic.file.endsWith(".vue") ? "text: string" : "dependencyCoverage"),
      );
    }
    assert.ok(valid.files["src/App000.vue"].includes("\r\n"));
    assert.ok(valid.files["src/graph/barrel.ts"].includes("export type"));
  }
});
void test("full diagnostic vectors reject hidden duplicates and unclassified entries", () => {
  const expected = censusFixture(1, true).expected;
  assertPacket([...expected].reverse(), expected);
  assert.throws(() => assertPacket([...expected, expected[0]], expected));
  assert.throws(() => assertPacket(expected.slice(1), expected));
  assert.throws(() =>
    packetDiagnostics(
      {
        errorCount: 1,
        warningCount: 0,
        fileCount: 1,
        files: [{ file: "x.vue", diagnostics: ["unclassified"] }],
      },
      "/fixture",
    ),
  );
  assert.throws(() =>
    packetDiagnostics(
      {
        errorCount: 0,
        warningCount: 0,
        fileCount: 2,
        files: [
          { file: "x.vue", diagnostics: [] },
          { file: "x.vue", diagnostics: [] },
        ],
      },
      "/fixture",
    ),
  );
});
void test("source profile admission requires real allocator and clone/parse reachability", () => {
  const counter = (name: string, units: number) => ({
    key: name,
    name,
    file: "test.rs",
    module: "test",
    kind: "counter",
    line: 1,
    calls: 1,
    units,
  });
  const key = "canon.dependency_scan.content.clone@test.rs:2";
  const profile: Profile = {
    schema_version: 1,
    truncation: { dropped_counters: 0, dropped_spans: 0 },
    allocation: { calls: 5, requested_bytes: 100 },
    spans: [
      { key, count: 1, wall_ns: {}, alloc: { calls: 1, bytes: 20, self_calls: 1, self_bytes: 20 } },
    ],
    micro: [
      counter("canon.dependency_scan.content.clone.calls", 1),
      counter("canon.dependency_scan.content.clone.source_bytes", 100),
      counter("canon.dependency_scan.content.generated_source_bytes", 90),
      counter("canon.dependency_scan.occurrences.uncached.calls", 1),
      {
        key,
        name: "canon.dependency_scan.content.clone",
        file: "test.rs",
        module: "test",
        kind: "span",
        line: 2,
        calls: 1,
        units: 0,
      },
    ],
  };
  const selected = censusProfile(profile, 1);
  assert.equal(selected.cachedOccurrences, null);
  assert.equal(selected.actualHashOperations, null);
  assert.equal(selected.physicalCopyBytes, null);
  assert.throws(() => censusProfile({ ...profile, allocation: null }, 1));
  assert.throws(() =>
    censusProfile({ ...profile, truncation: { dropped_counters: 0, dropped_spans: 1 } }, 1),
  );
  assert.throws(() => censusProfile({ ...profile, micro: [] }, 1));
});
