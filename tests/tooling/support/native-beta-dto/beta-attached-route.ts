import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import { verifyNativeRelations, type WatcherContext } from "./stable-watcher-boundary.ts";
import type { Receipt } from "./audit-wire-types.ts";

type Packet = Record<string, unknown>;
export interface AttachedOwner {
  nativeOwner: number;
  client: readonly unknown[];
  server: readonly unknown[];
  apiClient: readonly unknown[];
  apiServer: readonly unknown[];
}
const configurationMethods = new Set([
  "initialize",
  "parseConfigFile",
  "createSnapshot",
  "updateSnapshot",
  "describeCapabilities",
  "release",
]);
const semanticMethods = [
  "textDocument/diagnostic",
  "textDocument/hover",
  "textDocument/references",
  "textDocument/definition",
  "textDocument/completion",
  "textDocument/rename",
];
export const storageObserverPath =
  "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta/storage_authority.rs";
export const storageObserverSha256 =
  "4754d721b81e69ea80667d2085585e08697d669c3369ad7c5c698f42f6a75da4";
export const commonObserverSha256 =
  "f9fe37e13f4a566cc01daadfc48ebb96b1842d6f3215a4020312af41bcbb97e1";
const storageScope =
  "source getter and materialized file reads before close; no root inferred from observed RPC";
const mainText =
  '/*🧭é日本語\u2028\u2029*/\r\nimport { value } from "./support";\r\nexport const checked: number = value;\r\nexport const echo = value;\r\n';
const supportText = "export const value = 1;\r\n";
const virtualText = 'export const bad: number = "x";\r\n';
const hash = (value: string) => createHash("sha256").update(value).digest("hex");
function object(value: unknown): Packet {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Packet;
}
function closed(value: unknown, keys: string[]) {
  const row = object(value);
  assert.deepEqual(Object.keys(row).sort(), keys.sort(), "closed source ownership fields required");
  return row;
}
function absolute(value: unknown) {
  assert.equal(typeof value, "string");
  const text = value as string;
  assert.ok(path.posix.isAbsolute(text) && path.posix.normalize(text) === text);
  assert.ok(!/[\r\n\0]/u.test(text));
  return text;
}
export function attachedOwners(receipts: readonly Receipt[]): AttachedOwner[] {
  return receipts.slice(1).map((receipt) => {
    const api = receipt.streams.find((stream) => stream.fd >= 3 && stream.client && stream.server);
    return {
      nativeOwner: receipt.nativeOwner,
      client: receipt.streams.find((stream) => stream.fd === 0)!.client!.packets,
      server: receipt.streams.find((stream) => stream.fd === 1)!.server!.packets,
      apiClient: api?.client?.packets ?? [],
      apiServer: api?.server?.packets ?? [],
    };
  });
}
export function verifyObserverDependencies(
  compiled: unknown,
  prepared: unknown,
  compiledObserver: unknown,
  preparedObserver: unknown,
) {
  assert.equal(compiledObserver, commonObserverSha256);
  const observer = closed(preparedObserver, ["path", "sha256"]);
  assert.equal(
    observer.path,
    storageObserverPath.replace("/native_beta/storage_authority.rs", "/native_beta.rs"),
  );
  assert.equal(observer.sha256, commonObserverSha256);
  assert.ok(Array.isArray(compiled) && Array.isArray(prepared));
  assert.equal(compiled.length, 1, "exactly one separately hashed common observer helper");
  assert.deepEqual(compiled, prepared, "compiled and copied observer dependency bytes differ");
  const row = closed(compiled[0], ["path", "sha256"]);
  assert.equal(row.path, storageObserverPath);
  assert.equal(row.sha256, storageObserverSha256);
}

/** Only the reviewed source getter and physical file reads grant current mirror identities. */
export function verifyAfterAttachedStorage(
  owners: readonly AttachedOwner[],
  context: WatcherContext,
  expectedSource: string,
  attachedValue: unknown,
) {
  assert.equal(context.role, "after");
  assert.match(expectedSource, /^[a-f0-9]{40}$/u);
  assert.equal(context.source, expectedSource);
  assert.ok(["7.0.2", "7.1.0-beta"].includes(context.sdkVersion));
  assert.equal(
    context.sdkSha256,
    context.sdkVersion === "7.0.2"
      ? "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4"
      : "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
  );
  const attached = object(attachedValue),
    batch = object(attached.materializedBatch);
  const proof = closed(batch.storageAuthority, [
    "schema",
    "scope",
    "authoredRoot",
    "clientProjectRoot",
    "materializedRoot",
    "canonicalMaterializedRoot",
    "rootIsDirectory",
    "rootIsSymlink",
    "documents",
    "config",
  ]);
  const authored = absolute(attached.root),
    root = absolute(proof.materializedRoot);
  assert.ok(context.projectRoots.includes(authored));
  assert.equal(proof.schema, 1);
  assert.equal(proof.scope, storageScope);
  assert.equal(proof.authoredRoot, authored);
  assert.equal(proof.clientProjectRoot, authored);
  assert.equal(proof.canonicalMaterializedRoot, root);
  assert.equal(proof.rootIsDirectory, true);
  assert.equal(proof.rootIsSymlink, false);
  assert.equal(path.posix.basename(root), "project");
  assert.match(path.posix.basename(path.posix.dirname(root)), /^vize-corsa-overlay-[A-Za-z0-9]+$/u);
  assert.ok(!root.split("/").some((part) => part.toLowerCase() === "node_modules"));
  assert.ok(
    root !== authored && !root.startsWith(authored + "/") && !authored.startsWith(root + "/"),
  );
  assert.equal(attached.mainUri, pathToFileURL(authored + "/main.ts").href);
  assert.equal(attached.supportUri, pathToFileURL(authored + "/support.ts").href);
  assert.equal(batch.virtualUri, pathToFileURL(authored + "/Virtual.vue.ts").href);
  assert.equal(batch.mappedUri, pathToFileURL(root + "/Virtual.vue.ts").href);
  assert.equal(batch.source, virtualText);
  assert.ok(Array.isArray(attached.rows) && attached.rows.length === 4);
  for (const phase of attached.rows) assert.equal(object(phase).mainText, mainText);
  assert.equal(object(attached.rows[0]).supportOverlay, supportText);
  assert.equal(object(attached.rows[3]).supportOverlay, supportText);
  assert.ok(Array.isArray(proof.documents));
  assert.equal(proof.documents.length, 3);
  const documents = [
    ["main.ts", mainText],
    ["support.ts", supportText],
    ["Virtual.vue.ts", virtualText],
  ];
  const uris = documents.flatMap(([name, text], index) => {
    const row = closed((proof.documents as unknown[])[index], [
      "externalUri",
      "mappedUri",
      "relativePath",
      "bytes",
      "sha256",
    ]);
    const external = pathToFileURL(authored + "/" + name).href,
      mapped = pathToFileURL(root + "/" + name).href;
    assert.deepEqual(row, {
      externalUri: external,
      mappedUri: mapped,
      relativePath: name,
      bytes: Buffer.byteLength(text!),
      sha256: hash(text!),
    });
    return [external, mapped];
  });
  const config = closed(proof.config, ["path", "bytes", "sha256", "text"]);
  assert.equal(config.path, root + "/tsconfig.json");
  assert.equal(typeof config.text, "string");
  assert.equal(config.bytes, Buffer.byteLength(config.text as string));
  assert.equal(config.sha256, hash(config.text as string));
  assert.deepEqual(JSON.parse(config.text as string), {
    extends: authored + "/tsconfig.json",
    compilerOptions: { allowImportingTsExtensions: true, noEmit: true },
    include: ["**/*"],
    exclude: [],
  });
  assert.ok(owners.length > 0);
  const ownerIds = owners.map((owner) => owner.nativeOwner);
  assert.ok(ownerIds.every((owner) => Number.isSafeInteger(owner) && owner > 0));
  assert.equal(new Set(ownerIds).size, owners.length);
  for (const owner of owners) {
    const client = owner.client.map(object),
      server = owner.server.map(object);
    const initialized = client.filter((row) => row.method === "initialize");
    assert.equal(initialized.length, 1, "one source-bound native owner initialization");
    const params = object(initialized[0]!.params),
      selected = params.rootPath;
    assert.ok(
      selected === authored || selected === root,
      "native owner root is not source-selected",
    );
    assert.equal(params.rootUri, pathToFileURL(selected as string).href);
    const initializedReplies = server.filter(
      (row) => "id" in row && !("method" in row) && id(row.id) === id(initialized[0]!.id),
    );
    assert.equal(initializedReplies.length, 1);
    assert.equal("error" in initializedReplies[0]!, false);
    assert.deepEqual(object(initializedReplies[0]!.result).serverInfo, {
      name: "typescript-go",
      version: context.sdkVersion,
    });
    for (const request of client) {
      if (
        semanticMethods.includes(String(request.method)) ||
        request.method === "custom/projectInfo"
      ) {
        const uri = object(object(request.params).textDocument).uri;
        assert.ok(
          uris.includes(String(uri)),
          "only exact source-owned document joins are admitted",
        );
        if (request.method === "custom/projectInfo") {
          const replies = server.filter(
            (row) => "id" in row && !("method" in row) && id(row.id) === id(request.id),
          );
          assert.equal(replies.length, 1);
          assert.deepEqual(replies[0]!.result, { configFilePath: selected + "/tsconfig.json" });
        }
      }
    }
  }
  return {
    materializedRoot: root,
    authoredRoot: authored,
    documentUris: uris,
    storageAuthority: proof,
  };
}
function id(value: unknown) {
  assert.ok(
    typeof value === "string" || (typeof value === "number" && Number.isSafeInteger(value)),
  );
  return `${typeof value}:${value}`;
}
function canonical(value: unknown): string {
  if (Array.isArray(value)) return "[" + value.map(canonical).join(",") + "]";
  if (value !== null && typeof value === "object")
    return (
      "{" +
      Object.entries(value)
        .sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0))
        .map(([key, field]) => JSON.stringify(key) + ":" + canonical(field))
        .join(",") +
      "}"
    );
  assert.ok(
    value === null ||
      typeof value === "string" ||
      typeof value === "boolean" ||
      (typeof value === "number" && Number.isFinite(value)),
  );
  return JSON.stringify(value);
}
function sameCompletion(actual: unknown, expected: unknown): boolean {
  const left = object(actual),
    right = object(expected);
  assert.ok(Array.isArray(left.items) && Array.isArray(right.items));
  // Only Go's top-level item order varies. Keep every item field, nested array
  // order, list metadata and duplicate count; retained raw packets stay intact.
  return isDeepStrictEqual(
    { ...left, items: left.items.map(canonical).sort() },
    { ...right, items: right.items.map(canonical).sort() },
  );
}

/** Configuration attachments do not attest which transport served semantics. */
export function verifyBetaAttachedRoute(
  owners: readonly AttachedOwner[],
  context: WatcherContext,
  expectedSource: string,
  attachedRoot: string,
  sourceRows: readonly unknown[],
  attachedValue: unknown,
) {
  assert.equal(context.role, "after");
  assert.equal(context.sdkVersion, "7.1.0-beta");
  assert.equal(
    context.sdkSha256,
    "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
  );
  assert.match(expectedSource, /^[a-f0-9]{40}$/u);
  assert.equal(context.source, expectedSource);
  assert.ok(context.projectRoots.includes(attachedRoot));
  assert.ok(
    path.posix.isAbsolute(attachedRoot) && path.posix.normalize(attachedRoot) === attachedRoot,
  );
  const storage = verifyAfterAttachedStorage(owners, context, expectedSource, attachedValue);
  assert.equal(storage.authoredRoot, attachedRoot);
  const phases = sourceRows.map(object);
  assert.deepEqual(
    phases.map((row) => row.phase),
    ["valid", "dependency-error", "dependency-repair", "dependency-close"],
  );
  for (const row of phases) {
    assert.equal(row.apiEligible, false);
    assert.equal(object(row.nativePull).kind, "full");
    assert.deepEqual(object(row.selected).complete, object(row.nativePull).items);
  }
  const mainUris = new Set([
    pathToFileURL(attachedRoot + "/main.ts").href,
    pathToFileURL(storage.materializedRoot + "/main.ts").href,
  ]);
  const ownerIds = owners.map((owner) => owner.nativeOwner);
  assert.ok(
    ownerIds.length > 0 && ownerIds.every((owner) => Number.isSafeInteger(owner) && owner > 0),
  );
  assert.equal(new Set(ownerIds).size, ownerIds.length);
  const evidence = owners.map((owner) => {
    assert.equal(verifyNativeRelations(owner.client, owner.server, context), null);
    assert.equal(verifyNativeRelations(owner.apiClient, owner.apiServer, context), null);
    const apiRequests = owner.apiClient.map(object).filter((row) => "id" in row && "method" in row);
    for (const request of apiRequests) {
      assert.equal(typeof request.method, "string");
      assert.ok(
        configurationMethods.has(request.method as string),
        "attached beta API may only configure and own snapshots",
      );
    }
    const replies = new Map(
      owner.server
        .map(object)
        .filter((row) => "id" in row && !("method" in row))
        .map((row) => [id(row.id), row]),
    );
    const calls = owner.client.map(object).filter((row) => {
      if (!("method" in row)) return false;
      assert.ok(typeof row.method === "string");
      return semanticMethods.includes(row.method);
    });
    const packets = calls.map((request) => {
      const uri = object(object(request.params).textDocument).uri;
      assert.equal(typeof uri, "string");
      const filename = fileURLToPath(uri as string);
      assert.equal(pathToFileURL(filename).href, uri);
      assert.ok(
        storage.documentUris.includes(uri as string),
        "semantic request belongs to another project",
      );
      const response = replies.get(id(request.id));
      assert.ok(
        response && "result" in response && !("error" in response),
        "whole successful semantic LSP reply required",
      );
      return { request, response, uri };
    });
    const main = packets.filter((packet) => mainUris.has(packet.uri as string));
    const counts = Object.fromEntries(
      semanticMethods.map((method) => [
        method,
        main.filter((packet) => packet.request.method === method).length,
      ]),
    );
    const complete = semanticMethods.every(
      (method) => counts[method]! >= phases.length * (method === "textDocument/diagnostic" ? 3 : 2),
    );
    if (complete) {
      const order = [
        semanticMethods[0],
        semanticMethods[0],
        semanticMethods[0],
        ...semanticMethods.slice(1).flatMap((method) => [method, method]),
      ];
      assert.equal(
        main.length,
        phases.length * order.length,
        "complete distinct phase vectors required",
      );
      for (const [phaseIndex, phase] of phases.entries()) {
        const packets = main.slice(phaseIndex * order.length, (phaseIndex + 1) * order.length);
        assert.deepEqual(
          packets.map((packet) => packet.request.method),
          order,
          "raw semantic phase ordering differs",
        );
        const expected = [
          phase.nativePull,
          phase.nativePull,
          phase.nativePull,
          phase.hover,
          phase.hover,
          phase.references,
          phase.directReferences,
          phase.definition,
          phase.directDefinition,
          phase.completion,
          phase.directCompletion,
          object(phase.rename).Ok,
          phase.directRename,
        ];
        for (const [index, value] of expected.entries()) {
          assert.notEqual(value, undefined, "complete retained source phase output required");
          assert.ok(
            order[index] === "textDocument/completion"
              ? sameCompletion(packets[index]!.response.result, value)
              : isDeepStrictEqual(packets[index]!.response.result, value),
            `whole raw semantic result differs for phase ${phaseIndex}, query ${index}`,
          );
        }
      }
    }
    return {
      nativeOwner: owner.nativeOwner,
      apiRequests,
      apiResponses: owner.apiServer,
      semanticPackets: packets,
      counts,
      complete,
    };
  });
  assert.ok(
    evidence.some((owner) => owner.complete),
    "one native owner must serve all four phases through complete LSP query pairs",
  );
  return {
    schema: "vize.nativeBetaAttached.semanticRoute.v1",
    context,
    sourcePhases: sourceRows,
    storage,
    owners: evidence,
    scope:
      "Complete semantic LSP pairs and configuration-only API pairs; an API attachment is not semantic-route evidence.",
  };
}
