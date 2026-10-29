import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import type {
  JsonRpcMessage,
  PublishDiagnosticsParams,
} from "../../../../../tests/tooling/support/lsp/protocol.ts";
import type { LspSession } from "../../../../../tests/tooling/support/lsp/session.ts";
import { isRecord } from "../../../json-value.ts";
import { documentUri } from "./lsp-resource-workspace.ts";

export const cancellationCycles = 40;

export const cancellationSamplePolicy = {
  population: "all 40 admitted real stdio textDocument/hover cancellation responses",
  warmupSamplesExcluded: 0,
  outlierSamplesExcluded: 0,
  percentileMethod: "nearest-rank order statistic at sorted[ceil(q * n) - 1]",
} as const;

export type CancellationFrameReceipt = {
  raw_request_id: number;
  wire_order: ["$/cancelRequest", "textDocument/didChange"];
  version: number;
  source_bytes: number;
  source_sha256: string;
  source_utf8_base64: string;
};

export type CancellationPublication = {
  received_order: number;
  received_ns: string;
  uri: string;
  version: number | null;
  diagnostics: unknown[];
  diagnostics_sha256: string;
};

export type CancellationDurationStats = {
  samples: number;
  min_ms: number;
  p50_ms: number;
  p95_ms: number;
  p99_ms: number;
  max_ms: number;
  sample_policy: typeof cancellationSamplePolicy;
};

export const sourceSha256 = (source: string): string =>
  createHash("sha256").update(source).digest("hex");

export function isCancellationPublication(
  value: unknown,
): value is PublishDiagnosticsParams & object {
  return isRecord(value) && Array.isArray(value.diagnostics) && value.diagnostics.every(isRecord);
}

/** Retain every opened child/importer diagnostics payload in the measured window. */
export function recordCancellationPublications(session: LspSession, workspace: string) {
  const trackedUris = new Set([documentUri(workspace, 0), documentUri(workspace, 1)]);
  const rows: CancellationPublication[] = [];
  const receivedOrders = new WeakMap<object, number>();
  let stopped = false;
  const observer = (method: string, value: unknown): void => {
    if (method !== "textDocument/publishDiagnostics" || !isCancellationPublication(value)) return;
    if (!trackedUris.has(value.uri)) return;
    const receivedOrder = rows.length + 1;
    receivedOrders.set(value, receivedOrder);
    const diagnostics = JSON.parse(JSON.stringify(value.diagnostics)) as unknown[];
    rows.push({
      received_order: receivedOrder,
      received_ns: process.hrtime.bigint().toString(),
      uri: value.uri,
      version: Number.isSafeInteger(value.version) ? Number(value.version) : null,
      diagnostics,
      diagnostics_sha256: sourceSha256(JSON.stringify(diagnostics)),
    });
  };
  session.notificationObservers.push(observer);
  return {
    count: () => rows.length,
    orderOf: (value: object): number => receivedOrders.get(value) ?? 0,
    since: (index: number): CancellationPublication[] => rows.slice(index),
    stop() {
      if (stopped) return;
      stopped = true;
      const index = session.notificationObservers.indexOf(observer);
      if (index >= 0) session.notificationObservers.splice(index, 1);
    },
  };
}

export type CancellationPublicationRecorder = ReturnType<typeof recordCancellationPublications>;

type RecoveryPublication = {
  received_order: number;
  uri: string;
  version: number;
  diagnostics: unknown[];
};

/** Prove that recovery crossed a fresh child publication and its later importer publication. */
export function diagnosticRecoveryLifecycle(
  afterReceivedOrder: number,
  child: RecoveryPublication,
  parent: RecoveryPublication,
) {
  assert.ok(
    Number.isSafeInteger(afterReceivedOrder) && afterReceivedOrder >= 0,
    "a valid recovery publication boundary is required",
  );
  assert.ok(
    Number.isSafeInteger(child.received_order) && child.received_order > afterReceivedOrder,
    "the clean child diagnostics must publish after the recovery boundary",
  );
  assert.ok(
    Number.isSafeInteger(parent.received_order) && parent.received_order > child.received_order,
    "the clean importer diagnostics must publish after the clean child diagnostics",
  );
  assert.deepEqual(child.diagnostics, [], "the recovery child diagnostics must be clean");
  assert.deepEqual(parent.diagnostics, [], "the recovery importer diagnostics must be clean");
  return {
    after_received_order: afterReceivedOrder,
    child,
    parent,
    child_before_importer: true as const,
  };
}

/** Build the exact cancellation/edit notification pair for one stdio write. */
export function cancellationBatch(
  requestId: number,
  uri: string,
  version: number,
  source: string,
): { messages: JsonRpcMessage[]; receipt: CancellationFrameReceipt } {
  assert.ok(Number.isSafeInteger(requestId) && requestId > 0, "positive request ID required");
  assert.ok(Number.isSafeInteger(version) && version > 0, "positive document version required");
  assert.ok(uri.startsWith("file:"), "a real file URI is required");
  assert.ok(source.length > 0, "a complete document source is required");
  return {
    messages: [
      { jsonrpc: "2.0", method: "$/cancelRequest", params: { id: requestId } },
      {
        jsonrpc: "2.0",
        method: "textDocument/didChange",
        params: {
          textDocument: { uri, version },
          contentChanges: [{ text: source }],
        },
      },
    ],
    receipt: {
      raw_request_id: requestId,
      wire_order: ["$/cancelRequest", "textDocument/didChange"],
      version,
      source_bytes: Buffer.byteLength(source, "utf8"),
      source_sha256: sourceSha256(source),
      source_utf8_base64: Buffer.from(source).toString("base64"),
    },
  };
}

/**
 * Prove that every tracked opened-document publication belongs to a sent
 * source revision, never goes backwards within its URI, and stops clean.
 * `finalFenceCount` is captured when recovery sees the final child reply;
 * equality below rejects every late tracked publication during the quiet
 * window.
 */
export function auditCancellationPublications(
  publications: readonly CancellationPublication[],
  knownDocuments: ReadonlyMap<string, ReadonlyMap<number, string>>,
  finalChild: { uri: string; version: number },
  finalFenceCount: number,
) {
  assert.ok(publications.length > 0, "the cancellation window must publish tracked diagnostics");
  assert.equal(
    publications.length,
    finalFenceCount,
    "no tracked diagnostics may publish after the final clean recovery fence",
  );
  const previousVersions = new Map<string, number>();
  const tails = new Map<string, CancellationPublication & { version: number }>();
  const rows = publications.map((publication) => {
    assert.ok(
      Number.isSafeInteger(publication.version) && Number(publication.version) > 0,
      "cancellation-window diagnostics must carry a positive version",
    );
    const version = Number(publication.version);
    const sourceHash = knownDocuments.get(publication.uri)?.get(version);
    assert.ok(
      sourceHash,
      `diagnostics ${publication.uri}@${version} must name a sent source clone`,
    );
    const previous = previousVersions.get(publication.uri) ?? -1;
    assert.ok(
      version >= previous,
      `diagnostic versions for ${publication.uri} must be nondecreasing`,
    );
    previousVersions.set(publication.uri, version);
    const row = { ...publication, version, source_sha256: sourceHash };
    tails.set(publication.uri, row);
    return row;
  });
  const expectedUris = [...knownDocuments.keys()].sort();
  const observedUris = [...tails.keys()].sort();
  assert.deepEqual(
    observedUris,
    expectedUris,
    "every known opened document must publish diagnostics in the cancellation window",
  );
  const tailRows = [...tails.values()].sort((left, right) => left.uri.localeCompare(right.uri));
  for (const tail of tailRows) {
    assert.deepEqual(tail.diagnostics, [], `final diagnostics for ${tail.uri} must be clean`);
  }
  const childTail = tails.get(finalChild.uri);
  assert.ok(childTail, "the final child diagnostics receipt is required");
  assert.equal(
    childTail.version,
    finalChild.version,
    "the final child diagnostics must name the clean revision",
  );
  return {
    observed: rows.length,
    tracked_uris: observedUris,
    versions_by_uri: Object.fromEntries(
      [...previousVersions.keys()]
        .sort()
        .map((uri) => [uri, rows.filter((row) => row.uri === uri).map((row) => row.version)]),
    ),
    nondecreasing_versions_per_uri: true,
    known_document_source_versions: true,
    all_observed_uri_tails_clean: true,
    final_child_revision_clean: true,
    publications_after_final_fence: publications.length - finalFenceCount,
    tails: tailRows,
    rows,
  };
}

/** Summarize the complete population without warmup or outlier filtering. */
export function summarizeCancellationDurations(
  values: readonly number[],
): CancellationDurationStats {
  assert.equal(
    values.length,
    cancellationCycles,
    "the complete cancellation population is required",
  );
  assert.ok(
    values.every((value) => Number.isFinite(value) && value >= 0),
    "cancellation durations must be finite non-negative milliseconds",
  );
  const sorted = [...values].sort((left, right) => left - right);
  const percentile = (quantile: number): number => sorted[Math.ceil(quantile * sorted.length) - 1];
  return {
    samples: sorted.length,
    min_ms: sorted[0],
    p50_ms: percentile(0.5),
    p95_ms: percentile(0.95),
    p99_ms: percentile(0.99),
    max_ms: sorted[sorted.length - 1],
    sample_policy: cancellationSamplePolicy,
  };
}
