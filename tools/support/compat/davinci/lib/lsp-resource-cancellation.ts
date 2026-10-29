import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import type { JsonRpcMessage } from "../../../../../tests/tooling/support/lsp/protocol.ts";

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
 * Prove that every child publication observed after the baseline belongs to a
 * sent source revision, never goes backwards, and stops at the recovered clean
 * revision. `finalFenceCount` is captured when recovery sees that clean reply;
 * equality below rejects every late publication during the following quiet
 * window.
 */
export function auditCancellationPublications(
  publications: readonly CancellationPublication[],
  knownVersions: ReadonlyMap<number, string>,
  finalVersion: number,
  finalFenceCount: number,
) {
  assert.ok(publications.length > 0, "the cancellation window must publish final diagnostics");
  assert.equal(
    publications.length,
    finalFenceCount,
    "no child diagnostics may publish after the final clean recovery fence",
  );
  let previousVersion = -1;
  const rows = publications.map((publication) => {
    assert.ok(
      Number.isSafeInteger(publication.version) && Number(publication.version) > 0,
      "cancellation-window diagnostics must carry a positive version",
    );
    const version = Number(publication.version);
    const sourceHash = knownVersions.get(version);
    assert.ok(sourceHash, `diagnostics version ${version} must name a sent source clone`);
    assert.ok(version >= previousVersion, "diagnostic versions must be nondecreasing");
    previousVersion = version;
    return { ...publication, version, source_sha256: sourceHash };
  });
  const tail = rows.at(-1);
  assert.ok(tail, "final diagnostics receipt is required");
  assert.equal(tail.version, finalVersion, "the final diagnostics must name the clean revision");
  assert.deepEqual(tail.diagnostics, [], "the final diagnostics tail must be clean");
  return {
    observed: rows.length,
    versions: rows.map((row) => row.version),
    nondecreasing_versions: true,
    known_source_versions: true,
    final_clean_tail: true,
    publications_after_final_fence: publications.length - finalFenceCount,
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
