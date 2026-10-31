import assert from "node:assert/strict";
import { test } from "node:test";
import { verifyBetaAttachedRoute } from "./support/native-beta-dto/beta-attached-route.ts";
import { storageFixture } from "./support/native-beta-storage-fixture.ts";

type Packet = Record<string, unknown>;
function fixture() {
  const root = "/tmp/owned-beta-project",
    source = "b".repeat(40);
  const storage = storageFixture(root);
  const phases = ["valid", "dependency-error", "dependency-repair", "dependency-close"].map(
    (phase, index) => ({
      ...storage.attached.rows[index],
      phase,
      apiEligible: false,
      nativePull: {
        kind: "full",
        items:
          phase === "dependency-error"
            ? [{ code: 2322, message: "controlled error" }]
            : ([] as unknown[]),
      },
      selected: {
        complete:
          phase === "dependency-error"
            ? [{ code: 2322, message: "controlled error" }]
            : ([] as unknown[]),
      },
      hover: { complete: true },
      references: { complete: true },
      directReferences: { complete: true },
      definition: { complete: true },
      directDefinition: { complete: true },
      completion: {
        isIncomplete: false,
        items: [
          { label: "x", data: { opaque: [1, 2] } },
          { label: "y", data: { opaque: [3] } },
          { label: "x", data: { opaque: [1, 2] } },
        ],
      },
      directCompletion: {
        isIncomplete: false,
        items: [
          { label: "x", data: { opaque: [1, 2] } },
          { label: "y", data: { opaque: [3] } },
          { label: "x", data: { opaque: [1, 2] } },
        ],
      },
      rename: { Ok: { complete: true } },
      directRename: { complete: true },
    }),
  );
  const client: Packet[] = [],
    server: Packet[] = [],
    apiClient: Packet[] = [],
    apiServer: Packet[] = [];
  client.push({
    jsonrpc: "2.0",
    id: -1,
    method: "initialize",
    params: {
      rootPath: storage.root,
      rootUri: `file://${storage.root}`,
    },
  });
  server.push({
    jsonrpc: "2.0",
    id: -1,
    result: { serverInfo: { name: "typescript-go", version: "7.1.0-beta" } },
  });
  client.push({
    jsonrpc: "2.0",
    id: -2,
    method: "custom/projectInfo",
    params: {
      textDocument: { uri: `file://${storage.root}/main.ts` },
    },
  });
  server.push({
    jsonrpc: "2.0",
    id: -2,
    result: { configFilePath: storage.root + "/tsconfig.json" },
  });
  const main = `file://${root}/main.ts`,
    mapped = `file://${storage.root}/main.ts`;
  let id = 0;
  for (const phase of phases) {
    for (const uri of [main, mapped, mapped]) {
      client.push({
        jsonrpc: "2.0",
        id: ++id,
        method: "textDocument/diagnostic",
        params: { textDocument: { uri } },
      });
      server.push({ jsonrpc: "2.0", id, result: structuredClone(phase.nativePull) });
    }
    for (const method of ["hover", "references", "definition", "completion", "rename"])
      for (let count = 0; count < 2; count++) {
        client.push({
          jsonrpc: "2.0",
          id: ++id,
          method: "textDocument/" + method,
          params: { textDocument: { uri: main } },
        });
        server.push({
          jsonrpc: "2.0",
          id,
          result:
            method === "completion"
              ? structuredClone(count === 0 ? phase.completion : phase.directCompletion)
              : { complete: true },
        });
      }
  }
  for (const method of [
    "initialize",
    "parseConfigFile",
    "createSnapshot",
    "updateSnapshot",
    "describeCapabilities",
    "release",
  ]) {
    apiClient.push({ jsonrpc: "2.0", id: apiClient.length + 1, method, params: {} });
    apiServer.push({ jsonrpc: "2.0", id: apiClient.length, result: {} });
  }
  return {
    owners: [{ nativeOwner: 123, client, server, apiClient, apiServer }],
    phases,
    source,
    root,
    attached: { ...storage.attached, rows: phases },
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
const verify = (f: ReturnType<typeof fixture>) =>
  verifyBetaAttachedRoute(f.owners, f.context, f.source, f.root, f.phases, f.attached);

test("configuration attachment retains a complete same-owner LSP semantic-route witness", () => {
  const f = fixture(),
    proof = verify(f);
  assert.equal(proof.owners[0]!.complete, true);
  assert.equal(proof.owners[0]!.counts["textDocument/diagnostic"], 12);
  assert.equal(proof.owners[0]!.counts["textDocument/rename"], 8);
  assert.deepEqual(proof.owners[0]!.apiRequests, f.owners[0]!.apiClient);
  assert.deepEqual(proof.sourcePhases, f.phases);
});
test("missing, split, failed, foreign, API-semantic or incomplete phase evidence cannot attest routing", () => {
  const mutations: Array<[string, (f: ReturnType<typeof fixture>) => void]> = [
    [
      "reversed diagnostic phase results with unchanged IDs/counts",
      (f) => {
        const owner = f.owners[0]!;
        const calls = owner.client.filter((row) => row.method === "textDocument/diagnostic");
        const replies = calls.map((row) => owner.server.find((reply) => reply.id === row.id)!);
        const values = replies.map((reply) => structuredClone(reply.result)).reverse();
        replies.forEach((reply, index) => {
          reply.result = values[index];
        });
      },
    ],
    [
      "changed navigation phase result",
      (f) => {
        f.phases[0]!.hover = { complete: false };
      },
    ],
    [
      "wrong role",
      (f) => {
        f.context.role = "before";
      },
    ],
    [
      "wrong SDK",
      (f) => {
        f.context.sdkVersion = "7.0.2";
      },
    ],
    [
      "wrong image",
      (f) => {
        f.context.sdkSha256 = "0".repeat(64);
      },
    ],
    [
      "wrong source",
      (f) => {
        f.context.source = "a".repeat(40);
      },
    ],
    [
      "API eligible phase",
      (f) => {
        f.phases[0]!.apiEligible = true;
      },
    ],
    [
      "missing phase",
      (f) => {
        f.phases.pop();
      },
    ],
    [
      "wrong phase diagnostic",
      (f) => {
        f.phases[0]!.nativePull.items = [{ code: 9999, message: "forged" }];
        f.phases[0]!.selected.complete = [{ code: 9999, message: "forged" }];
      },
    ],
    [
      "missing LSP reply",
      (f) => {
        f.owners[0]!.server.pop();
      },
    ],
    [
      "failed LSP reply",
      (f) => {
        f.owners[0]!.server[0]!.error = { code: -1 };
      },
    ],
    [
      "missing API reply",
      (f) => {
        f.owners[0]!.apiServer.pop();
      },
    ],
    [
      "semantic API with complete reply",
      (f) => {
        f.owners[0]!.apiClient[0]!.method = "getSemanticDiagnostics";
      },
    ],
    [
      "unknown API method",
      (f) => {
        f.owners[0]!.apiClient[0]!.method = "futureSemanticQuery";
      },
    ],
    [
      "foreign query",
      (f) => {
        (
          f.owners[0]!.client.find((row) => row.method === "textDocument/diagnostic")!.params as {
            textDocument: { uri: string };
          }
        ).textDocument.uri = "file:///tmp/foreign/main.ts";
      },
    ],
    [
      "missing complete query pair",
      (f) => {
        f.owners[0]!.client.pop();
        f.owners[0]!.server.pop();
      },
    ],
    [
      "routes split across owners",
      (f) => {
        const old = f.owners[0]!;
        f.owners.push({
          ...old,
          nativeOwner: 456,
          client: old.client.splice(26),
          server: old.server.splice(26),
        });
      },
    ],
    [
      "duplicate native owner",
      (f) => {
        f.owners.push(structuredClone(f.owners[0]!));
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const f = fixture();
    mutate(f);
    assert.throws(() => verify(f), name);
  }
});
test("completion comparison retains complete metadata, nested arrays and duplicate counts", () => {
  const reordered = fixture();
  reordered.phases[0]!.completion.items = [
    reordered.phases[0]!.completion.items[1]!,
    reordered.phases[0]!.completion.items[0]!,
    reordered.phases[0]!.completion.items[2]!,
  ];
  assert.equal(verify(reordered).owners[0]!.complete, true);
  for (const mutate of [
    (value: Record<string, unknown>) => {
      delete value.isIncomplete;
    },
    (value: Record<string, unknown>) => {
      (value.items as unknown[]).pop();
    },
    (value: Record<string, unknown>) => {
      (value.items as Array<{ data: { opaque: number[] } }>)[0]!.data.opaque.reverse();
    },
  ]) {
    const f = fixture(),
      owner = f.owners[0]!;
    const request = owner.client.find((row) => row.method === "textDocument/completion")!;
    const response = owner.server.find((row) => row.id === request.id)!;
    mutate(response.result as Record<string, unknown>);
    assert.throws(() => verify(f));
  }
});
