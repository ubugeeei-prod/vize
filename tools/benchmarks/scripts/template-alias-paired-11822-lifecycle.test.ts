import assert from "node:assert/strict";
import { test } from "node:test";
import { qualifyLifecycle } from "./template-alias-paired-11822-protocol.ts";
import type { ProtocolMessage } from "./template-alias-paired-11822-types.ts";

void test("whole protocol qualification rejects missing initialization, project opens, duplicate and unordered responses", () => {
  const requests = [
    { id: 1, method: "initialize", params: { positionEncodings: ["utf-8"] } },
    { id: 2, method: "openProject", params: { projectHandle: "p" } },
    { id: 3, method: "transform", params: { projectHandle: "p", fileName: "/authored.vue" } },
  ] as unknown as ProtocolMessage[];
  const responses = [
    { jsonrpc: "2.0", id: 1, result: { positionEncoding: "utf-8" } },
    { jsonrpc: "2.0", id: 2, result: { configIdentity: "c", optionDiagnostics: [] } },
    { jsonrpc: "2.0", id: 3, result: { text: "authored projection" } },
  ] as unknown as ProtocolMessage[];
  assert.deepEqual(qualifyLifecycle(requests, responses), { requests: 3, transforms: 1 });
  assert.throws(() => qualifyLifecycle(requests.slice(1), responses.slice(1)));
  assert.throws(() => qualifyLifecycle([requests[0], requests[2]], [responses[0], responses[2]]));
  assert.throws(() => qualifyLifecycle(requests, [responses[0], responses[2], responses[1]]));
  assert.throws(() => qualifyLifecycle([...requests, requests[2]], [...responses, responses[2]]));
  assert.throws(() =>
    qualifyLifecycle(requests, [responses[0], responses[1], { id: 3, error: {} }]),
  );
});
