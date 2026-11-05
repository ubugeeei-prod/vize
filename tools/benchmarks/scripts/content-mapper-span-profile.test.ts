import assert from "node:assert/strict";
import { test } from "node:test";
import { counts, families, summarize, type Sample } from "./content-mapper-span-profile.ts";

const binaries = { base: { source: "a".repeat(40) }, head: { source: "b".repeat(40) } };
function samples(): Sample[] {
  const rows: Sample[] = [];
  for (const family of families)
    for (const count of counts)
      for (const role of ["base", "head"] as const)
        for (const mode of ["runtime-off", "allocation"] as const)
          for (let round = 1; round <= 5; round++) {
            rows.push({
              schema: "vize.content-mapper-span-profile.v1",
              family,
              count,
              role,
              mode,
              round,
              sourceRevision: binaries[role].source,
              source: "日本 🧭\r\n",
              packet: JSON.stringify({
                text: "generated",
                extension: ".tsx",
                mappings: [[1, 2, 3, 4, 0, 63]],
                semanticLinks: [{ sourceStart: 1, targetStart: 2 }],
                diagnosticDirectives: { directives: [[1, 2, 3, 4, 0, 0]] },
                diagnostics: [
                  { code: 100001, start: 1, length: 2, messageText: "whole diagnostic" },
                ],
              }),
              elapsedNanoseconds: round,
              peakRssKiB: 1024 + round,
              allocation: {
                calls: mode === "allocation" ? 4 : 0,
                requestedBytes: mode === "allocation" ? 160 : 0,
                releasedBytes: 0,
                failures: 0,
              },
              cold: { generationCalls: 1, warmups: 0, persistentCache: false },
            });
          }
  return rows;
}
void test("whole matrix preserves every family and median", () => {
  const result = summarize(samples(), binaries);
  assert.deepEqual(
    result.map((row) => [row.family, row.count]),
    families.flatMap((family) => counts.map((count) => [family, count])),
  );
  const first = result[0];
  assert.ok(first);
  assert.deepEqual(first.base, {
    generationNs: 3,
    processPeakRssKiB: 1027,
    allocationCalls: 4,
    requestedBytes: 160,
  });
});
for (const field of [
  "text",
  "extension",
  "mappings",
  "semanticLinks",
  "diagnosticDirectives",
  "diagnostics",
])
  void test(`reject changed complete ${field}`, () => {
    const rows = samples();
    const first = rows[0];
    assert.ok(first);
    const packet = JSON.parse(first.packet) as Record<string, unknown>;
    delete packet[field];
    first.packet = JSON.stringify(packet);
    assert.throws(() => summarize(rows, binaries));
  });
const mutations: ReadonlyArray<(rows: Sample[]) => unknown> = [
  (rows) => rows.pop(),
  (rows) => {
    const first = rows[0];
    assert.ok(first);
    rows[1] = first;
  },
  (rows) => {
    const first = rows[0];
    assert.ok(first);
    first.source += "x";
  },
  (rows) => {
    const first = rows[0];
    assert.ok(first);
    first.allocation.calls = 1;
  },
  (rows) => {
    const row = rows.find((sample) => sample.mode === "allocation");
    assert.ok(row);
    row.allocation = { calls: 0, requestedBytes: 0, releasedBytes: 0, failures: 0 };
  },
];
for (const mutation of mutations)
  void test("reject incomplete, duplicated or contaminated evidence", () => {
    const rows = samples();
    mutation(rows);
    assert.throws(() => summarize(rows, binaries));
  });
