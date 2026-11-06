import assert from "node:assert/strict";
import test from "node:test";
import {
  certifyListenerRelationProjections,
  mapperCampaign,
} from "./template-alias-paired-11822-public.ts";

const inline = "[Extract<NonNullable<__H>, (...args: any) => any>] extends [never]";
const shared = 'import("vize:virtual-ts").$ListenerArgs<';

test("listener projection certification requires the named relation rewrite", () => {
  const before = [
    { path: "src/A.vue", text: `type A = ${inline};`, metadata: { mappings: [1] } },
    { path: "src/B.vue", text: "type B = string;", metadata: { mappings: [2] } },
  ];
  const after = [
    { path: "src/A.vue", text: `type A = ${shared}P, K>;`, metadata: { mappings: [3] } },
    structuredClone(before[1]!),
  ];
  assert.deepEqual(certifyListenerRelationProjections(before, after), {
    changedDocuments: 1,
    removedInlineRelations: 1,
    sharedReferences: 1,
    beforeBytes: Buffer.byteLength(before[0]!.text) + Buffer.byteLength(before[1]!.text),
    afterBytes: Buffer.byteLength(after[0]!.text) + Buffer.byteLength(after[1]!.text),
  });
  for (const mutation of [
    after.map((row) => ({ ...row, path: "src/changed.vue" })),
    before.map((row) => structuredClone(row)),
    after.map((row) => ({ ...row, text: row.text.replace(shared, "Unknown<") })),
    after.map((row) => (row.path === "src/B.vue" ? { ...row, metadata: { mappings: [99] } } : row)),
  ])
    assert.throws(() => certifyListenerRelationProjections(before, mutation));
});

test("mapper campaign admission isolates listener and dispatcher production changes", () => {
  const listener = [
    "crates/vize_canon/src/virtual_ts/scope/component_events/listener_relations.rs",
    "crates/vize_canon/src/virtual_ts/generator.rs",
    "crates/vize_canon/snapshots/vize_canon__virtual_ts__loose_is_listener_tests__sample.snap",
    "npm/cli/virtual-ts-listeners.d.ts",
    "tests/tooling/listener.test.ts",
    "tools/benchmarks/scripts/template-alias-paired-11822-public.ts",
  ];
  assert.deepEqual(mapperCampaign(listener), {
    campaign: "listener-argument-relations",
    dispatcher: false,
    listenerRelations: true,
  });
  assert.deepEqual(mapperCampaign(["crates/vize/src/commands/content_mapper/dispatch.rs"]), {
    campaign: "dispatcher-admission",
    dispatcher: true,
    listenerRelations: false,
  });
  assert.throws(() => mapperCampaign([]));
  assert.throws(() =>
    mapperCampaign([...listener, "crates/vize_canon/src/virtual_ts/unrelated.rs"]),
  );
  assert.throws(() =>
    mapperCampaign([...listener, "crates/vize/src/commands/content_mapper/dispatch.rs"]),
  );
});
