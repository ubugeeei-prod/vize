import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { loadCorpus, wholeJson } from "../lint-default-correctness-reference.mjs";

function location(source, offset) {
  const before = Buffer.from(source).subarray(0, offset).toString("utf8");
  const lines = before.split("\n");
  return [lines.length, lines.at(-1).length + 1];
}

// The frozen reference helper records Unicode scalar columns. Current CLI
// output follows LSP and reports one-based UTF-16 columns.
export function currentWholeJson(entry, findings) {
  const packet = wholeJson(entry, findings);
  assert.equal(packet[0].messages.length, findings.length);
  for (const [index, finding] of findings.entries()) {
    const message = packet[0].messages[index];
    const [line, column] = location(entry.source, finding.start);
    const [endLine, endColumn] = location(entry.source, finding.end);
    assert.equal(message.line, line);
    assert.equal(message.endLine, endLine);
    message.column = column;
    message.endColumn = endColumn;
  }
  return packet;
}

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
  withoutDefaultPropsDestructuring(inverse);
  withoutCurrentUnusedSlotProps(inverse);
  for (const [index, id, start, end, lspStart, lspEnd] of [
    [0, "sync", 18, 37, 18, 37],
    [2, "slot-scope", 28, 38, 28, 38],
    [3, "scope", 28, 33, 28, 33],
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
  restoreHistoricalHelp(inverse, original);
  assert.deepEqual(inverse, original, "only the approved full byte/UTF-16 range differences");
  return current;
}

export function restoreHistoricalHelp(packet, historical) {
  const fields = ["diagnostics", "essentialDiagnostics", "baselineDiagnostics"];
  for (const [caseIndex, row] of packet.cases.entries()) {
    const before = historical.cases[caseIndex];
    assert.equal(row.id, before.id);
    for (const field of fields) {
      if (!row[field] && !before[field]) continue;
      assert.equal(row[field].length, before[field].length, `${row.id} / ${field}`);
      for (const [diagnosticIndex, diagnostic] of row[field].entries()) {
        const previous = before[field][diagnosticIndex];
        if (Object.hasOwn(previous, "help")) diagnostic.help = previous.help;
        else delete diagnostic.help;
      }
    }
  }
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

// The props destructuring happy-path default was promoted after the frozen
// correctness corpus. Remove only its three newly observed macro witnesses
// when proving the complete historical inverse.
export function withoutDefaultPropsDestructuring(packet) {
  const ids = new Set(["macro-import", "macro-valid", "macro-comment"]);
  for (const row of packet.cases) {
    if (!ids.has(row.id)) continue;
    for (const [field, identity] of [
      ["diagnostics", "rule_name"],
      ["essentialDiagnostics", "rule_name"],
      ["baselineDiagnostics", "rule_name"],
      ["lspDiagnostics", "code"],
      ["baselineLspDiagnostics", "code"],
    ]) {
      if (!row[field]) continue;
      const kept = row[field].filter(
        (entry) => entry[identity] !== "script/define-props-destructuring",
      );
      assert.equal(kept.length, row[field].length - 1, `${row.id} / ${field}`);
      row[field] = kept;
    }
  }
}

// Current semantic analysis recognizes unused bindings in the two legacy slot
// spellings. The frozen corpus predates that accuracy fix, so remove exactly
// those two findings when proving the historical inverse.
export function withoutCurrentUnusedSlotProps(packet) {
  for (const row of packet.cases) {
    if (!new Set(["slot-scope", "scope"]).has(row.id)) continue;
    for (const [field, identity] of [
      ["diagnostics", "rule_name"],
      ["baselineDiagnostics", "rule_name"],
      ["lspDiagnostics", "code"],
      ["baselineLspDiagnostics", "code"],
    ]) {
      const kept = row[field].filter((entry) => entry[identity] !== "vue/no-unused-vars");
      assert.equal(kept.length, row[field].length - 1, `${row.id} / ${field}`);
      row[field] = kept;
    }
  }
}
