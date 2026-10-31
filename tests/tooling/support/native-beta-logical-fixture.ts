// Constructed controls only: no actual native/source execution or qualification.
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";
import { storageFixture } from "./native-beta-storage-fixture.ts";
import type { AttachedOwner } from "./native-beta-dto/beta-attached-route.ts";

type Packet = Record<string, unknown>;
export function logicalFixture() {
  const old = storageFixture(),
    root = old.attached.root,
    source = "d".repeat(40);
  const hash = (text: string) => createHash("sha256").update(text).digest("hex");
  const texts: Array<[string, string]> = [
    ["main.ts", old.attached.rows[0]!.mainText],
    ["support.ts", old.attached.rows[0]!.supportOverlay],
    ["Virtual.vue.ts", old.attached.materializedBatch.source],
  ];
  const config =
    '{"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","types":[],"noEmit":true},"files":["main.ts","support.ts"]}';
  const documents = texts.map(([name, text]) => ({
    externalUri: pathToFileURL(root + "/" + name).href,
    mappedUri: pathToFileURL(root + "/" + name).href,
    relativePath: name,
    storage: "retained-buffer",
    text,
    bytes: Buffer.byteLength(text),
    sha256: hash(text),
    disk:
      name === "Virtual.vue.ts"
        ? null
        : { path: root + "/" + name, bytes: Buffer.byteLength(text), sha256: hash(text) },
  }));
  const revision = {
    schema: 2,
    mode: "logical-editor-revision",
    scope:
      "source-selected original logical root and complete retained buffers; no disk materialization",
    authoredRoot: root,
    clientProjectRoot: root,
    logicalRoot: root,
    canonicalLogicalRoot: root,
    documents,
    retainedDocuments: documents
      .map((row) => ({
        uri: row.externalUri,
        text: row.text,
        bytes: row.bytes,
        sha256: row.sha256,
      }))
      .sort((a, b) => (a.uri < b.uri ? -1 : a.uri > b.uri ? 1 : 0)),
    config: {
      path: root + "/tsconfig.json",
      text: config,
      bytes: Buffer.byteLength(config),
      sha256: hash(config),
    },
  };
  const phases = old.attached.rows.map((row, index) => ({
    ...row,
    apiEligible: false,
    nativePull: {
      kind: "full",
      items: index === 1 ? [{ code: 2322, message: "controlled dependency error" }] : [],
    },
    selected: {
      complete: index === 1 ? [{ code: 2322, message: "controlled dependency error" }] : [],
    },
    hover: { phase: index },
    references: { phase: index },
    directReferences: { phase: index },
    definition: { phase: index },
    directDefinition: { phase: index },
    completion: { isIncomplete: false, items: [{ label: "item", data: { phase: index } }] },
    directCompletion: { isIncomplete: false, items: [{ label: "item", data: { phase: index } }] },
    rename: { Ok: { phase: index } },
    directRename: { phase: index },
  }));
  const attached = {
    ...old.attached,
    config,
    rows: phases,
    materializedBatch: {
      ...old.attached.materializedBatch,
      mappedUri: old.attached.materializedBatch.virtualUri,
      storageAuthority: revision,
    },
  };
  const client: Packet[] = [
    {
      jsonrpc: "2.0",
      id: -1,
      method: "initialize",
      params: { rootPath: root, rootUri: pathToFileURL(root).href },
    },
    {
      jsonrpc: "2.0",
      id: -2,
      method: "custom/projectInfo",
      params: { textDocument: { uri: attached.mainUri } },
    },
  ];
  const server: Packet[] = [
    {
      jsonrpc: "2.0",
      id: -1,
      result: { serverInfo: { name: "typescript", version: "7.1.0-beta" } },
    },
    { jsonrpc: "2.0", id: -2, result: { configFilePath: root + "/tsconfig.json" } },
  ];
  let id = 0;
  for (const phase of phases) {
    const values = [
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
      phase.rename.Ok,
      phase.directRename,
    ];
    const methods = [
      "diagnostic",
      "diagnostic",
      "diagnostic",
      "hover",
      "hover",
      "references",
      "references",
      "definition",
      "definition",
      "completion",
      "completion",
      "rename",
      "rename",
    ];
    for (const [index, method] of methods.entries()) {
      client.push({
        jsonrpc: "2.0",
        id: ++id,
        method: "textDocument/" + method,
        params: { textDocument: { uri: attached.mainUri } },
      });
      server.push({ jsonrpc: "2.0", id, result: structuredClone(values[index]) });
    }
  }
  const apiClient: Packet[] = [],
    apiServer: Packet[] = [];
  for (const method of [
    "initialize",
    "parseConfigFile",
    "createSnapshot",
    "updateSnapshot",
    "describeCapabilities",
    "release",
  ]) {
    const id = apiClient.length + 1;
    apiClient.push({ jsonrpc: "2.0", id, method, params: {} });
    apiServer.push(
      method === "describeCapabilities"
        ? {
            jsonrpc: "2.0",
            id,
            error: {
              code: -32603,
              message: 'api: invalid request: unknown API method "describeCapabilities"',
            },
          }
        : { jsonrpc: "2.0", id, result: {} },
    );
  }
  const owner: AttachedOwner = { nativeOwner: 123, client, server, apiClient, apiServer };
  return {
    root,
    source,
    attached,
    phases,
    owners: [owner],
    context: {
      source,
      role: "after",
      sdkVersion: "7.1.0-beta",
      sdkSha256: "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
      sdkExecutable: "/sdk/node_modules/@typescript/typescript-linux-x64/lib/tsc",
      projectRoots: [root],
    },
  };
}
