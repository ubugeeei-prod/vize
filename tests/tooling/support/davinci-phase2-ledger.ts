import assert from "node:assert/strict";

export function recordsTaskRow(source: string, id: string): string {
  const escaped = id.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  const match = new RegExp(`^\\| \\[${escaped}\\][^\\n]+$`, "mu").exec(source);
  assert.ok(match, `missing records row for ${id}`);
  return match[0];
}

export function requiredSection(source: string, start: RegExp, end: RegExp, label: string): string {
  const startMatch = start.exec(source);
  assert.ok(startMatch, `missing ${label}`);
  const afterStart = source.slice(startMatch.index);
  const endMatch = end.exec(afterStart.slice(startMatch[0].length));
  if (endMatch == null) return afterStart;
  return afterStart.slice(0, startMatch[0].length + endMatch.index);
}

export function requiredLine(source: string, pattern: RegExp, label: string): string {
  const match = pattern.exec(source);
  assert.ok(match, `missing ${label}`);
  return match[0];
}

export function p2_11CurrentRecordEvidence(source: string): string {
  return [
    requiredSection(
      source,
      /^The `emit\.rs` module docs keep/mu,
      /^## Installment table/mu,
      "P2-11 current record preface",
    ),
    requiredLine(source, /^\| 28\s+\|[^\n]+#5009[^\n]+$/mu, "P2-11 installment 28 row"),
    requiredLine(source, /^\| 29\s+\| pending\s+\|[^\n]+#5011[^\n]+$/mu, "P2-11 pending row"),
    requiredSection(
      source,
      /^## Current named remainder/mu,
      /^## Not series installments/mu,
      "P2-11 current named remainder",
    ),
  ].join("\n");
}
