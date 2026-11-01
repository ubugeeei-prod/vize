import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { requireAfter4CleanupSource, type After4CleanupSource } from "./after4-cleanup-source.ts";
import type { AttachedOwner } from "./beta-attached-route.ts";
import { verifySemanticPhases, type SemanticPacket } from "./semantic-route.ts";
import type { WatcherContext } from "./stable-watcher-boundary.ts";
type Row = Record<string, unknown>;
const object = (value: unknown): Row => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Row;
};
const id = (value: unknown) => {
  assert.ok(
    typeof value === "string" || (typeof value === "number" && Number.isSafeInteger(value)),
  );
  return value;
};
/** Whole source/config/snapshot vectors for this SDK4 same-owner refresh lineage. */
function configuration(owner: AttachedOwner, attached: Row) {
  const root = attached.root,
    config = root + "/tsconfig.json";
  const parsed = {
    fileNames: [root + "/main.ts", root + "/support.ts"],
    options: {
      module: 99,
      moduleResolution: 100,
      noEmit: true,
      strict: true,
      target: 99,
      types: [],
      configFilePath: config,
    },
    compileOnSave: false,
    raw: JSON.parse(String(attached.config)) as unknown,
    errors: [],
  };
  const client = owner.apiClient.map(object),
    server = owner.apiServer.map(object);
  assert.equal(client.length, 26);
  assert.equal(server.length, 26);
  assert.equal(new Set(client.map((row) => id(row.id))).size, client.length);
  assert.equal(new Set(server.map((row) => id(row.id))).size, server.length);
  const pairs = client.map((request) => {
    const response = server.find((row) => row.id === request.id);
    assert.ok(response);
    assert.equal(request.jsonrpc, "2.0");
    assert.equal(response.jsonrpc, "2.0");
    return { request, response };
  });
  const call = (index: number, method: string, params: unknown, result: unknown) => {
    const pair = pairs[index]!;
    assert.deepEqual(pair.request, { jsonrpc: "2.0", id: pair.request.id, method, params });
    assert.deepEqual(pair.response, { jsonrpc: "2.0", id: pair.request.id, result });
    return pair;
  };
  const seen = new Map<number, number>();
  const snapshot = (index: number, method: string, params: unknown, projects: boolean) => {
    const value = object(pairs[index]!.response.result),
      handle = value.snapshot;
    assert.ok(
      typeof handle === "number" && Number.isSafeInteger(handle) && handle > 0 && !seen.has(handle),
    );
    seen.set(handle, server.indexOf(pairs[index]!.response));
    call(index, method, params, {
      snapshot: handle,
      projects: projects
        ? [
            {
              id: config,
              configFileName: config,
              currentDirectory: root,
              dirty: false,
              parsedCommandLine: parsed,
            },
          ]
        : [],
      operation: {},
    });
    return handle;
  };
  const open = { ensurePrograms: true, openProjects: [config] };
  call(0, "initialize", null, { caseSensitivity: 1, currentDirectory: root });
  call(1, "parseConfigFile", { file: config }, parsed);
  const first = snapshot(2, "createSnapshot", open, true);
  let current = snapshot(3, "updateSnapshot", { changes: open, snapshot: first }, false);
  assert.deepEqual(pairs[4]!.request, {
    jsonrpc: "2.0",
    id: pairs[4]!.request.id,
    method: "describeCapabilities",
    params: null,
  });
  assert.deepEqual(pairs[4]!.response, {
    jsonrpc: "2.0",
    id: pairs[4]!.request.id,
    error: {
      code: -32603,
      message: 'api: invalid request: unknown API method "describeCapabilities"',
    },
  });
  call(5, "release", { handle: String(first), snapshot: first }, true);
  assert.ok(seen.get(current)! < server.indexOf(pairs[5]!.response));
  // The c75 source's retained mutations and final two partial closes fork ten
  // new lineages. Each releases the previous handle without closing the owner.
  for (let index = 6; index < 26; index += 2) {
    const next = snapshot(index, "createSnapshot", open, true);
    call(index + 1, "release", { handle: String(current), snapshot: current }, true);
    assert.ok(seen.get(next)! < server.indexOf(pairs[index + 1]!.response));
    current = next;
  }
  return pairs;
}
export function cleanupReadinessSymbols(mainUri: unknown) {
  return [
    {
      name: "checked",
      kind: 13,
      location: {
        uri: mainUri,
        range: { start: { line: 2, character: 13 }, end: { line: 2, character: 36 } },
      },
    },
    {
      name: "echo",
      kind: 13,
      location: {
        uri: mainUri,
        range: { start: { line: 3, character: 13 }, end: { line: 3, character: 25 } },
      },
    },
  ];
}
/** This follows the original observer's explicit closes after its complete batch. */
export function verifyLogicalBeta4Cleanup(
  owner: AttachedOwner,
  phases: readonly SemanticPacket[],
  suffix: readonly SemanticPacket[],
  attachedValue: unknown,
  authority: After4CleanupSource,
  context: WatcherContext,
  sourceRows: readonly unknown[],
) {
  requireAfter4CleanupSource(authority, attachedValue, sourceRows, context);
  verifySemanticPhases(phases, sourceRows.map(object));
  const attached = object(attachedValue),
    batch = object(attached.materializedBatch),
    proof = object(batch.storageAuthority);
  assert.equal(proof.mode, "logical-editor-revision");
  assert.equal(proof.schema, 2);
  const main = attached.mainUri,
    virtual = batch.virtualUri,
    support = attached.supportUri;
  assert.equal(fileURLToPath(String(main)), String(attached.root) + "/main.ts");
  const apiPairs = configuration(owner, attached);
  const client = owner.client.map(object),
    server = owner.server.map(object),
    cutoff = client.indexOf(suffix[2]!.request);
  assert.ok(cutoff >= 0);
  assert.equal(suffix.length, 3);
  const shutdowns = client.filter((row) => row.method === "shutdown");
  assert.equal(shutdowns.length, 1);
  for (const [index, row] of client.entries())
    if (
      index > client.indexOf(phases[51]!.request) &&
      index <= cutoff &&
      row.method === "textDocument/didClose"
    )
      assert.ok(![main, support, virtual].includes(object(object(row.params).textDocument).uri));
  const tail = client.slice(cutoff + 1);
  assert.equal(tail.length, 5);
  for (const [offset, uri] of [
    [0, virtual],
    [2, support],
  ] as const) {
    assert.deepEqual(tail[offset], {
      jsonrpc: "2.0",
      method: "textDocument/didClose",
      params: { textDocument: { uri } },
    });
    const request = tail[offset + 1]!;
    id(request.id);
    assert.deepEqual(request, {
      jsonrpc: "2.0",
      id: request.id,
      method: "textDocument/documentSymbol",
      params: { textDocument: { uri: main } },
    });
    const responses = server.filter((row) => row.id === request.id && !("method" in row));
    assert.equal(responses.length, 1);
    assert.deepEqual(responses[0], {
      jsonrpc: "2.0",
      id: request.id,
      result: cleanupReadinessSymbols(main),
    });
  }
  const shutdown = tail[4]!;
  id(shutdown.id);
  assert.deepEqual(shutdown, { jsonrpc: "2.0", id: shutdown.id, method: "shutdown" });
  const response = server.filter((row) => row.id === shutdown.id && !("method" in row));
  assert.equal(response.length, 1);
  assert.deepEqual(response[0], { jsonrpc: "2.0", id: shutdown.id, result: null });
  const serverPosition = (request: Row) =>
    server.findIndex((row) => row.id === request.id && !("method" in row));
  assert.ok(
    serverPosition(suffix[2]!.request) < serverPosition(tail[1]!) &&
      serverPosition(tail[1]!) < serverPosition(tail[3]!) &&
      serverPosition(tail[3]!) < serverPosition(shutdown),
  );
  return {
    schema: "vize.nativeBetaLogical.sdk4SourceCleanup.v1",
    nativeOwner: owner.nativeOwner,
    source: authority.source,
    observerSha256: authority.observerSha256,
    sourceCompilerAuthority: authority,
    apiPairs,
    apiClient: owner.apiClient,
    apiServer: owner.apiServer,
    cutoff: suffix[2],
    packets: tail,
    responses: [tail[1]!, tail[3]!, shutdown].map((request) => server[serverPosition(request)]),
    scope:
      "exact source-authored two-close cleanup after complete55 results; source/compiler/RPC proof only, no EOF or native qualification",
  };
}
