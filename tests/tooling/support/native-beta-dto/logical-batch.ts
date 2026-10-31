import assert from "node:assert/strict";
import type { AttachedOwner } from "./beta-attached-route.ts";
import type { SemanticPacket, SourceSupplement } from "./semantic-route.ts";
type Packet = Record<string, unknown>;
const object = (value: unknown): Packet => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Packet;
};
/** The current source pulls Virtual once, then the ordered main/Virtual batch. */
export function verifyLogicalBatchSuffix(
  owner: AttachedOwner,
  packets: readonly SemanticPacket[],
  attachedValue: unknown,
): SourceSupplement | null {
  if (packets.length === 0) return null;
  const attached = object(attachedValue),
    batch = object(attached.materializedBatch);
  assert.equal(packets.length, 55, "52 phase pairs and exactly three source batch pairs required");
  const mainUri = attached.mainUri,
    virtualUri = batch.virtualUri;
  assert.equal(typeof mainUri, "string");
  assert.equal(typeof virtualUri, "string");
  const phasePackets: SemanticPacket[] = [],
    suffix: SemanticPacket[] = [];
  for (const [index, packet] of packets.entries()) {
    if (index < 52) {
      assert.equal(packet.uri, mainUri);
      phasePackets.push(packet);
    } else suffix.push(packet);
  }
  assert.deepEqual(
    suffix.map((row) => row.request.method),
    ["textDocument/diagnostic", "textDocument/diagnostic", "textDocument/diagnostic"],
  );
  assert.deepEqual(
    suffix.map((row) => row.uri),
    [virtualUri, mainUri, virtualUri],
  );
  for (const row of suffix)
    assert.deepEqual(row.request.params, { textDocument: { uri: row.uri } });
  const pull = object(batch.nativePull);
  assert.deepEqual(pull, {
    kind: "full",
    items: [
      {
        range: { start: { line: 0, character: 13 }, end: { line: 0, character: 16 } },
        severity: 1,
        code: 2322,
        source: "ts",
        message: "Type 'string' is not assignable to type 'number'.",
      },
    ],
  });
  assert.deepEqual(suffix[0]!.response.result, pull, "whole initial Virtual pull differs");
  const selected = object(batch.selected);
  assert.deepEqual(Object.keys(selected), ["Ok"]);
  assert.ok(Array.isArray(selected.Ok));
  assert.equal(selected.Ok.length, 2);
  const expectedUris = [mainUri, virtualUri];
  for (const [index, value] of selected.Ok.entries()) {
    assert.ok(Array.isArray(value));
    assert.equal(value.length, 2);
    assert.equal(value[0], expectedUris[index]);
    assert.deepEqual(suffix[index + 1]!.response.result, { kind: "full", items: value[1] });
  }
  assert.deepEqual((selected.Ok[0] as unknown[])[1], []);
  assert.deepEqual((selected.Ok[1] as unknown[])[1], pull.items);
  const client = owner.client.map(object),
    server = owner.server.map(object);
  const indices = suffix.map((row) => client.indexOf(row.request));
  assert.ok(
    client.indexOf(packets[51]!.request) < indices[0]! && indices.every((value) => value >= 0),
  );
  assert.ok(indices[0]! < indices[1]! && indices[1]! < indices[2]!);
  const responses = suffix.map((row) => server.indexOf(row.response));
  assert.ok(responses[0]! >= 0 && responses[0]! < responses[1]! && responses[1]! < responses[2]!);
  const open = (uri: unknown, text: unknown, before: number) => {
    const found = client.flatMap((row, index) =>
      row.method === "textDocument/didOpen" &&
      object(object(row.params).textDocument).uri === uri &&
      index > client.indexOf(packets[51]!.request) &&
      index < before
        ? [{ row, index }]
        : [],
    );
    assert.equal(found.length, 1, "one actual post-phase source document open required");
    const row = found[0]!,
      document = object(object(row.row.params).textDocument);
    assert.deepEqual(Object.keys(row.row).sort(), ["jsonrpc", "method", "params"]);
    assert.deepEqual(Object.keys(document).sort(), ["languageId", "text", "uri", "version"]);
    assert.equal(document.languageId, "typescript");
    assert.equal(document.text, text);
    assert.ok(
      typeof document.version === "number" &&
        Number.isSafeInteger(document.version) &&
        document.version > 0,
    );
    return row;
  };
  const virtualOpen = open(virtualUri, batch.source, indices[0]!);
  const proof = object(batch.storageAuthority);
  assert.ok(Array.isArray(proof.retainedDocuments));
  const support = proof.retainedDocuments
    .map(object)
    .filter((row) => row.uri === attached.supportUri);
  assert.equal(support.length, 1);
  const supportOpen = open(attached.supportUri, support[0]!.text, indices[1]!);
  const shutdown = client.flatMap((row, index) =>
    row.method === "shutdown" ? [{ row, index }] : [],
  );
  assert.equal(shutdown.length, 1);
  assert.ok(shutdown[0]!.index > indices[2]!);
  for (const [index, row] of client.entries())
    if (index > client.indexOf(packets[51]!.request) && index < shutdown[0]!.index)
      if (row.method === "textDocument/didClose")
        assert.ok(
          ![mainUri, virtualUri, attached.supportUri].includes(
            object(object(row.params).textDocument).uri,
          ),
        );
  return {
    phasePackets,
    batchPackets: suffix,
    evidence: {
      schema: "vize.nativeBetaLogical.wholeBatchSuffix.v1",
      nativeOwner: owner.nativeOwner,
      scope:
        "actual initial Virtual pull and ordered whole main/Virtual batch after unchanged52 phase queries; no API semantic credit",
      virtualOpen,
      supportOpen,
      packets: suffix,
      sourceNativePull: batch.nativePull,
      sourceBatch: batch.selected,
    },
  };
}
