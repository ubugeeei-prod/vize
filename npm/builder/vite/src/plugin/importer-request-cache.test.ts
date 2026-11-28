import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { after, test } from "node:test";

type RequestFacts = ReturnType<(typeof import("@vizejs/native"))["classifyVitePluginRequest"]>;

// Wrap before the source modules capture the binding's classifier. The oracle
// calls the original native function, independently of the importer cache.
const native = createRequire(import.meta.url)("@vizejs/native") as {
  classifyVitePluginRequest(id: string): RequestFacts;
};
const classifyNative = native.classifyVitePluginRequest;
let classifications = 0;
native.classifyVitePluginRequest = (id) => {
  classifications++;
  return classifyNative(id);
};
after(() => {
  native.classifyVitePluginRequest = classifyNative;
});

const { classifyImporterRequest } = await import("./importer-request.ts");
const { isPluginVisibleSsrVirtualId } = await import("../virtual.ts");
const { makeState } = await import("../test/precompile-cache-project.ts");

const importers = [
  "/app/main.ts",
  "/app/Foo.vue",
  "/app/Foo.vue?vue&vize",
  "/app/Foo.vue?vue&vize-ssr",
  "/app/Foo.vue?vue&vize-ssr=false",
  "/app/Foo.vue?vue&%76ize%2Dssr=true",
  "/app/Foo.vue?owner=x%26vize-ssr",
  "/app/Foo.vue?vue&type=style&index=2&lang=scss&scoped=data-v-x",
  "/app/Foo.vue?macro=true",
  "/app/Foo.vue?definePage",
  "/app/Foo.vue?nuxt_component=async",
  "/app/Foo.vue?raw",
  "/app/Foo.vue?vue&vize&t=123",
  "\0/app/Foo.vue.ts",
  "\0/app/Foo.vue.tsx?macro=true",
  "\0vize:/app/Foo.vue.ts",
  "\0vize-ssr:/app/Foo.vue.ts",
  "/@id/__x00__/app/Foo.vue.ts?vue&vize-ssr",
  "/@fs/app/Foo.vue?vue&vize",
  "/@fs/C:/app/Foo.vue?vue&vize-ssr",
  "C:/app/Foo.vue?vue&vize",
  "C:\\app\\Foo.vue?vue&vize-ssr",
  "/app/日本語/😀.vue?vue&vize-ssr",
  "/app/Foo%3F.vue?vue&vize",
];

test("interleaved importer facts equal the native classifier in independent states", () => {
  const states = [makeState("/app/first"), makeState("/app/second")];
  const sequence = [...importers, ...importers.toReversed(), ...importers];
  for (const [index, importer] of sequence.entries()) {
    const state = states[index % states.length];
    const expected = classifyNative(importer);
    const actual = classifyImporterRequest(state, importer);
    assert.deepEqual(actual, expected, importer);
    assert.equal(state.viteImporterRequest?.id, importer, importer);
    assert.equal(state.viteImporterRequest?.request, actual, importer);
    assert.deepEqual(Reflect.ownKeys(state.viteImporterRequest!).sort(), ["id", "request"]);
  }
});

test("only the last exact importer is retained, with no sharing between states", () => {
  const first = makeState("/app/first");
  const second = makeState("/app/second");
  const client = "/app/Foo.vue?vue&vize";
  const ssr = "/app/Foo.vue?vue&vize-ssr";
  const before = classifications;

  assert.equal(classifyImporterRequest(first, undefined), null);
  assert.equal(classifyImporterRequest(first, ""), null);
  assert.equal(first.viteImporterRequest, undefined);
  assert.equal(classifications, before);

  const firstClient = classifyImporterRequest(first, client);
  const firstSlot = first.viteImporterRequest;
  assert.equal(classifyImporterRequest(first, client), firstClient);
  assert.equal(classifications, before + 1, "an adjacent exact importer crosses native once");

  const secondClient = classifyImporterRequest(second, client);
  assert.notEqual(secondClient, firstClient, "another state owns its own facts");
  assert.equal(classifications, before + 2);

  const firstSsr = classifyImporterRequest(first, ssr);
  assert.notEqual(firstSsr, firstClient, "a different query is a different importer");
  assert.deepEqual(firstSsr, classifyNative(ssr));
  assert.notEqual(first.viteImporterRequest, firstSlot);
  assert.equal(first.viteImporterRequest?.id, ssr);
  assert.equal(classifications, before + 3);

  const revisited = classifyImporterRequest(first, client);
  assert.notEqual(revisited, firstClient, "returning to an evicted importer classifies it again");
  assert.deepEqual(revisited, firstClient);
  assert.equal(classifications, before + 4);
  assert.equal(classifyImporterRequest(second, client), secondClient);
  assert.equal(classifications, before + 4, "the other state's slot survived interleaving");

  const retained = first.viteImporterRequest;
  for (const absent of [undefined, "", undefined]) {
    assert.equal(classifyImporterRequest(first, absent), null);
    assert.equal(first.viteImporterRequest, retained);
  }
  assert.equal(classifications, before + 4, "absent importers never cross native");

  for (let index = 0; index < 64; index++) {
    const importer = `/app/Item${index}.vue?vue&vize`;
    const facts = classifyImporterRequest(first, importer);
    assert.deepEqual(Reflect.ownKeys(first.viteImporterRequest!).sort(), ["id", "request"]);
    assert.equal(first.viteImporterRequest?.id, importer);
    assert.equal(first.viteImporterRequest?.request, facts);
  }
  assert.equal(classifications, before + 68, "each replacement has one native classification");
  assert.equal(classifyImporterRequest(second, client), secondClient);
});

test("SSR queries retain their uncached meaning without mutating or reclassifying supplied facts", () => {
  const state = makeState("/app");
  for (const importer of importers) {
    const expectedSsr = isPluginVisibleSsrVirtualId(importer);
    const request = classifyImporterRequest(state, importer);
    assert.ok(request);
    const snapshot = structuredClone(request);
    Object.freeze(request);
    const before = classifications;

    assert.equal(isPluginVisibleSsrVirtualId(importer, request), expectedSsr, importer);
    assert.equal(classifyImporterRequest(state, importer), request, importer);
    assert.equal(
      classifications,
      before,
      "provided facts and an adjacent importer need no native call",
    );
    assert.deepEqual(request, snapshot, importer);
    assert.deepEqual(request, classifyNative(importer), importer);
  }
});
