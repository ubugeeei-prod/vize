import assert from "node:assert/strict";
import { test } from "node:test";
import { instrumentHmr } from "./hmr.ts";
import { read } from "./hmr-runtime.ts";

const runtime = new URL("./hmr-runtime.ts", import.meta.url).href;
const identity = "same-component-request";
const shape = {
  hasDefaultExport: true,
  hasSfcMainDefined: false,
  hasNamedRenderExport: false,
  hasNamedSsrRenderExport: false,
  defaultExportIsSfcMain: false,
};
let nonce = 0;

async function evaluate(code: string) {
  const instrumented = await instrumentHmr(code, shape, runtime, identity);
  assert.notEqual(instrumented, code);
  return import(`data:text/javascript,${encodeURIComponent(instrumented)}#${nonce++}`);
}

void test("an actual reevaluation changes the token even for the same exported object", async () => {
  const provider = `data:text/javascript,${encodeURIComponent("export const component = {};")}`;
  const code = `import { component } from ${JSON.stringify(provider)}; export default component;`;
  const before = await evaluate(code);
  const token = read(before.default, identity);
  assert.ok(token);
  const after = await evaluate(code);
  assert.equal(before.default, after.default);
  assert.notEqual(read(after.default, identity), token);
});

void test("unchanged module instances keep the exact same captured token", async () => {
  const result = await evaluate("export default { setup() { return { n: 1 } } };");
  const token = read(result.default, identity);
  assert.ok(token);
  assert.equal(read(result.default, identity), token);
});

void test("named default functions and classes keep their authored local declaration", async () => {
  for (const code of [
    "export default function Example() {} export const original=Example;",
    "export default class Example {} export const original=Example;",
  ]) {
    const result = await evaluate(code);
    assert.equal(result.default, result.original);
    assert.ok(read(result.default, identity));
  }
});

void test("Unicode and escaped private-name collisions preserve exports and bindings", async () => {
  const result = await evaluate(
    "// 😺\r\nconst __vize_parcel_\\u0072emember=7; const __vize_parcel_component=8; export default { café: 9, original: __vize_parcel_remember + __vize_parcel_component }; export const later=10;",
  );
  assert.deepEqual(result.default, { café: 9, original: 15 });
  assert.equal(result.later, 10);
  assert.deepEqual(Object.keys(result), ["default", "later"]);
});

void test("side-effect imports, re-exports and dynamic imports retain actual evaluation tokens", async () => {
  const provider = `data:text/javascript,${encodeURIComponent("export const value = 4;")}`;
  for (const code of [
    `import ${JSON.stringify(provider)}; export default {};`,
    `export * from ${JSON.stringify(provider)}; export default {};`,
    `export { value } from ${JSON.stringify(provider)}; export default {};`,
    `export default { later: () => import(${JSON.stringify(provider)}) };`,
  ]) {
    const result = await evaluate(code);
    assert.ok(read(result.default, identity));
  }
});

void test("malformed output and unsupported module shapes receive no fabricated token", async () => {
  const code = "export default { invalid";
  assert.equal(await instrumentHmr(code, shape, runtime, identity), code);
  const namedOnly = "export const unrelated = 1;";
  assert.equal(await instrumentHmr(namedOnly, undefined, runtime, identity), namedOnly);
});
