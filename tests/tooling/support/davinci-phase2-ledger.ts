import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  P2_11_CURRENT,
  p2_11FileExpectations,
  p2_11Installment,
  p2_11TableRows,
} from "./davinci-phase2-ledger/p2-11-rows.ts";

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
  const current = P2_11_CURRENT.number;
  assert.ok(
    [
      new RegExp(`${current} (?:landed\\s+)?installments`, "iu"),
      new RegExp(`through installment ${current}`, "iu"),
      new RegExp(`installment ${current}`, "iu"),
      new RegExp(`^\\| ${current}\\s+\\|`, "imu"),
    ].some((marker) => marker.test(record)),
    `${label} current record must cite installment ${current}`,
  );
  assert.match(
    record,
    new RegExp(`#${P2_11_CURRENT.pr}`, "u"),
    `${label} current record must cite #${P2_11_CURRENT.pr}`,
  );
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
      return requiredLine(
        source,
        new RegExp(`^\\| ${P2_11_CURRENT.number}\\s+\\|[^\\n]+$`, "mu"),
        `P2-11 installment ${P2_11_CURRENT.number} row`,
      );
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
    ...p2_11TableRows.map(([number, pr, sha]) =>
      requiredLine(
        source,
        new RegExp(`^\\| ${number}\\s+\\|[^\\n]+#${pr}[^\\n]+${sha}[^\\n]+$`, "mu"),
        `P2-11 installment ${number} row`,
      ),
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
  const expectedNumbers = Array.from({ length: P2_11_CURRENT.number }, (_, index) => index + 1);
  assert.deepEqual(
    existingP2_11InstallmentNumbers(),
    expectedNumbers,
    "P2-11 installment files must be contiguous through the current ledger pin",
  );

  const installments = new Map(
    [...new Set(p2_11FileExpectations.map(([number]) => number))].map((number) => [
      number,
      fs.readFileSync(p2_11Installment(number), "utf8"),
    ]),
  );
  for (const [number, pattern] of p2_11FileExpectations) {
    assert.match(installments.get(number)!, pattern);
  }
}

function existingP2_11InstallmentNumbers(): number[] {
  const directory = path.dirname(fileURLToPath(p2_11Installment(1)));
  return fs
    .readdirSync(directory)
    .map((entry) => /^installment-(\d+)\.md$/u.exec(entry)?.[1])
    .filter((number): number is string => number != null)
    .map((number) => Number(number))
    .sort((left, right) => left - right);
}

function phaseTaskSection(source: string, id: string): string {
  const start = new RegExp(`^## ${id} —`, "mu").exec(source)?.index;
  if (start == null) throw new Error(`missing ${id} contract`);
  const tail = source.slice(start);
  const next = /^## P2-/mu.exec(tail.slice(1))?.index;
  return next == null ? tail : tail.slice(0, next + 1);
}

function phaseDependencySet(source: string, id: string, taskIds: string[]): string[] {
  const section = phaseTaskSection(source, id);
  const raw = /\*\*Deps:\*\* (?<deps>[\s\S]*?) \*\*Non-goals:\*\*/u.exec(section)?.groups?.deps;
  assert.ok(raw, `missing ${id} dependency clause`);
  if (raw === "all of P2-1..P2-19.") return taskIds.filter((task) => task !== "P2-20");
  return raw.match(/P2-\d+(?:[ab])?/gu) ?? [];
}

function exitGateItems(source: string) {
  const gate = requiredSection(
    source,
    /^## Exit gate \(machine-checkable\)/mu,
    /$a/mu,
    "P2 exit gate",
  );
  return [...gate.matchAll(/^- \[(?<checked>[ x])\] \*\*(?<title>[^*]+)\*\*/gmu)].map((match) => ({
    checked: match.groups!.checked === "x",
    title: match.groups!.title,
  }));
}

export function assertP2_17P2_20ExitBlockers(
  phase: string,
  tasksLater: string,
  taskIds: string[],
  p2_17Checked: boolean | undefined,
  p2_20Checked: boolean | undefined,
): void {
  const phaseLedger = requiredSection(
    phase,
    /^## Current execution ledger/mu,
    /^## Davinci describes/mu,
    "Phase 2 current ledger",
  );
  const p2_17 = phaseTaskSection(tasksLater, "P2-17");
  const p2_20 = phaseTaskSection(tasksLater, "P2-20");
  const gateItems = exitGateItems(phase);

  assert.equal(p2_17Checked, false, "P2-17 must not be ticked before review sign-off");
  assert.equal(p2_20Checked, false, "P2-20 must not be ticked before exit evaluation");
  assert.deepEqual(phaseDependencySet(tasksLater, "P2-17", taskIds), ["P2-11", "P2-12b", "P2-13"]);
  assert.deepEqual(
    phaseDependencySet(tasksLater, "P2-20", taskIds),
    taskIds.filter((id) => id !== "P2-20"),
  );

  assert.match(p2_17, /mechanical half is machine-checked and must land as tests/);
  assert.match(p2_17, /every S2 op's span resolves into its authored SFC/);
  assert.match(p2_17, /`schema_version` is present and negotiated/);
  assert.match(p2_20, /a line is ticked only when it is satisfied/);
  assert.match(p2_20, /an unticked line names its blocker/);
  assert.match(p2_20, /no line's wording is softened to make it tickable/);

  assert.match(phaseLedger, /P2-17\/P2-20 pre-exit blocker map/);
  assert.match(phaseLedger, /P2-11's S2 DOM lane/);
  assert.match(phaseLedger, /P2-12b's traversal-budget swap/);
  assert.match(phaseLedger, /P2-13's failure\s+provenance contract/);
  assert.match(phaseLedger, /ir_contract_spans\.rs/);
  assert.match(phaseLedger, /spolvero_feed\.rs/);
  assert.match(phaseLedger, /pre-signoff evidence, not a P2-17 completion/);
  assert.match(phaseLedger, /P2-20 cannot evaluate\s+the exit gate until every P2-1\.\.P2-19/);
  assert.match(phaseLedger, /tick a line only\s+with evidence/);
  assertP2_17MechanicalWitnesses();

  assert.equal(gateItems.length, 12, "the P2 exit gate item count changed");
  assert.deepEqual(
    gateItems.filter((item) => item.checked),
    [],
    "P2 exit gate must remain unticked until P2-20 evaluation records evidence",
  );
  assert.ok(
    gateItems.some((item) => item.title === "IR contract review signed off"),
    "P2-17 must remain an exit-gate line",
  );
  assert.ok(
    gateItems.some(
      (item) => item.title === "Differential lanes green and their retirement condition restated",
    ),
    "P2-20 must continue to gate differential-lane retirement",
  );
  assert.ok(
    gateItems.some(
      (item) =>
        item.title === "Corpus waiver ledger empty and the phase-boundary expansion audit done",
    ),
    "P2-20 must continue to gate the C-16 waiver-ledger review",
  );
}

function assertP2_17MechanicalWitnesses(): void {
  const spanWitness = fs.readFileSync(
    new URL("../../../crates/vize_s1_to_s2/tests/ir_contract_spans.rs", import.meta.url),
    "utf8",
  );
  assert.match(spanWitness, /P2-17 mechanical span gate/);
  assert.match(spanWitness, /owned_folio_spans_resolve_for_optional_corpus/);
  assert.match(spanWitness, /assert_folio_spans_resolve/);

  const schemaWitness = fs.readFileSync(
    new URL("../../../crates/vize_davinci/tests/spolvero_feed.rs", import.meta.url),
    "utf8",
  );
  assert.match(schemaWitness, /consumers_negotiate_schema_version_before_reading_pages/);
  assert.match(schemaWitness, /not read until the version is accepted/);
  assert.match(schemaWitness, /SchemaGateError::VersionMismatch/);
}
