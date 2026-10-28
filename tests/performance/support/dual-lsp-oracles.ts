import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { performance } from "node:perf_hooks";

import { processTreeRss, type ProcessRss } from "./process-metrics.ts";
import type { DualLspProduct, ProductKind, ProductResponse } from "./dual-lsp-product.ts";
import {
  compareRange,
  offsetToPosition,
  positionToOffset,
  rangeForNeedle,
  rangeKey,
  wordRanges,
  type DualLspFixture,
  type Range,
} from "./dual-lsp-fixture.ts";

type Diagnostic = {
  code?: unknown;
  message?: unknown;
  range?: unknown;
  severity?: unknown;
};

type DiagnosticPublication = {
  uri?: unknown;
  version?: unknown;
  diagnostics?: unknown;
};

export type ObservedDiagnosticPublication = {
  provider: string;
  receivedAtMs: number;
  params: unknown;
};

export type DiagnosticEvidence = {
  expectedVersion: number;
  provenVersion: number;
  reportedVersion: number | null;
  fingerprint: string;
  publicationLatencyMs: number;
  qualificationMs: number;
  publicationsObserved: number;
  protocolLimitation: string | null;
};

export type OperationSample = {
  totalMs: number;
  timingsMs: Record<string, number>;
  rss: ProductRss;
  oracleReceipt: OperationOracleReceipt;
};

export type OperationOracleReceipt = {
  authoredUri: string;
  completion: { requiredLabel: string; virtualNameCount: number };
  hoverRange: Range;
  definition: Array<{ uri: string; range: Range }>;
  references: Array<{ uri: string; range: Range }>;
  prepareRenameRange: Range;
  rename: {
    uri: string;
    newName: string;
    edits: Array<{ range: Range; newText: string }>;
    appliedTextSha256: string;
  };
  passedOperations: string[];
  sha256: string;
};

export type ProductRss = {
  totalKiB: number;
  processes: number;
  roots: Array<{
    rootPid: number;
    totalKiB: number;
    processes: number;
    members: LinuxProcessIdentity[];
  }>;
};

export type LinuxProcessIdentity = {
  role: "root" | "descendant";
  pid: number;
  parentPid: number;
  birthTicks: string;
  executable: string;
  command: string[];
  commandSha256: string;
  sampledRssKiB: number;
};

const diagnosticMethod = "textDocument/publishDiagnostics";
export const diagnosticQuietMs = 250;
export const diagnosticBarrierCount = 2;
export const diagnosticDeadlineMs = 60_000;

type DiagnosticWaiter = {
  waitForNotification(
    method: string,
    afterSequence: number,
    predicate: (params: unknown) => boolean,
    timeoutMs?: number,
  ): Promise<unknown>;
};

function normalizeMessage(value: unknown): string {
  if (typeof value !== "string") {
    throw new TypeError("diagnostic message must be a string");
  }
  return value.replace(/\s+/g, " ").trim();
}

function asRange(value: unknown): Range {
  assert.ok(value && typeof value === "object", `missing range: ${JSON.stringify(value)}`);
  const range = value as Range;
  for (const position of [range.start, range.end]) {
    assert.ok(position && Number.isInteger(position.line) && Number.isInteger(position.character));
  }
  return {
    start: { line: range.start.line, character: range.start.character },
    end: { line: range.end.line, character: range.end.character },
  };
}

export function expectedDiagnostic(uri: string, source: string, version: 1 | 2): string {
  const range = rangeForNeedle(source, "broken");
  const message =
    version === 1
      ? "Type 'string' is not assignable to type 'number'."
      : "Type 'number' is not assignable to type 'string'.";
  return JSON.stringify({ uri, range, code: "2322", message });
}

function publicationFingerprint(params: unknown): string | null {
  const publication = params as DiagnosticPublication;
  if (typeof publication?.uri !== "string" || !Array.isArray(publication.diagnostics)) return null;
  if (publication.diagnostics.length === 0) return "empty";
  if (publication.diagnostics.length !== 1) return `multiple:${publication.diagnostics.length}`;
  const diagnostic = publication.diagnostics[0] as Diagnostic;
  try {
    return JSON.stringify({
      uri: publication.uri,
      range: asRange(diagnostic.range),
      code: String(diagnostic.code),
      message: normalizeMessage(diagnostic.message),
    });
  } catch {
    return "invalid";
  }
}

export async function waitForExpectedDiagnostic(
  wire: DiagnosticWaiter,
  afterSequence: number,
  expected: string,
): Promise<void> {
  await wire.waitForNotification(
    diagnosticMethod,
    afterSequence,
    (params) => publicationFingerprint(params) === expected,
    diagnosticDeadlineMs,
  );
}

function assertExactPublication(
  kind: ProductKind,
  params: unknown,
  expected: string,
  version: number,
): { reportedVersion: number | null } {
  const publication = params as DiagnosticPublication;
  assert.equal(
    publicationFingerprint(params),
    expected,
    `diagnostic multiset is not exact: ${JSON.stringify(params)}`,
  );
  const diagnostics = publication.diagnostics as Diagnostic[];
  assert.equal(diagnostics[0]?.severity, 1, "TS2322 must remain an error");
  if (kind === "vize") {
    assert.equal(publication.version, version, "Vize must report the exact open-document version");
    return { reportedVersion: version };
  }
  assert.ok(
    publication.version == null || publication.version === version,
    "Vue stack reported a stale version",
  );
  return { reportedVersion: typeof publication.version === "number" ? publication.version : null };
}

export function assertExactProductDiagnostics(
  kind: ProductKind,
  observed: readonly ObservedDiagnosticPublication[],
  expected: string,
  version: number,
): { match: ObservedDiagnosticPublication; reportedVersion: number | null } {
  const nonEmpty = observed.filter((item) => publicationFingerprint(item.params) !== "empty");
  assert.equal(
    nonEmpty.length,
    1,
    `complete product diagnostics were missing, duplicated, stale, or extra: ${JSON.stringify(observed)}`,
  );
  const match = nonEmpty[0];
  const { reportedVersion } = assertExactPublication(kind, match.params, expected, version);

  const terminalByProvider = new Map<string, ObservedDiagnosticPublication>();
  for (const publication of observed) terminalByProvider.set(publication.provider, publication);
  const terminalNonEmpty = [...terminalByProvider.values()].filter(
    (item) => publicationFingerprint(item.params) !== "empty",
  );
  assert.equal(
    terminalNonEmpty.length,
    1,
    `terminal product diagnostics were cleared, stale, or extra: ${JSON.stringify([
      ...terminalByProvider.values(),
    ])}`,
  );
  assert.equal(
    publicationFingerprint(terminalNonEmpty[0]?.params),
    expected,
    `terminal product diagnostic is not exact: ${JSON.stringify(terminalNonEmpty[0])}`,
  );
  return { match, reportedVersion };
}

async function qualifyRevision(
  product: DualLspProduct,
  fixture: DualLspFixture,
  source: string,
  version: 1 | 2,
  action: () => void,
): Promise<DiagnosticEvidence> {
  const wires = product.diagnosticWires.map(({ provider, wire }) => ({
    provider,
    wire,
    cursor: wire.notificationCursor,
  }));
  const expectedWire = product.expectedDiagnosticWire;
  const expectedCursor = expectedWire.notificationCursor;
  const expected = expectedDiagnostic(fixture.uri, source, version);
  const startedAt = performance.now();
  action();
  await waitForExpectedDiagnostic(expectedWire, expectedCursor, expected);
  for (let barrier = 0; barrier < diagnosticBarrierCount; barrier += 1) {
    await product.diagnosticBarrier(fixture.uri);
  }
  await Promise.all(
    wires.map(({ wire, cursor }) =>
      wire.observeQuiet(diagnosticMethod, cursor, diagnosticQuietMs, 5_000),
    ),
  );
  const observed = wires.flatMap(({ provider, wire, cursor }) =>
    wire.notificationsSince(diagnosticMethod, cursor).map((item) => ({
      provider,
      receivedAtMs: item.receivedAtMs,
      params: item.params,
    })),
  );
  const { match, reportedVersion } = assertExactProductDiagnostics(
    product.kind,
    observed,
    expected,
    version,
  );
  const publicationLatencyMs = match.receivedAtMs - startedAt;
  assert.ok(publicationLatencyMs >= 0, "diagnostic publication preceded its document revision");

  return {
    expectedVersion: version,
    provenVersion: version,
    reportedVersion,
    fingerprint: expected,
    publicationLatencyMs,
    qualificationMs: performance.now() - startedAt,
    publicationsObserved: observed.length,
    protocolLimitation:
      product.kind === "vue-language-tools" && reportedVersion == null
        ? "typescript-language-server does not emit PublishDiagnosticsParams.version; exact content provenance proves this revision but is not version-field parity"
        : null,
  };
}

export async function qualifyDocumentRevisions(
  product: DualLspProduct,
  fixture: DualLspFixture,
): Promise<{ v1: DiagnosticEvidence; v2: DiagnosticEvidence }> {
  const v1 = await qualifyRevision(product, fixture, fixture.sourceV1, 1, () => {
    product.open(fixture.uri, fixture.sourceV1, 1);
  });
  const v1Fingerprint = v1.fingerprint;
  const v2Fingerprint = expectedDiagnostic(fixture.uri, fixture.sourceV2, 2);
  assert.notEqual(
    v1Fingerprint,
    v2Fingerprint,
    "diagnostic revision fingerprints must be exclusive",
  );
  const v2 = await qualifyRevision(product, fixture, fixture.sourceV2, 2, () => {
    product.change(fixture.uri, fixture.sourceV2, 2);
  });
  return { v1, v2 };
}

function completionItems(result: unknown): Array<Record<string, unknown>> {
  if (Array.isArray(result)) return result;
  if (
    result &&
    typeof result === "object" &&
    Array.isArray((result as { items?: unknown }).items)
  ) {
    return (result as { items: Array<Record<string, unknown>> }).items;
  }
  return [];
}

function hoverText(result: unknown): string {
  if (result == null || typeof result !== "object") return "";
  const contents = (result as { contents?: unknown }).contents;
  if (typeof contents === "string") return contents;
  if (Array.isArray(contents)) return contents.map(hoverTextPart).join("\n");
  return hoverTextPart(contents);
}

function hoverTextPart(value: unknown): string {
  if (typeof value === "string") return value;
  if (value && typeof value === "object" && "value" in value)
    return String((value as { value: unknown }).value);
  return "";
}

export function assertAuthoredHovers(hovers: readonly unknown[], target: Range): void {
  assert.ok(hovers.length > 0, "hover was missing");
  assert.ok(
    hovers.some(
      (item) => /message/.test(hoverText(item)) && /string|Template binding/.test(hoverText(item)),
    ),
    JSON.stringify(hovers),
  );
  for (const item of hovers) {
    assert.ok(item && typeof item === "object", "hover result must be an object");
    const range = (item as { range?: unknown }).range;
    assert.ok(range != null, "hover omitted its authored range");
    assert.deepEqual(asRange(range), target, "hover escaped the authored token");
  }
}

type Location = { uri: string; range: Range };

export function authoredLocations(result: unknown): Location[] {
  const values = result == null ? [] : Array.isArray(result) ? result : [result];
  return values.map((value) => {
    if (value == null || typeof value !== "object") {
      throw new TypeError(`location must be an object: ${JSON.stringify(value)}`);
    }
    const item = value as Record<string, unknown>;
    const uri = item.targetUri ?? item.uri;
    const range = item.targetSelectionRange ?? item.targetRange ?? item.range;
    if (typeof uri !== "string" || uri.length === 0) {
      throw new TypeError(`location lacks an authored URI: ${JSON.stringify(value)}`);
    }
    return { uri, range: asRange(range) };
  });
}

function dedupeLocations(values: Location[]): Location[] {
  const byKey = new Map(values.map((item) => [`${item.uri}:${rangeKey(item.range)}`, item]));
  return [...byKey.values()].sort(
    (left, right) => left.uri.localeCompare(right.uri) || compareRange(left.range, right.range),
  );
}

type TextEdit = { range: Range; newText: string };

function workspaceEdits(result: unknown, expectedUri: string): TextEdit[] {
  if (result == null) return [];
  const edit = result as {
    changes?: Record<string, TextEdit[]>;
    documentChanges?: Array<{ textDocument?: { uri?: string }; edits?: TextEdit[] }>;
  };
  const entries: Array<[string, TextEdit[]]> = Object.entries(edit.changes ?? {});
  for (const change of edit.documentChanges ?? []) {
    assert.ok(
      change.textDocument?.uri && Array.isArray(change.edits),
      `unsupported workspace edit: ${JSON.stringify(change)}`,
    );
    entries.push([change.textDocument.uri, change.edits]);
  }
  const resultEdits: TextEdit[] = [];
  for (const [uri, edits] of entries) {
    assert.equal(uri, expectedUri, `rename escaped the authored document: ${uri}`);
    for (const item of edits) {
      assert.equal(typeof item.newText, "string");
      resultEdits.push({ range: asRange(item.range), newText: item.newText });
    }
  }
  return resultEdits;
}

function applyEdits(source: string, edits: TextEdit[]): string {
  const spans = edits
    .map((edit) => ({
      start: positionToOffset(source, edit.range.start),
      end: positionToOffset(source, edit.range.end),
      newText: edit.newText,
    }))
    .sort((left, right) => right.start - left.start || right.end - left.end);
  let result = source;
  let previousStart = source.length + 1;
  for (const edit of spans) {
    assert.ok(edit.end <= previousStart, "rename edits overlap");
    result = result.slice(0, edit.start) + edit.newText + result.slice(edit.end);
    previousStart = edit.start;
  }
  return result;
}

function flatten(responses: ProductResponse[]): unknown[] {
  return responses.map((response) => response.result).filter((result) => result != null);
}

async function timed<T>(operation: () => Promise<T>): Promise<{ elapsedMs: number; value: T }> {
  const startedAt = performance.now();
  const value = await operation();
  return { elapsedMs: performance.now() - startedAt, value };
}

function sampleRss(product: DualLspProduct): ProductRss {
  assert.equal(process.platform, "linux", "dual-server process identity receipts require Linux");
  const roots = product.processIds.map((rootPid) => {
    const sample = processTreeRss(rootPid);
    assert.ok(sample, `process-tree RSS unavailable for root ${rootPid}`);
    const members = sample.members.map((member) => linuxProcessIdentity(member, rootPid));
    assert.equal(
      members.length,
      sample.processes,
      `process-tree ${rootPid} identity count drifted`,
    );
    assert.equal(
      members.filter((member) => member.role === "root").length,
      1,
      `process-tree ${rootPid} lost its root identity`,
    );
    return {
      rootPid,
      totalKiB: sample.totalKiB,
      processes: sample.processes,
      members,
    };
  });
  return {
    totalKiB: roots.reduce((sum, item) => sum + item.totalKiB, 0),
    processes: roots.reduce((sum, item) => sum + item.processes, 0),
    roots,
  };
}

function procIdentity(pid: number): { parentPid: number; birthTicks: string } {
  const stat = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
  const fields = stat
    .slice(stat.lastIndexOf(") ") + 2)
    .trim()
    .split(/\s+/u);
  assert.match(fields[1], /^\d+$/u);
  assert.match(fields[19], /^\d+$/u);
  return { parentPid: Number(fields[1]), birthTicks: fields[19] };
}

export function linuxProcessIdentity(member: ProcessRss, rootPid: number): LinuxProcessIdentity {
  const before = procIdentity(member.pid);
  assert.equal(
    before.parentPid,
    member.parentPid,
    `process ${member.pid} changed parent during RSS custody`,
  );
  const executable = fs.realpathSync(`/proc/${member.pid}/exe`);
  const command = fs
    .readFileSync(`/proc/${member.pid}/cmdline`)
    .toString("utf8")
    .split("\0")
    .filter((entry) => entry !== "");
  assert.ok(command.length > 0, `process ${member.pid} has no command identity`);
  const after = procIdentity(member.pid);
  assert.deepEqual(after, before, `process ${member.pid} identity changed during RSS custody`);
  return {
    role: member.pid === rootPid ? "root" : "descendant",
    pid: member.pid,
    parentPid: member.parentPid,
    birthTicks: before.birthTicks,
    executable,
    command,
    commandSha256: createHash("sha256").update(command.join("\0")).digest("hex"),
    sampledRssKiB: member.rssKiB,
  };
}

export async function runQualifiedOperationPacket(
  product: DualLspProduct,
  fixture: DualLspFixture,
): Promise<OperationSample> {
  const uri = fixture.uri;
  const source = fixture.sourceV2;
  const ranges = wordRanges(source, "message");
  assert.equal(ranges.length, 3, "fixture must contain declaration and exactly two authored uses");
  const declaration = ranges[0];
  const target = ranges[1];
  const position = offsetToPosition(source, positionToOffset(source, target.start) + 4);
  const timingsMs: Record<string, number> = {};

  const completion = await timed(() =>
    product.requestAll("textDocument/completion", { textDocument: { uri }, position }),
  );
  timingsMs.completion = completion.elapsedMs;
  const completionLabels = new Set(
    flatten(completion.value)
      .flatMap(completionItems)
      .map((item) => String(item.label)),
  );
  assert.ok(
    completionLabels.has("message"),
    `completion omitted message: ${[...completionLabels].join(", ")}`,
  );
  assert.equal(
    [...completionLabels].some((label) => label.startsWith("__VLS_")),
    false,
    "virtual names leaked",
  );

  const hover = await timed(() =>
    product.requestAll("textDocument/hover", { textDocument: { uri }, position }),
  );
  timingsMs.hover = hover.elapsedMs;
  const hovers = flatten(hover.value);
  assertAuthoredHovers(hovers, target);

  const definition = await timed(() =>
    product.requestAll("textDocument/definition", { textDocument: { uri }, position }),
  );
  timingsMs.definition = definition.elapsedMs;
  const definitionLocations = dedupeLocations(flatten(definition.value).flatMap(authoredLocations));
  assert.deepEqual(definitionLocations, [{ uri, range: declaration }]);

  const references = await timed(() =>
    product.requestAll("textDocument/references", {
      textDocument: { uri },
      position,
      context: { includeDeclaration: true },
    }),
  );
  timingsMs.references = references.elapsedMs;
  const referenceLocations = dedupeLocations(flatten(references.value).flatMap(authoredLocations));
  assert.deepEqual(
    referenceLocations,
    ranges.map((range) => ({ uri, range })),
  );

  const prepare = await timed(() =>
    product.requestAll("textDocument/prepareRename", { textDocument: { uri }, position }),
  );
  timingsMs.prepareRename = prepare.elapsedMs;
  const preparedRanges = flatten(prepare.value).map((value) =>
    asRange((value as { range?: unknown }).range ?? value),
  );
  assert.ok(preparedRanges.length > 0, "prepareRename was missing");
  assert.ok(
    preparedRanges.every((range) => rangeKey(range) === rangeKey(target)),
    JSON.stringify(preparedRanges),
  );

  const rename = await timed(() =>
    product.requestAll("textDocument/rename", {
      textDocument: { uri },
      position,
      newName: "title",
    }),
  );
  timingsMs.rename = rename.elapsedMs;
  const edits = flatten(rename.value).flatMap((value) => workspaceEdits(value, uri));
  const uniqueEdits = [
    ...new Map(edits.map((edit) => [`${rangeKey(edit.range)}:${edit.newText}`, edit])).values(),
  ].sort((left, right) => compareRange(left.range, right.range));
  assert.deepEqual(
    uniqueEdits.map((edit) => edit.range),
    ranges,
  );
  assert.ok(uniqueEdits.every((edit) => edit.newText === "title"));
  const appliedText = applyEdits(source, uniqueEdits);
  assert.equal(appliedText, source.replaceAll(/\bmessage\b/g, "title"));

  const oraclePayload = {
    authoredUri: uri,
    completion: { requiredLabel: "message", virtualNameCount: 0 },
    hoverRange: target,
    definition: definitionLocations,
    references: referenceLocations,
    prepareRenameRange: target,
    rename: {
      uri,
      newName: "title",
      edits: uniqueEdits,
      appliedTextSha256: createHash("sha256").update(appliedText).digest("hex"),
    },
    passedOperations: [
      "completion",
      "hover",
      "definition",
      "references",
      "prepareRename",
      "rename",
    ],
  };
  await product.settleBackgroundWork();

  return {
    totalMs: Object.values(timingsMs).reduce((sum, value) => sum + value, 0),
    timingsMs,
    rss: sampleRss(product),
    oracleReceipt: {
      ...oraclePayload,
      sha256: createHash("sha256").update(JSON.stringify(oraclePayload)).digest("hex"),
    },
  };
}
