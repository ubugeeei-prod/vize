import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { assertImageServerInfo, type AttachedOwner } from "./beta-attached-route.ts";
import type { WatcherContext } from "./stable-watcher-boundary.ts";

type Packet = Record<string, unknown>;
const object = (value: unknown): Packet => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Packet;
};
const closed = (value: unknown, keys: string[]) => {
  const row = object(value);
  assert.deepEqual(
    Object.keys(row).sort(),
    keys.sort(),
    "closed logical revision evidence required",
  );
  return row;
};
const hash = (text: string) => createHash("sha256").update(text).digest("hex");
const mainText =
  '/*🧭é日本語\u2028\u2029*/\r\nimport { value } from "./support";\r\nexport const checked: number = value;\r\nexport const echo = value;\r\n';
const supportText = "export const value = 1;\r\n";
const virtualText = 'export const bad: number = "x";\r\n';
const configText =
  '{"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","types":[],"noEmit":true},"files":["main.ts","support.ts"]}';
const id = (value: unknown) => {
  assert.ok(
    typeof value === "string" || (typeof value === "number" && Number.isSafeInteger(value)),
  );
  return `${typeof value}:${value}`;
};

/** The exact current source owns original names and a complete retained editor revision. */
export function verifyAfterLogicalRevision(
  owners: readonly AttachedOwner[],
  context: WatcherContext,
  expectedSource: string,
  attachedValue: unknown,
) {
  assert.equal(context.role, "after");
  assert.match(expectedSource, /^[a-f0-9]{40}$/u);
  assert.equal(context.source, expectedSource);
  assert.ok(["7.0.2", "7.1.0-beta"].includes(context.sdkVersion));
  assertImageServerInfo(
    {
      name: context.sdkVersion === "7.0.2" ? "typescript-go" : "typescript",
      version: context.sdkVersion,
    },
    context,
  );
  const attached = object(attachedValue),
    batch = object(attached.materializedBatch);
  const proof = closed(batch.storageAuthority, [
    "schema",
    "mode",
    "scope",
    "authoredRoot",
    "clientProjectRoot",
    "logicalRoot",
    "canonicalLogicalRoot",
    "documents",
    "retainedDocuments",
    "config",
  ]);
  const root = attached.root;
  assert.equal(typeof root, "string");
  const authored = root as string;
  assert.ok(path.posix.isAbsolute(authored) && path.posix.normalize(authored) === authored);
  assert.ok(!/[\r\n\0]/u.test(authored));
  assert.ok(context.projectRoots.includes(authored));
  assert.equal(proof.schema, 2);
  assert.equal(proof.mode, "logical-editor-revision");
  assert.equal(
    proof.scope,
    "source-selected original logical root and complete retained buffers; no disk materialization",
  );
  for (const key of ["authoredRoot", "clientProjectRoot", "logicalRoot", "canonicalLogicalRoot"])
    assert.equal(proof[key], authored);
  assert.equal(attached.config, configText);
  const config = closed(proof.config, ["path", "text", "bytes", "sha256"]);
  assert.deepEqual(config, {
    path: authored + "/tsconfig.json",
    text: configText,
    bytes: Buffer.byteLength(configText),
    sha256: hash(configText),
  });
  const documents = [
    ["main.ts", mainText],
    ["support.ts", supportText],
    ["Virtual.vue.ts", virtualText],
  ] as const;
  const expectedDocuments = documents.map(([name, text]) => {
    const uri = pathToFileURL(authored + "/" + name).href;
    return {
      externalUri: uri,
      mappedUri: uri,
      relativePath: name,
      storage: "retained-buffer",
      text,
      bytes: Buffer.byteLength(text),
      sha256: hash(text),
      disk:
        name === "Virtual.vue.ts"
          ? null
          : { path: authored + "/" + name, bytes: Buffer.byteLength(text), sha256: hash(text) },
    };
  });
  assert.ok(Array.isArray(proof.documents));
  assert.equal(proof.documents.length, documents.length);
  for (const row of proof.documents)
    closed(row, [
      "externalUri",
      "mappedUri",
      "relativePath",
      "storage",
      "text",
      "bytes",
      "sha256",
      "disk",
    ]);
  assert.deepEqual(
    proof.documents,
    expectedDocuments,
    "retained and real disk evidence must match every source-authored byte",
  );
  const retained = expectedDocuments
    .map((row) => ({ uri: row.externalUri, text: row.text, bytes: row.bytes, sha256: row.sha256 }))
    .sort((a, b) => (a.uri < b.uri ? -1 : a.uri > b.uri ? 1 : 0));
  assert.ok(Array.isArray(proof.retainedDocuments));
  for (const row of proof.retainedDocuments) closed(row, ["uri", "text", "bytes", "sha256"]);
  assert.deepEqual(
    proof.retainedDocuments,
    retained,
    "whole sorted retained map required, without missing, extra or duplicate owners",
  );
  const uris = expectedDocuments.map((row) => row.externalUri);
  assert.equal(attached.mainUri, uris[0]);
  assert.equal(attached.supportUri, uris[1]);
  assert.equal(batch.virtualUri, uris[2]);
  assert.equal(batch.mappedUri, uris[2]);
  assert.equal(batch.source, virtualText);
  assert.ok(Array.isArray(attached.rows));
  assert.equal(attached.rows.length, 4);
  const supportPhases = [
    supportText,
    'export const value = "oops";\r\n',
    "export const value = 2;\r\n",
    supportText,
  ];
  for (const [index, row] of attached.rows.entries()) {
    assert.equal(object(row).mainText, mainText);
    assert.equal(object(row).supportOverlay, supportPhases[index]);
  }
  assert.ok(owners.length > 0);
  const ids = owners.map((owner) => owner.nativeOwner);
  assert.ok(ids.every((value) => Number.isSafeInteger(value) && value > 0));
  assert.equal(new Set(ids).size, ids.length);
  for (const owner of owners) {
    const client = owner.client.map(object),
      server = owner.server.map(object);
    const initialize = client.filter((row) => row.method === "initialize");
    assert.equal(initialize.length, 1);
    const params = object(initialize[0]!.params);
    assert.equal(params.rootPath, authored, "current source-selected logical root only");
    assert.equal(params.rootUri, pathToFileURL(authored).href);
    const initialized = server.filter(
      (row) => "id" in row && !("method" in row) && id(row.id) === id(initialize[0]!.id),
    );
    assert.equal(initialized.length, 1);
    assert.equal("error" in initialized[0]!, false);
    assertImageServerInfo(object(initialized[0]!.result).serverInfo, context);
    for (const request of client) {
      if (typeof request.method !== "string") continue;
      if (request.method === "custom/projectInfo" || request.method.startsWith("textDocument/")) {
        const document = object(object(request.params).textDocument);
        assert.ok(
          uris.includes(String(document.uri)),
          "only exact original source-owned URI joins, never physical aliases",
        );
        if (request.method === "custom/projectInfo") {
          const replies = server.filter(
            (row) => "id" in row && !("method" in row) && id(row.id) === id(request.id),
          );
          assert.equal(replies.length, 1);
          assert.equal("error" in replies[0]!, false);
          assert.deepEqual(replies[0]!.result, { configFilePath: authored + "/tsconfig.json" });
        }
      }
    }
  }
  return {
    logicalRoot: authored,
    authoredRoot: authored,
    documentUris: uris,
    revisionAuthority: proof,
  };
}
