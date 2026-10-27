import assert from "node:assert/strict";
import { test } from "node:test";
import { CapacityCoverage, capacitySource } from "./support/lsp/diagnostic-capacity.ts";

test("bulk diagnostic coverage preserves multiplicity and requires exact native tokens and UTF16 ranges", () => {
  const probe = capacitySource(2, "file:///workspace/Coverage0002.vue");
  const coverage = new CapacityCoverage([probe]);
  const good = {
    uri: probe.uri,
    version: 1,
    diagnostics: [
      {
        source: "vize/types",
        code: 2339,
        message: "Property '" + probe.token + "' does not exist",
        range: {
          start: { line: probe.line, character: probe.character },
          end: { line: probe.line, character: probe.character + probe.token.length },
        },
      },
    ],
  };
  const diagnostic = good.diagnostics[0];
  assert.ok(diagnostic);
  for (const packet of [
    { ...good, version: 0 },
    { ...good, diagnostics: [] },
    { ...good, diagnostics: [{ ...diagnostic, source: "vize/lint" }] },
    { ...good, diagnostics: [{ ...diagnostic, message: "wrong token" }] },
    {
      ...good,
      diagnostics: [
        {
          ...diagnostic,
          range: {
            ...diagnostic.range,
            start: { line: probe.line, character: probe.character + 1 },
          },
        },
      ],
    },
    { ...good, diagnostics: [diagnostic, diagnostic] },
  ])
    coverage.record(packet, 1);
  assert.throws(() => coverage.assertComplete(1, false));
  coverage.record(good, 2);
  coverage.record(good, 3);
  coverage.assertComplete(1, false);
  const row = coverage.rows(1, false)[0];
  assert.ok(row);
  assert.equal(row.historicalMatchingPackets, 2);
  assert.equal(coverage.packets.length, 8);
  coverage.record({ ...good, diagnostics: [] }, 4);
  assert.throws(() => coverage.assertComplete(1, false), "later empty must replace the old error");
  coverage.record(good, 5);
  coverage.assertComplete(1, false);
  assert.ok(probe.invalid.includes("\r\n"));
  assert.equal(probe.character, "/*🌱日本語*/ value0002.".length);
});

test("repair needs a current complete empty packet for every URI", () => {
  const first = capacitySource(0, "file:///workspace/A.vue");
  const second = capacitySource(1, "file:///workspace/B.vue");
  const coverage = new CapacityCoverage([first, second]);
  coverage.record({ uri: first.uri, version: 3, diagnostics: [] }, 1);
  coverage.record({ uri: second.uri, version: 2, diagnostics: [] }, 1);
  coverage.record({ uri: second.uri, version: 3, diagnostics: [{ code: "typecheck-pending" }] }, 1);
  assert.throws(() => coverage.assertComplete(3, true));
  coverage.record({ uri: second.uri, version: 3, diagnostics: [] }, 2);
  coverage.assertComplete(3, true);
  coverage.record({ uri: second.uri, version: 3, diagnostics: [{ code: 2339 }] }, 3);
  assert.throws(() => coverage.assertComplete(3, true), "later error must replace repaired empty");
});
