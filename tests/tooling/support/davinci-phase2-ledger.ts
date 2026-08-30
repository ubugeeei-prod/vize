import assert from "node:assert/strict";
import fs from "node:fs";

function p2_11Installment(number: number): URL {
  return new URL(
    `../../../davinci-road/plan/phase-2-records/p2-11/installment-${number}.md`,
    import.meta.url,
  );
}

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
  const record = currentP2_11InstallmentRecord(source, label);
  assert.ok(
    [
      /44 (?:landed\s+)?installments/iu,
      /through installment 44/iu,
      /installment 44/iu,
      /^\| 44\s+\|/imu,
    ].some((marker) => marker.test(record)),
    `${label} current record must cite installment 44`,
  );
  assert.match(record, /#5363/u, `${label} current record must cite #5363`);
  assert.doesNotMatch(record, /\bpending\b/iu, `${label} current record must not be pending`);
}

function currentP2_11InstallmentRecord(source: string, label: string): string {
  switch (label) {
    case "roadmap":
      return requiredSection(
        source,
        /^\*\*Current execution ledger/mu,
        /^\*\*Exit gate:/mu,
        "roadmap current execution ledger",
      );
    case "readme":
      return requiredLine(source, /^\| \[phase-2\.md\][^\n]+$/mu, "plan README phase 2 row");
    case "tasks":
      return requiredSection(
        source,
        /^\*\*Current series evidence/mu,
        /^\*\*Steps:\*\*/mu,
        "P2-11 current series evidence",
      );
    case "records":
      return requiredLine(source, /^\| \[P2-11\][^\n]+$/mu, "P2-11 records index row");
    case "p2_11":
      return requiredLine(source, /^\| 44\s+\|[^\n]+$/mu, "P2-11 installment 44 row");
    default:
      throw new Error(`unknown P2-11 current evidence label: ${label}`);
  }
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
    requiredLine(
      source,
      /^\| 40\s+\|[^\n]+#5214[^\n]+be344e787[^\n]+$/mu,
      "P2-11 installment 40 row",
    ),
    requiredLine(
      source,
      /^\| 41\s+\|[^\n]+#5359[^\n]+5b5ac0924[^\n]+$/mu,
      "P2-11 installment 41 row",
    ),
    requiredLine(
      source,
      /^\| 42\s+\|[^\n]+#5360[^\n]+f659b7e4e[^\n]+$/mu,
      "P2-11 installment 42 row",
    ),
    requiredLine(
      source,
      /^\| 43\s+\|[^\n]+#4862[^\n]+fdaa8d165[^\n]+$/mu,
      "P2-11 installment 43 row",
    ),
    requiredLine(
      source,
      /^\| 44\s+\|[^\n]+#5363[^\n]+1a717a959[^\n]+$/mu,
      "P2-11 installment 44 row",
    ),
    requiredSection(
      source,
      /^## Current named remainder/mu,
      /^## Not series installments/mu,
      "P2-11 current named remainder",
    ),
  ].join("\n");
}

export function assertP2_11InstallmentFiles(): void {
  const installments = new Map(
    [
      20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42,
      43, 44,
    ].map((number) => [number, fs.readFileSync(p2_11Installment(number), "utf8")]),
  );
  for (const [number, pattern] of [
    [20, /14-fixture S2-vs-shipped byte-for-byte battery/],
    [20, /does not tick P2-11/],
    [21, /Vue 2 pipe filters/],
    [22, /Vue 2 filter helper order/],
    [23, /Slot outlet same-name names/],
    [24, /Patch-flag matrix expansion/],
    [25, /Dynamic component patch flags/],
    [26, /Model listener patch order/],
    [27, /Dynamic component model arguments/],
    [28, /SFC style carriers are DOM-inert/],
    [29, /Bare Template Default Slots/],
    [30, /Inert Slot-Template Bindings/],
    [31, /Inline Slot-Template Carriers/],
    [31, /f5aa60553/],
    [32, /V-show Runtime Directives/],
    [32, /2be66b0f0/],
    [33, /V-html Raw HTML Props/],
    [33, /13cff4d99/],
    [34, /V-text Text-Content Props/],
    [34, /11750115a/],
    [35, /V-cloak DOM Cloak Markers/],
    [35, /02c4eb1a7/],
    [36, /Slot Outlet V-on Props/],
    [36, /cf7fc9a22/],
    [37, /Object V-bind Modifiers/],
    [37, /4e577b62/],
    [38, /Object V-on Modifiers/],
    [38, /f3959e7e3/],
    [39, /Recent Patch-Flag Witness/],
    [39, /22674520f/],
    [40, /publish graph firewall/i],
    [40, /be344e787/],
    [41, /Corpus Comparison Count/],
    [41, /5b5ac0924/],
    [42, /S2 DOM Emit Allocations/],
    [42, /f659b7e4e/],
    [43, /Dynamic Directive Argument Prefixing/],
    [43, /fdaa8d165/],
    [44, /Single Nested Slot Wrapper Defaults/],
    [44, /1a717a959/],
  ] as const) {
    assert.match(installments.get(number)!, pattern);
  }
}
