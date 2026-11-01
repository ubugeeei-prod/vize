import assert from "node:assert/strict";
import { test } from "node:test";
import { logicalFixture } from "./support/native-beta-logical-fixture.ts";
import { verifyBetaLogicalRoute } from "./support/native-beta-dto/logical-attached-route.ts";
import { verifySemanticPhases } from "./support/native-beta-dto/semantic-route.ts";
type Fixture = ReturnType<typeof logicalFixture>;
type Packet = Record<string, unknown>;
const object = (value: unknown): Packet => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Packet;
};
const verify = (f: Fixture) =>
  verifyBetaLogicalRoute(f.owners, f.context, f.source, f.root, f.phases, f.attached);
const request = (f: Fixture, id: number) =>
  object(f.owners[0]!.client.find((row) => object(row).id === id));
const reply = (f: Fixture, id: number) =>
  object(f.owners[0]!.server.find((row) => object(row).id === id));
const open = (f: Fixture, name: string) =>
  object(
    f.owners[0]!.client.find(
      (row) =>
        object(row).method === "textDocument/didOpen" &&
        String(object(object(object(row).params).textDocument).uri).endsWith(name),
    ),
  );
test("logical batch retains all55 pairs while the shared phase comparator remains exactly52", () => {
  const f = logicalFixture(),
    proof = verify(f),
    actor = proof.owners[0]!;
  assert.equal(actor.semanticPackets.length, 55);
  assert.equal(actor.complete, true);
  const suffix = object(actor.sourceSupplement);
  assert.equal(suffix.schema, "vize.nativeBetaLogical.wholeBatchSuffix.v1");
  assert.equal((suffix.packets as unknown[]).length, 3);
  assert.throws(
    () => verifySemanticPhases(actor.semanticPackets, f.phases),
    "the shared comparator never accepts55 or53 as52",
  );
  assert.deepEqual(actor.configurationMembership, []);
});
test("missing, duplicate, extra, reordered or foreign batch packets cannot replace phase pairs", () => {
  const changes: Array<(f: Fixture) => void> = [
    ...[53, 54, 55].map((id) => (f: Fixture) => {
      f.owners[0]!.client = f.owners[0]!.client.filter((row) => object(row).id !== id);
      f.owners[0]!.server = f.owners[0]!.server.filter((row) => object(row).id !== id);
    }),
    (f) => {
      f.owners[0]!.client = [...f.owners[0]!.client, { ...request(f, 54), id: 99 }];
      f.owners[0]!.server = [...f.owners[0]!.server, { ...reply(f, 54), id: 99 }];
    },
    (f) => {
      request(f, 54).method = "textDocument/hover";
    },
    (f) => {
      object(object(request(f, 54).params).textDocument).uri = f.attached.supportUri;
    },
    (f) => {
      reply(f, 54).result = { kind: "full", items: [{ code: 1 }] };
    },
    (f) => {
      reply(f, 55).result = { kind: "full", items: [] };
    },
    (f) => {
      reply(f, 53).result = { kind: "full", items: [] };
    },
    (f) => {
      const c = [...f.owners[0]!.client],
        a = c.indexOf(request(f, 54)),
        b = c.indexOf(request(f, 55));
      [c[a], c[b]] = [c[b]!, c[a]!];
      f.owners[0]!.client = c;
    },
    (f) => {
      const s = [...f.owners[0]!.server],
        a = s.indexOf(reply(f, 53)),
        b = s.indexOf(reply(f, 54));
      [s[a], s[b]] = [s[b]!, s[a]!];
      f.owners[0]!.server = s;
    },
    (f) => {
      f.attached.materializedBatch.selected.Ok.reverse();
    },
    (f) => {
      f.attached.materializedBatch.selected.Ok.push([f.attached.mainUri, []]);
    },
    (f) => {
      object(f.attached.materializedBatch.selected).extra = true;
    },
    (f) => {
      const rows = f.attached.materializedBatch.selected.Ok;
      (rows[0] as unknown[])[1] = [{ code: 1 }];
      reply(f, 54).result = { kind: "full", items: [{ code: 1 }] };
    },
    (f) => {
      f.attached.materializedBatch.nativePull.items = [];
      (f.attached.materializedBatch.selected.Ok[1] as unknown[])[1] = [];
      reply(f, 53).result = reply(f, 55).result = { kind: "full", items: [] };
    },
  ];
  for (const change of changes) {
    const f = logicalFixture();
    change(f);
    assert.throws(() => verify(f));
  }
});
test("source document lifecycle, source identity and six-method API boundary remain closed", () => {
  const changes: Array<(f: Fixture) => void> = [
    (f) => {
      object(object(open(f, "Virtual.vue.ts").params).textDocument).text = "wrong";
    },
    (f) => {
      object(object(open(f, "support.ts").params).textDocument).text = "wrong";
    },
    (f) => {
      const row = open(f, "Virtual.vue.ts");
      f.owners[0]!.client = f.owners[0]!.client.filter((value) => value !== row);
    },
    (f) => {
      const row = open(f, "support.ts");
      f.owners[0]!.client = f.owners[0]!.client.filter((value) => value !== row);
    },
    (f) => {
      const row = open(f, "support.ts");
      const c = [...f.owners[0]!.client];
      c.splice(c.indexOf(request(f, 54)), 0, structuredClone(row));
      f.owners[0]!.client = c;
    },
    (f) => {
      const c = [...f.owners[0]!.client];
      c.splice(c.indexOf(request(f, 54)), 0, {
        jsonrpc: "2.0",
        method: "textDocument/didClose",
        params: { textDocument: { uri: f.attached.mainUri } },
      });
      f.owners[0]!.client = c;
    },
    (f) => {
      f.context.role = "before";
    },
    (f) => {
      f.context.source = "0".repeat(40);
    },
    (f) => {
      f.context.sdkSha256 = "0".repeat(64);
    },
    (f) => {
      f.owners[0]!.apiClient = [
        ...f.owners[0]!.apiClient,
        { jsonrpc: "2.0", id: 99, method: "getSemanticDiagnostics", params: {} },
      ];
      f.owners[0]!.apiServer = [...f.owners[0]!.apiServer, { jsonrpc: "2.0", id: 99, result: {} }];
    },
  ];
  for (const change of changes) {
    const f = logicalFixture();
    change(f);
    assert.throws(() => verify(f));
  }
});
