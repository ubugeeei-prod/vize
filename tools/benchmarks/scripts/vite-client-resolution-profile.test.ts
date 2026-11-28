import { createHash } from "node:crypto";
import assert from "node:assert/strict";
import { runInNewContext } from "node:vm";
import path from "node:path";
import { test } from "node:test";
import { metadataVariant, reportInput } from "./vite-client-resolution-profile.ts";

const source = [
  "const oncePerBuild = async (_ctx,_state,_key,callback) => callback();",
  "async function cacheEntry(entry) { const ctx=null, state=null;",
  'return (await oncePerBuild(ctx, state, [\n\t\t"project-vue-runtime",],async()=>{',
  "return entry ? { id: entry } : null;",
  "}))?.id ?? null; }",
  "function native(resolved, projectLocalEntry) { const state={logger:{log(){}}};",
  "state.logger.log(`resolveId: resolved project-local Vue runtime to ${projectLocalEntry}`);\n\t\t\t\treturn projectLocalEntry; }",
].join("\n");
const evaluate = (text: string) =>
  runInNewContext(text + ";({cacheEntry,native});", { path: path.win32 }) as {
    cacheEntry(entry: unknown): Promise<unknown>;
    native(result: { id: string }, selected: string): unknown;
  };

void test("the experiment preserves the complete native result only for an identical file", async () => {
  const before = evaluate(source),
    after = evaluate(metadataVariant(source));
  const resolved = {
    id: "/project/vue.js",
    moduleSideEffects: false,
    attributes: { type: "javascript" },
    meta: { owner: "native" },
  };
  assert.equal(before.native(resolved, resolved.id), resolved.id);
  const selected = after.native(resolved, resolved.id);
  assert.equal(await after.cacheEntry(selected), resolved);
  const redirected = after.native(resolved, "/isolated/vue.js");
  assert.equal(redirected, "/isolated/vue.js");
  assert.equal(await after.cacheEntry(redirected), "/isolated/vue.js");
  assert.equal(await after.cacheEntry(null), null);
});

void test("a missing, duplicate or already changed source span refuses the experiment", () => {
  for (const invalid of [
    "",
    source + source,
    source.replace("}))?.id ?? null;", ""),
    metadataVariant(source),
  ])
    assert.throws(() => metadataVariant(invalid));
});

void test("Windows separator normalization preserves the selected id and all native metadata", async () => {
  const after = evaluate(metadataVariant(source));
  const native = {
    id: "C:/project/vue.js",
    moduleSideEffects: false,
    attributes: { type: "javascript" },
    meta: { owner: "native" },
  };
  const selected = String.raw`C:\project\vue.js`;
  const result = (await after.cacheEntry(after.native(native, selected))) as typeof native;
  assert.equal(result.id, selected);
  assert.equal(result.moduleSideEffects, false);
  assert.equal(result.attributes, native.attributes);
  assert.equal(result.meta, native.meta);
});

void test("the measured generator, timing hooks and native observer retain the reporter's exact bytes", () => {
  const expected = {
    "vite.config.ts.txt": "ee6bf2569229bd6983e2f362af20bc3e5f25a9c636e929d7c2bb6998960cd0cf",
    "generate.mjs.txt": "e3a17ecfdf4892ce1d09160b5428f1a613be63958961beed3e4526d425893df3",
    "count-native.mjs.txt": "b2befe77d1465363445403faf6c0d35cf19127ffa7a18452c8d6522058e23279",
    "tsconfig.json.txt": "f669b2786d6071d7ce53cb0046d61d6ac877b530774fcf58c4fe19c4da36e4e1",
    "index.html.txt": "bea87c52c2bf1e851498ae0daded297b9572be1219985163c861e547a17067f1",
  } as const;
  for (const name of Object.keys(expected) as Array<keyof typeof expected>)
    assert.equal(createHash("sha256").update(reportInput(name)).digest("hex"), expected[name]);
});
