import assert from "node:assert/strict";

import { assertNormalizedPath, canonicalJson, sha256 } from "./syntax-evidence.ts";
import { semanticCategories, type SemanticSpan } from "./syntax-semantic-divergence.ts";

export type DivergenceRecord = {
  category: string;
  endColumn: number;
  file: string;
  kind: "false-negative" | "false-positive";
  line: number;
  startColumn: number;
};

export type DocumentedDivergence = DivergenceRecord & {
  issue: number;
  project: string;
  reason: string;
};

export function compareSemanticSpans(
  file: string,
  source: string,
  vize: SemanticSpan[],
  oracle: SemanticSpan[],
) {
  const shared: Array<Omit<DivergenceRecord, "kind">> = [];
  const falsePositives: DivergenceRecord[] = [];
  const falseNegatives: DivergenceRecord[] = [];
  const vizeByLine = groupByLine(vize);
  const oracleByLine = groupByLine(oracle);
  for (const [lineIndex, line] of source.split("\n").entries()) {
    const lineNumber = lineIndex + 1;
    const left = vizeByLine.get(lineNumber) ?? [];
    const right = oracleByLine.get(lineNumber) ?? [];
    const boundaries = new Set([1, line.length + 1]);
    for (const span of [...left, ...right]) {
      boundaries.add(span.startColumn);
      boundaries.add(span.endColumn);
    }
    const ordered = [...boundaries].sort((a, b) => a - b);
    for (let index = 0; index + 1 < ordered.length; index += 1) {
      const startColumn = ordered[index];
      const endColumn = ordered[index + 1];
      if (startColumn === endColumn) continue;
      const leftCategories = categoriesAt(left, startColumn);
      const rightCategories = categoriesAt(right, startColumn);
      for (const category of intersection(leftCategories, rightCategories)) {
        appendRecord(shared, { category, file, line: lineNumber, startColumn, endColumn });
      }
      for (const category of difference(leftCategories, rightCategories)) {
        appendRecord(falsePositives, {
          category,
          file,
          kind: "false-positive",
          line: lineNumber,
          startColumn,
          endColumn,
        });
      }
      for (const category of difference(rightCategories, leftCategories)) {
        appendRecord(falseNegatives, {
          category,
          file,
          kind: "false-negative",
          line: lineNumber,
          startColumn,
          endColumn,
        });
      }
    }
  }
  const classified = { shared, falsePositives, falseNegatives };
  return { ...classified, sha256: sha256(canonicalJson(classified)) };
}

export function applyDocumentedDivergences(
  project: string,
  comparison: ReturnType<typeof compareSemanticSpans>,
  ledger: unknown,
) {
  const entries = validateLedger(ledger).filter((entry) => entry.project === project);
  const falsePositives = [...comparison.falsePositives];
  const falseNegatives = [...comparison.falseNegatives];
  const documented: DocumentedDivergence[] = [];
  for (const entry of entries) {
    const records = entry.kind === "false-positive" ? falsePositives : falseNegatives;
    const index = records.findIndex((record) => recordIdentity(record) === recordIdentity(entry));
    if (index < 0) {
      throw new Error(`stale syntax divergence ledger entry: ${recordIdentity(entry)}`);
    }
    records.splice(index, 1);
    documented.push(entry);
  }
  return { falseNegatives, falsePositives, documented };
}

export function validateLedger(
  value: unknown,
  knownProjectIds?: ReadonlySet<string>,
): DocumentedDivergence[] {
  assert.ok(value != null && typeof value === "object" && !Array.isArray(value));
  const ledger = value as { entries?: unknown; schema?: unknown; version?: unknown };
  assert.equal(
    ledger.schema,
    "vize.fixtureSyntaxDivergenceLedger",
    "unexpected syntax divergence ledger schema",
  );
  assert.equal(ledger.version, 1, "unexpected syntax divergence ledger version");
  const entries = ledger.entries;
  assert.ok(Array.isArray(entries), "syntax divergence ledger entries must be an array");
  const identities = new Set<string>();
  return entries.map((raw, index) => {
    assert.ok(raw != null && typeof raw === "object" && !Array.isArray(raw));
    const entry = raw as DocumentedDivergence;
    assert.deepEqual(Object.keys(entry).sort(), [
      "category",
      "endColumn",
      "file",
      "issue",
      "kind",
      "line",
      "project",
      "reason",
      "startColumn",
    ]);
    assert.match(entry.project, /^[a-z0-9][a-z0-9-]*$/);
    assert.ok(
      knownProjectIds == null || knownProjectIds.has(entry.project),
      `unknown syntax divergence ledger project ${entry.project}`,
    );
    assertNormalizedPath(entry.file, "ledger path");
    assert.ok(semanticCategories.includes(entry.category as (typeof semanticCategories)[number]));
    assert.ok(entry.kind === "false-positive" || entry.kind === "false-negative");
    for (const field of ["line", "startColumn", "endColumn"] as const) {
      assert.ok(Number.isSafeInteger(entry[field]) && entry[field] > 0, `ledger ${index}.${field}`);
    }
    assert.ok(entry.endColumn > entry.startColumn, `ledger ${index} has empty range`);
    assert.ok(Number.isSafeInteger(entry.issue) && entry.issue > 0, `ledger ${index}.issue`);
    assert.ok(entry.reason.replace(/\s+/g, " ").trim().length >= 40, `ledger ${index}.reason`);
    const identity = `${entry.project}\0${recordIdentity(entry)}`;
    assert.ok(!identities.has(identity), `duplicate syntax divergence ledger entry ${identity}`);
    identities.add(identity);
    return { ...entry, reason: entry.reason.replace(/\s+/g, " ").trim() };
  });
}

function appendRecord<
  T extends {
    category: string;
    endColumn: number;
    file: string;
    line: number;
    startColumn: number;
  },
>(records: T[], record: T): void {
  const previous = records.at(-1);
  if (
    previous?.file === record.file &&
    previous.line === record.line &&
    previous.endColumn === record.startColumn &&
    previous.category === record.category
  ) {
    previous.endColumn = record.endColumn;
  } else records.push(record);
}

function categoriesAt(spans: SemanticSpan[], column: number): string[] {
  return (
    spans.find((span) => span.startColumn <= column && column < span.endColumn)?.categories ?? []
  );
}

function groupByLine(spans: SemanticSpan[]): Map<number, SemanticSpan[]> {
  const byLine = new Map<number, SemanticSpan[]>();
  for (const span of spans) {
    const bucket = byLine.get(span.line);
    if (bucket == null) byLine.set(span.line, [span]);
    else bucket.push(span);
  }
  return byLine;
}

function difference(left: string[], right: string[]): string[] {
  const excluded = new Set(right);
  return left.filter((category) => !excluded.has(category));
}

function intersection(left: string[], right: string[]): string[] {
  const included = new Set(right);
  return left.filter((category) => included.has(category));
}

function recordIdentity(record: Omit<DivergenceRecord, "kind"> & { kind?: string }): string {
  return [
    record.kind,
    record.file,
    record.line,
    record.startColumn,
    record.endColumn,
    record.category,
  ].join(":");
}
