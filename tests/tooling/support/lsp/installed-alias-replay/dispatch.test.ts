import assert from "node:assert/strict";
import { test } from "node:test";
import { settlePublication, settleReply } from "./dispatch.ts";
import {
  InstalledAliasError,
  type Packet,
  type Pending,
  type PublicationWait,
} from "./protocol-types.ts";

function pending(observed: boolean) {
  const replies: Packet[] = [],
    errors: Error[] = [],
    failures: string[] = [];
  const item: Pending = {
    observed,
    outcome: { id: 3, method: "textDocument/definition", status: "pending" },
    timer: setTimeout(() => assert.fail("unsettled request"), 60_000),
    resolve: (value) => replies.push(value),
    reject: (error) => errors.push(error),
  };
  return { item, map: new Map([[3, item]]), replies, errors, failures };
}
test("uncontracted RPC errors retain whole replies while contracted requests fail", () => {
  const packet = {
    jsonrpc: "2.0",
    id: 3,
    error: { code: -32603, message: "definition unavailable", data: { complete: true } },
  };
  for (const observed of [false, true]) {
    const result = pending(observed);
    settleReply(packet, result.map, result.failures);
    assert.equal(result.map.size, 0);
    assert.equal(result.item.outcome.status, "rpc-error");
    assert.equal(result.item.outcome.response, packet);
    assert.equal(result.replies.length, observed ? 1 : 0);
    assert.equal(result.errors.length, observed ? 0 : 1);
    assert.equal(result.failures.length, observed ? 0 : 1);
    if (observed) assert.equal(result.replies[0], packet);
    else assert.equal((result.errors[0] as InstalledAliasError).packet, packet);
  }
});
test("observational mode cannot accept a malformed response", () => {
  const result = pending(true),
    packet = { jsonrpc: "2.0", id: 3, extra: "missing-result-and-error" };
  settleReply(packet, result.map, result.failures);
  assert.equal(result.replies.length, 0);
  assert.equal(result.errors.length, 1);
  assert.equal(result.item.outcome.status, "malformed-response");
  assert.equal((result.errors[0] as InstalledAliasError).packet, packet);
});
test("null response stays complete and an unrelated ID never consumes a pending request", () => {
  const result = pending(false),
    packet = { jsonrpc: "2.0", id: 3, result: null };
  settleReply({ ...packet, id: 4 }, result.map, result.failures);
  assert.equal(result.map.size, 1);
  settleReply(packet, result.map, result.failures);
  assert.deepEqual(result.replies, [packet]);
  assert.deepEqual(result.failures, []);
});
function publication(count: number, after = 0) {
  const sequences: Packet[][] = [],
    errors: Error[] = [],
    failures: string[] = [];
  const wait: PublicationWait = {
    uri: "file:///project/Guard.vue",
    version: 2,
    after,
    count,
    packets: [],
    timer: setTimeout(() => assert.fail("unsettled publication"), 60_000),
    resolve: (packets) => sequences.push(packets),
    reject: (error) => errors.push(error),
  };
  return { wait, waits: [wait], sequences, errors, failures };
}
const empty: Packet = {
  jsonrpc: "2.0",
  method: "textDocument/publishDiagnostics",
  params: { uri: "file:///project/Guard.vue", version: 2, diagnostics: [] },
};
test("one ordered publication deadline retains identical prompt and native packets", () => {
  const result = publication(2),
    timer = result.wait.timer;
  settlePublication(
    { ...empty, params: { uri: "file:///unrelated.vue", version: 2, diagnostics: [] } },
    1,
    result.waits,
    result.failures,
  );
  assert.deepEqual(result.wait.packets, []);
  settlePublication(empty, 2, result.waits, result.failures);
  assert.equal(result.waits.length, 1);
  assert.equal(result.wait.timer, timer);
  assert.deepEqual(result.sequences, []);
  settlePublication(empty, 3, result.waits, result.failures);
  assert.equal(result.waits.length, 0);
  assert.deepEqual(result.sequences, [[empty, empty]]);
  assert.equal(result.sequences[0][0], empty);
  assert.equal(result.sequences[0][1], empty);
});
test("original single-publication waits preserve a complete snapshot while removing matches", () => {
  const first = publication(1),
    second = publication(1),
    waits = [first.wait, second.wait];
  settlePublication(empty, 1, waits, []);
  assert.deepEqual(waits, []);
  assert.deepEqual(first.sequences, [[empty]]);
  assert.deepEqual(second.sequences, [[empty]]);
});
test("pre-transaction notifications and wrong versions cannot satisfy the ordered sequence", () => {
  const result = publication(2, 4);
  settlePublication(empty, 4, result.waits, result.failures);
  settlePublication(
    { ...empty, params: { ...(empty.params as Packet), version: 3 } },
    5,
    result.waits,
    result.failures,
  );
  assert.deepEqual(result.wait.packets, []);
  clearTimeout(result.wait.timer);
});
test("malformed second publication retains the first and rejects the whole sequence", () => {
  const result = publication(2),
    malformed = {
      ...empty,
      params: { uri: "file:///project/Guard.vue", version: 2, diagnostics: null },
    };
  settlePublication(empty, 1, result.waits, result.failures);
  settlePublication(malformed, 2, result.waits, result.failures);
  assert.deepEqual(result.wait.packets, [empty]);
  assert.deepEqual(result.sequences, []);
  assert.equal(result.waits.length, 0);
  assert.equal((result.errors[0] as InstalledAliasError).packet, malformed);
  assert.deepEqual(result.failures, ["malformed diagnostic publication"]);
});
