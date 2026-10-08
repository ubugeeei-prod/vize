import assert from "node:assert/strict";
import type { JsonRpcMessage } from "../../tooling/support/lsp/protocol.ts";

/** Keep the original complete request/response envelope assertions together. */
export function assertQueryFrames(
  rows: Array<Record<string, unknown>>,
  client: JsonRpcMessage[],
  responses: JsonRpcMessage[],
) {
  for (const row of rows) {
    const sent = client.filter((message) => message.id === row.requestId);
    const received = responses.filter((message) => message.id === row.requestId);
    assert.equal(sent.length, 1);
    assert.equal(received.length, 1);
    assert.deepEqual(sent[0], {
      jsonrpc: "2.0",
      id: row.requestId,
      method: row.method,
      params: row.params,
    });
    assert.deepEqual(
      received[0],
      row.response,
      "unknown envelope/error fields and their presence remain whole",
    );
  }
}
