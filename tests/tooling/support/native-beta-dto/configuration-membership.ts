import assert from "node:assert/strict";
import path from "node:path";
import { pathToFileURL } from "node:url";
import type { AttachedOwner, verifyAfterAttachedStorage } from "./beta-attached-route.ts";
import type { WatcherContext } from "./stable-watcher-boundary.ts";

type Packet = Record<string, unknown>;
const object = (value: unknown): Packet => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Packet;
};
const closed = (value: unknown, keys: string[]) => {
  const row = object(value);
  assert.deepEqual(Object.keys(row).sort(), keys.sort());
  return row;
};
const id = (value: unknown) => {
  assert.ok(
    typeof value === "string" || (typeof value === "number" && Number.isSafeInteger(value)),
  );
  return `${typeof value}:${value}`;
};
const one = <T>(rows: readonly T[]): T => {
  assert.equal(rows.length, 1);
  return rows[0]!;
};

/** The SDK's project-selection RPC grants configuration membership, never semantics. */
export function verifyAttachedConfigurationMembership(
  owner: AttachedOwner,
  context: WatcherContext,
  expectedSource: string,
  storage: ReturnType<typeof verifyAfterAttachedStorage>,
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
  const requests = owner.apiClient.map(object),
    replies = owner.apiServer.map(object);
  const membership = requests.filter((row) => row.method === "getDefaultProjectForFile");
  if (membership.length === 0) return [];
  const root = storage.materializedRoot,
    authored = storage.authoredRoot;
  assert.ok(context.projectRoots.includes(authored));
  const config = root + "/tsconfig.json";
  const initialized = one(owner.client.map(object).filter((row) => row.method === "initialize"));
  assert.equal(
    object(initialized.params).rootPath,
    root,
    "only the source-selected materialized owner may select a member",
  );
  assert.equal(object(initialized.params).rootUri, pathToFileURL(root).href);
  const response = (request: Packet) => {
    const row = one(
      replies.filter((row) => "id" in row && !("method" in row) && id(row.id) === id(request.id)),
    );
    closed(row, ["jsonrpc", "id", "result"]);
    assert.equal(row.jsonrpc, "2.0");
    return row;
  };
  const parse = one(requests.filter((row) => row.method === "parseConfigFile"));
  assert.deepEqual(parse.params, { file: config });
  const parsed = closed(response(parse).result, [
    "fileNames",
    "options",
    "compileOnSave",
    "raw",
    "errors",
  ]);
  const ordinary = [root + "/main.ts", root + "/support.ts"];
  const virtual = root + "/Virtual.vue.ts";
  const virtualUris = [
    pathToFileURL(authored + "/Virtual.vue.ts").href,
    pathToFileURL(virtual).href,
  ];
  const sourceClient = owner.client.map(object);
  const virtualCalls = sourceClient.filter((row) => {
    if (typeof row.method !== "string" || !row.method.startsWith("textDocument/")) return false;
    const uri = object(object(row.params).textDocument).uri;
    return virtualUris.includes(String(uri));
  });
  const virtualOpens = virtualCalls.filter((row) => row.method === "textDocument/didOpen");
  const virtualDiagnostics = virtualCalls.filter((row) => row.method === "textDocument/diagnostic");
  const hasVirtual = virtualCalls.length > 0;
  if (hasVirtual) {
    assert.ok(
      virtualOpens.length > 0 && virtualDiagnostics.length > 0,
      "Virtual membership requires independently observed client open and diagnostic phases",
    );
    for (const row of virtualOpens) {
      const document = closed(object(row.params).textDocument, [
        "uri",
        "text",
        "languageId",
        "version",
      ]);
      assert.equal(document.text, object(object(attachedValue).materializedBatch).source);
      assert.equal(document.languageId, "typescript");
      assert.equal(document.version, 1);
    }
  }
  const expectedMapped = hasVirtual ? [virtual, ...ordinary] : ordinary;
  const expectedFiles = [authored + "/main.ts", authored + "/support.ts", ...expectedMapped];
  assert.deepEqual(
    parsed.fileNames,
    expectedFiles,
    "only the source fixture's exact written member roster may select a project",
  );
  for (const filename of expectedMapped)
    assert.ok(storage.documentUris.includes(pathToFileURL(filename).href));
  const authoredConfig = {
    compilerOptions: {
      strict: true,
      target: "ESNext",
      module: "ESNext",
      moduleResolution: "Bundler",
      types: [],
      noEmit: true,
    },
    files: ["main.ts", "support.ts"],
  };
  assert.deepEqual(JSON.parse(String(object(attachedValue).config)), authoredConfig);
  assert.deepEqual(parsed, {
    fileNames: expectedFiles,
    options: {
      allowImportingTsExtensions: true,
      module: 99,
      moduleResolution: 100,
      noEmit: true,
      strict: true,
      target: 99,
      types: [],
      configFilePath: config,
    },
    compileOnSave: false,
    raw: {
      ...JSON.parse(String(object(storage.storageAuthority.config).text)),
      files: authoredConfig.files.map((file) => path.posix.relative(root, authored + "/" + file)),
    },
    errors: [],
  });
  const preferred = expectedMapped[0]!;
  const identity = one(
    owner.client.map(object).filter((row) => row.method === "custom/projectInfo"),
  );
  assert.deepEqual(identity.params, { textDocument: { uri: pathToFileURL(preferred).href } });
  const identityReply = one(
    owner.server
      .map(object)
      .filter((row) => "id" in row && !("method" in row) && id(row.id) === id(identity.id)),
  );
  assert.deepEqual(identityReply.result, { configFilePath: config });
  assert.equal("error" in identityReply, false);
  let liveSnapshot: unknown,
    preceding: Packet | null = null;
  const seenSnapshots = new Set<string>();
  const evidence: Array<{
    request: Packet;
    response: Packet;
    snapshotRequest: Packet;
    snapshotResponse: Packet;
    parsedConfiguration: Packet;
    sourceClientVirtualActivity: Packet[];
  }> = [];
  for (const request of requests) {
    if (request.method === "createSnapshot" || request.method === "updateSnapshot") {
      const params = object(request.params);
      let changes: Packet;
      if (request.method === "createSnapshot") {
        assert.equal(preceding, null, "one initial snapshot lineage required");
        changes = params;
      } else {
        closed(params, ["changes", "snapshot"]);
        assert.notEqual(preceding, null);
        assert.equal(
          params.snapshot,
          liveSnapshot,
          "update must extend this owner's current snapshot",
        );
        changes = object(params.changes);
      }
      assert.deepEqual(
        Object.keys(changes).sort(),
        ("fileNotifications" in changes
          ? ["ensurePrograms", "fileNotifications", "openProjects"]
          : ["ensurePrograms", "openProjects"]
        ).sort(),
      );
      assert.equal(changes.ensurePrograms, true);
      assert.deepEqual(changes.openProjects, [config]);
      if ("fileNotifications" in changes) {
        const notifications = object(changes.fileNotifications);
        for (const [kind, files] of Object.entries(notifications)) {
          assert.ok(["changed", "created", "deleted"].includes(kind));
          assert.ok(Array.isArray(files));
          for (const file of files) assert.ok(expectedMapped.includes(String(file)));
        }
      }
      const result = object(response(request).result);
      id(result.snapshot);
      assert.ok(
        !seenSnapshots.has(id(result.snapshot)),
        "snapshot handles cannot be reused within the owner",
      );
      seenSnapshots.add(id(result.snapshot));
      liveSnapshot = result.snapshot;
      preceding = request;
    } else if (request.method === "getDefaultProjectForFile") {
      closed(request, ["jsonrpc", "id", "method", "params"]);
      assert.equal(request.jsonrpc, "2.0");
      assert.ok(preceding, "membership cannot precede its owned snapshot");
      assert.equal(
        preceding.method,
        "updateSnapshot",
        "the source project session resolves its opened or refreshed snapshot",
      );
      assert.ok(requests.indexOf(parse) < requests.indexOf(request));
      assert.deepEqual(request.params, { file: preferred, snapshot: liveSnapshot });
      assert.ok(
        !evidence.some((row) => object(row.request.params).snapshot === liveSnapshot),
        "one membership read per snapshot transition",
      );
      const reply = response(request),
        project = closed(reply.result, [
          "id",
          "configFileName",
          "currentDirectory",
          "dirty",
          "parsedCommandLine",
        ]);
      assert.deepEqual(project, {
        id: config,
        configFileName: config,
        currentDirectory: root,
        dirty: false,
        parsedCommandLine: parsed,
      });
      assert.ok(
        replies.indexOf(response(preceding)) < replies.indexOf(reply),
        "snapshot reply must precede its membership reply on the complete server stream",
      );
      evidence.push({
        request,
        response: reply,
        snapshotRequest: preceding,
        snapshotResponse: response(preceding),
        parsedConfiguration: parsed,
        sourceClientVirtualActivity: virtualCalls,
      });
    } else if (request.method === "release") {
      if (object(request.params).snapshot === liveSnapshot) liveSnapshot = undefined;
    }
  }
  return evidence;
}
