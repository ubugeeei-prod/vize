import assert from "node:assert/strict";
import { test } from "node:test";
import { readAssets } from "../../support/compat/content-mapper/constructor-projection-parity.ts";
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
                semanticLinks: [
                  { sourceStart: 1, sourceLength: 1, targetStart: 2, targetLength: 1 },
                ],
                diagnosticDirectives: {
                  directives: [[1, 2, 3, 4, 0, 0]],
                  unusedExpectDirectiveDiagnostics: [],
                },
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
void test("certified generated field relocation preserves cross-role packets", () => {
  const rows = samples();
  const marker = " readonly __vizeHasFallthroughProps: true;";
  const prefix = " readonly __vizeComponentMarker: true; readonly __vizeRawProps?: Props;";
  const field = ' readonly __vizeFallthroughSource?: Partial<$NativeElement<"main">>;';
  const forwarded =
    ' readonly __vizeFallthroughProps?: $UnownedComponentAttrs<Partial<$NativeElement<"main">>, Props, {}>;';
  for (const row of rows) {
    const packet = JSON.parse(row.packet) as {
      text: string;
      mappings: unknown[];
      semanticLinks: unknown[];
      diagnosticDirectives: { directives: unknown[] };
      diagnostics: unknown[];
    };
    packet.text = prefix + marker + forwarded;
    packet.mappings = [];
    packet.semanticLinks = [];
    packet.diagnosticDirectives.directives = [];
    packet.diagnostics = [];
    if (row.role === "head") packet.text = prefix + field + marker + forwarded;
    row.packet = JSON.stringify(packet);
  }
  assert.doesNotThrow(() => summarize(rows, binaries, readAssets(process.cwd(), process.cwd())));
  const chosen = rows.find((row) => row.role === "head");
  assert.ok(chosen);
  for (const head of rows.filter(
    (row) => row.role === "head" && row.family === chosen.family && row.count === chosen.count,
  )) {
    const packet = JSON.parse(head.packet) as { text: string };
    packet.text = packet.text.replace("__vizeFallthroughSource?", "__vizeFallthroughSourceOther?");
    head.packet = JSON.stringify(packet);
  }
  assert.throws(() => summarize(rows, binaries, readAssets(process.cwd(), process.cwd())));
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
