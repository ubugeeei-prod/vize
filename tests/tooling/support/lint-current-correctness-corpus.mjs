import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { loadCorpus } from "../lint-default-correctness-reference.mjs";

// Preserve the complete historical reader and all of its authored controls.
export function loadCurrentCorpus(root) {
  const original = loadCorpus(root);
  const current = JSON.parse(
    fs.readFileSync(
      path.join(root, "tests/_fixtures/lint-default-correctness/current-range-cases.json"),
    ),
  );
  const inverse = structuredClone(current);
  withoutDefaultBindStyle(inverse);
  for (const [index, id, start, end, lspStart, lspEnd] of [
    [0, "sync", 18, 37, 18, 37],
    [2, "slot-scope", 28, 38, 28, 38],
    [4, "full-modifiers", 18, 49, 18, 49],
    [5, "utf8-crlf", 40, 59, 32, 51],
  ]) {
    const row = inverse.cases[index];
    const before = original.cases[index];
    assert.equal(row.id, id);
    assert.deepEqual([row.diagnostics[0].start, row.diagnostics[0].end], [start, end]);
    assert.deepEqual(
      [row.lspDiagnostics[0].range.start.character, row.lspDiagnostics[0].range.end.character],
      [lspStart, lspEnd],
    );
    row.diagnostics[0].start = before.diagnostics[0].start;
    row.diagnostics[0].end = before.diagnostics[0].end;
    row.lspDiagnostics[0].range.start.character = before.lspDiagnostics[0].range.start.character;
    row.lspDiagnostics[0].range.end.character = before.lspDiagnostics[0].range.end.character;
  }
  assert.deepEqual(inverse, original, "only the approved full byte/UTF-16 range differences");
  return current;
}

// #9643 enabled vue/v-bind-style by default, after the original corpus; its
// one finding on `full-modifiers` is the only other approved difference.
export function withoutDefaultBindStyle(packet) {
  const row = packet.cases[4];
  assert.equal(row.id, "full-modifiers");
  for (const key of [
    "diagnostics",
    "lspDiagnostics",
    "baselineDiagnostics",
    "baselineLspDiagnostics",
  ]) {
    const kept = row[key].filter((entry) => (entry.rule_name ?? entry.code) !== "vue/v-bind-style");
    assert.equal(kept.length, row[key].length - 1, key);
    row[key] = kept;
  }
}
