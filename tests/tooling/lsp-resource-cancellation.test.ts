import assert from "node:assert/strict";
import test from "node:test";
import {
  cancellationCycles,
  cancellationFollowing,
  sourceSha256,
  summarizeCancellationDurations,
} from "../../tools/support/compat/davinci/lib/lsp-resource-cancellation.ts";

test("resource cancellation frames retain exact write order, version and source identity", () => {
  const source = "<template><p>日本語</p></template>\n";
  const { messages, receipt } = cancellationFollowing(
    7,
    "file:///workspace/Component.vue",
    12,
    source,
  );

  assert.deepEqual(messages, [
    { jsonrpc: "2.0", method: "$/cancelRequest", params: { id: 7 } },
    {
      jsonrpc: "2.0",
      method: "textDocument/didChange",
      params: {
        textDocument: { uri: "file:///workspace/Component.vue", version: 12 },
        contentChanges: [{ text: source }],
      },
    },
  ]);
  assert.deepEqual(receipt, {
    raw_request_id: 7,
    wire_order: ["workspace/symbol", "$/cancelRequest", "textDocument/didChange"],
    version: 12,
    source_bytes: Buffer.byteLength(source),
    source_sha256: sourceSha256(source),
  });
});

test("resource cancellation statistics use every nearest-rank sample", () => {
  const values = Array.from({ length: cancellationCycles }, (_, index) => index + 1);
  assert.deepEqual(summarizeCancellationDurations(values), {
    samples: 40,
    min_ms: 1,
    p50_ms: 20,
    p95_ms: 38,
    p99_ms: 40,
    max_ms: 40,
    sample_policy: {
      population: "all 40 real stdio workspace/symbol cancellation responses",
      warmupSamplesExcluded: 0,
      outlierSamplesExcluded: 0,
      percentileMethod: "nearest-rank order statistic at sorted[ceil(q * n) - 1]",
    },
  });
  assert.throws(() => summarizeCancellationDurations(values.slice(1)), /complete cancellation/);
  assert.throws(
    () => summarizeCancellationDurations([...values.slice(0, -1), Number.NaN]),
    /finite non-negative/,
  );
});
