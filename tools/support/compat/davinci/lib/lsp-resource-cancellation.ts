import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import type { JsonRpcMessage } from "../../../../../tests/tooling/support/lsp/protocol.ts";

export const cancellationCycles = 40;

export const cancellationSamplePolicy = {
  population: "all 40 real stdio workspace/symbol cancellation responses",
  warmupSamplesExcluded: 0,
  outlierSamplesExcluded: 0,
  percentileMethod: "nearest-rank order statistic at sorted[ceil(q * n) - 1]",
} as const;

export type CancellationFrameReceipt = {
  raw_request_id: number;
  wire_order: ["workspace/symbol", "$/cancelRequest", "textDocument/didChange"];
  version: number;
  source_bytes: number;
  source_sha256: string;
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

/** The returned notifications follow the request in the same `stdin.write`. */
export function cancellationFollowing(
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
      wire_order: ["workspace/symbol", "$/cancelRequest", "textDocument/didChange"],
      version,
      source_bytes: Buffer.byteLength(source, "utf8"),
      source_sha256: sourceSha256(source),
    },
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
