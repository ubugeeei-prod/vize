import assert from "node:assert/strict";
import { performance } from "node:perf_hooks";

import type { DualLspProduct, ProductKind } from "./dual-lsp-product.ts";
import { rangeForNeedle, type DualLspFixture, type Range } from "./dual-lsp-fixture.ts";

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

export function asRange(value: unknown): Range {
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
