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

export function assertCurrentP2_11Installment(source: string, label: string): void {
  const marker = /39 (?:landed\s+)?installments|installment 39|\| 39\s+\|/i;
  const match = new RegExp(
    `(?:${marker.source})[\\s\\S]{0,160}#5212|#5212[\\s\\S]{0,160}(?:${marker.source})`,
    "iu",
  ).exec(source);
  assert.ok(match, `${label} must cite current installment 39 and #5212 together`);
  assert.doesNotMatch(match[0], /\bpending\b/i, `${label} current installment must not be pending`);
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
    requiredLine(
      source,
      /^\| 29\s+\|[^\n]+#5011[^\n]+3565326fe[^\n]+$/mu,
      "P2-11 installment 29 row",
    ),
    requiredLine(
      source,
      /^\| 30\s+\|[^\n]+#5178[^\n]+2299a2114[^\n]+$/mu,
      "P2-11 installment 30 row",
    ),
    requiredLine(
      source,
      /^\| 31\s+\|[^\n]+#5183[^\n]+f5aa60553[^\n]+$/mu,
      "P2-11 installment 31 row",
    ),
    requiredLine(
      source,
      /^\| 32\s+\|[^\n]+#5198[^\n]+2be66b0f0[^\n]+$/mu,
      "P2-11 installment 32 row",
    ),
    requiredLine(
      source,
      /^\| 33\s+\|[^\n]+#5200[^\n]+13cff4d99[^\n]+$/mu,
      "P2-11 installment 33 row",
    ),
    requiredLine(
      source,
      /^\| 34\s+\|[^\n]+#5203[^\n]+11750115a[^\n]+$/mu,
      "P2-11 installment 34 row",
    ),
    requiredLine(
      source,
      /^\| 35\s+\|[^\n]+#5205[^\n]+02c4eb1a7[^\n]+$/mu,
      "P2-11 installment 35 row",
    ),
    requiredLine(
      source,
      /^\| 36\s+\|[^\n]+#5207[^\n]+cf7fc9a22[^\n]+$/mu,
      "P2-11 installment 36 row",
    ),
    requiredLine(
      source,
      /^\| 37\s+\|[^\n]+#5208[^\n]+4e577b62a[^\n]+$/mu,
      "P2-11 installment 37 row",
    ),
    requiredLine(
      source,
      /^\| 38\s+\|[^\n]+#5210[^\n]+f3959e7e3[^\n]+$/mu,
      "P2-11 installment 38 row",
    ),
    requiredLine(
      source,
      /^\| 39\s+\|[^\n]+#5212[^\n]+22674520f[^\n]+$/mu,
      "P2-11 installment 39 row",
    ),
    requiredSection(
      source,
      /^## Current named remainder/mu,
      /^## Not series installments/mu,
      "P2-11 current named remainder",
    ),
  ].join("\n");
}
