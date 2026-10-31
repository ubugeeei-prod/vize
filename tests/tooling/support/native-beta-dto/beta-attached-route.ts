import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import { pathToFileURL } from "node:url";
import type { WatcherContext } from "./stable-watcher-boundary.ts";
import { verifyBetaSemanticRoute, semanticMethods } from "./semantic-route.ts";
import type { Receipt } from "./audit-wire-types.ts";
import { verifyAttachedConfigurationMembership } from "./configuration-membership.ts";

type Packet = Record<string, unknown>;
export interface AttachedOwner {
  nativeOwner: number;
  client: readonly unknown[];
  server: readonly unknown[];
  apiClient: readonly unknown[];
  apiServer: readonly unknown[];
}
export const storageObserverPath =
  "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta/storage_authority.rs";
export const storageObserverSha256 =
  "9bc79324540fd1b37f649f102fa2ed90a09aaf53eca1765f799cb6bc1fe88d40";
export const commonObserverSha256 =
  "c75ef79c388eb5aee6c93115150196c42aecb75d638ead10dab1a1b94872ca61";
export const compiledInputPath =
  "tests/_fixtures/differential/typechecker/logical-workspace-overlay-revision-11145/input.json";
export const compiledInputSha256 =
  "4659fc88d76ee98c9f0ddc74f6450ecc80b720be9b3ab4a0335bf8f2a2050c58";
const storageScope =
  "source getter and materialized file reads before close; no root inferred from observed RPC";
const mainText =
  '/*🧭é日本語\u2028\u2029*/\r\nimport { value } from "./support";\r\nexport const checked: number = value;\r\nexport const echo = value;\r\n';
const supportText = "export const value = 1;\r\n";
const virtualText = 'export const bad: number = "x";\r\n';
const hash = (value: string) => createHash("sha256").update(value).digest("hex");
/** Exact authenticated native image tuples own their distinct server labels. */
export function assertImageServerInfo(
  value: unknown,
  context: Pick<WatcherContext, "sdkVersion" | "sdkSha256">,
) {
  let name: string;
  if (context.sdkVersion === "7.0.2") {
    assert.equal(
      context.sdkSha256,
      "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
    );
    name = "typescript-go";
  } else {
    assert.equal(context.sdkVersion, "7.1.0-beta");
    assert.equal(
      context.sdkSha256,
      "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
    );
    name = "typescript";
  }
  assert.deepEqual(value, { name, version: context.sdkVersion });
}
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
/** Actual compiled fixture bytes are separate from the sole Rust helper identity. */
export function verifyCompiledInput(compiled: unknown, prepared: unknown) {
  assert.ok(Array.isArray(compiled) && Array.isArray(prepared));
  assert.equal(compiled.length, 1);
  assert.deepEqual(compiled, prepared, "compiled and prepared input bytes differ");
  closed(compiled[0], ["path", "bytes", "sha256"]);
  assert.deepEqual(compiled[0], {
    path: compiledInputPath,
    bytes: 1108,
    sha256: compiledInputSha256,
  });
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
    assertImageServerInfo(object(initializedReplies[0]!.result).serverInfo, context);
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
  return verifyBetaSemanticRoute(
    owners,
    context,
    expectedSource,
    attachedRoot,
    sourceRows,
    storage,
    storage.materializedRoot,
    (owner) =>
      verifyAttachedConfigurationMembership(owner, context, expectedSource, storage, attachedValue),
  );
}
