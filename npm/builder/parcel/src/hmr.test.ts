import assert from "node:assert/strict";
import { test } from "node:test";
import { hmrRuntime, instrumentHmr } from "./hmr.ts";

const runtime = `data:text/javascript,${encodeURIComponent(hmrRuntime)}`;
const shape = {
  hasDefaultExport: true,
  hasSfcMainDefined: false,
  hasNamedRenderExport: false,
  hasNamedSsrRenderExport: false,
  defaultExportIsSfcMain: false,
};
let nonce = 0;

async function evaluate(code: string) {
  const instrumented = await instrumentHmr(code, shape, runtime, "same-host-options");
  assert.notEqual(instrumented, code);
  return import(`data:text/javascript,${encodeURIComponent(instrumented)}#${nonce++}`);
}

void test("identical output and imported snapshots admit CSS-only virtual reevaluation", async () => {
  const api = await import(runtime);
  const code = "export default { setup() { return { n: 1 } } };";
  const before = await evaluate(code);
  const after = await evaluate(code);
  assert.notEqual(before.default, after.default);
  assert.equal(api.same(api.read(before.default), api.read(after.default)), true);
});

void test("unchanged SFC output cannot hide a changed live imported binding", async () => {
  const provider = `data:text/javascript,${encodeURIComponent("export let value=1; export function update(next) { value=next; }")}`;
  const dependency = await import(provider);
  const api = await import(runtime);
  const code = `import { value } from ${JSON.stringify(provider)}; export default { value };`;
  const before = await evaluate(code);
  dependency.update(2);
  const after = await evaluate(code);
  assert.equal(api.read(before.default).imports[0][1], 1);
  assert.equal(api.read(after.default).imports[0][1], 2);
  assert.equal(api.same(api.read(before.default), api.read(after.default)), false);
});

void test("namespace snapshots capture values rather than retain a live namespace", async () => {
  const provider = `data:text/javascript,${encodeURIComponent("export let value=3; export function update(next) { value=next; }")}`;
  const dependency = await import(provider);
  const api = await import(runtime);
  const code = `import * as values from ${JSON.stringify(provider)}; export default { value: values.value };`;
  const before = await evaluate(code);
  dependency.update(4);
  const after = await evaluate(code);
  assert.equal(api.same(api.read(before.default), api.read(after.default)), false);
});

void test("named default functions and classes keep their authored local declaration", async () => {
  for (const code of [
    "export default function Example() {} export const original=Example;",
    "export default class Example {} export const original=Example;",
  ]) {
    const result = await evaluate(code);
    assert.equal(result.default, result.original);
  }
});

void test("Unicode and escaped private-name collisions do not change authored bindings", async () => {
  const result = await evaluate(
    "// 😺\r\nconst __vize_parcel_\\u0072emember=7; const __vize_parcel_component=8; export default { café: 9, original: __vize_parcel_remember + __vize_parcel_component }; export const later=10;",
  );
  assert.deepEqual(result.default, { café: 9, original: 15 });
  assert.equal(result.later, 10);
});

void test("side effects, dynamic imports and malformed output disable state preservation", async () => {
  for (const code of [
    "import './side-effect.js'; export default {};",
    "export * from './side-effect.js'; export default {};",
    "export { value } from './side-effect.js'; export default {};",
    "export default { later: () => import('./dynamic.js') };",
    "export default { invalid",
  ]) {
    assert.equal(await instrumentHmr(code, shape, runtime, "host"), code);
  }
});
