import assert from "node:assert/strict";
import { test } from "node:test";
import type { ModuleShapeNapi } from "@vizejs/native";
import { hmrRuntime, instrumentHmr } from "./hmr.ts";

const runtime = `data:text/javascript,${encodeURIComponent(hmrRuntime)}`;
let nonce = 0;

function shapeFor(code: string, override: Partial<ModuleShapeNapi> = {}): ModuleShapeNapi {
  const marker = "export default";
  const start = code.indexOf(marker);
  const statementEnd = start < 0 ? -1 : code.indexOf(";", start);
  return {
    hasDefaultExport: start >= 0,
    hasSfcMainDefined: false,
    hasNamedRenderExport: false,
    hasNamedSsrRenderExport: false,
    defaultExportStart: start < 0 ? undefined : Buffer.byteLength(code.slice(0, start)),
    defaultExportKeywordEnd:
      start < 0 ? undefined : Buffer.byteLength(code.slice(0, start + marker.length)),
    defaultExportEnd:
      start < 0
        ? undefined
        : Buffer.byteLength(code.slice(0, statementEnd < 0 ? code.length : statementEnd + 1)),
    defaultExportIsSfcMain: false,
    imports: [],
    hasSideEffectImport: false,
    hasReexport: false,
    hasDynamicImport: false,
    ...override,
  };
}

async function evaluate(code: string, override: Partial<ModuleShapeNapi> = {}) {
  const shape = shapeFor(code, override);
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
  const importShape = { imports: [{ source: provider, local: "value", namespace: false }] };
  const before = await evaluate(code, importShape);
  dependency.update(2);
  const after = await evaluate(code, importShape);
  assert.equal(api.read(before.default).imports[0][1], 1);
  assert.equal(api.read(after.default).imports[0][1], 2);
  assert.equal(api.same(api.read(before.default), api.read(after.default)), false);
});

void test("namespace snapshots capture values rather than retain a live namespace", async () => {
  const provider = `data:text/javascript,${encodeURIComponent("export let value=3; export function update(next) { value=next; }")}`;
  const dependency = await import(provider);
  const api = await import(runtime);
  const code = `import * as values from ${JSON.stringify(provider)}; export default { value: values.value };`;
  const importShape = { imports: [{ source: provider, local: "values", namespace: true }] };
  const before = await evaluate(code, importShape);
  dependency.update(4);
  const after = await evaluate(code, importShape);
  assert.equal(api.same(api.read(before.default), api.read(after.default)), false);
});

void test("named default functions and classes keep their authored local declaration", async () => {
  for (const code of [
    "export default function Example() {} export const original=Example;",
    "export default class Example {} export const original=Example;",
  ]) {
    const result = await evaluate(code, { defaultExportLocal: "Example" });
    assert.equal(result.default, result.original);
  }
});

void test("Unicode byte offsets and private-name collisions preserve authored bindings", async () => {
  const result = await evaluate(
    "// 😺\r\nconst __vize_parcel_remember=7; const __vize_parcel_component=8; export default { café: 9, original: __vize_parcel_remember + __vize_parcel_component }; export const later=10;",
  );
  assert.deepEqual(result.default, { café: 9, original: 15 });
  assert.equal(result.later, 10);
});

void test("unsafe native facts and stale geometry disable state preservation", async () => {
  const cases: Array<[string, Partial<ModuleShapeNapi>]> = [
    ["import './side-effect.js'; export default {};", { hasSideEffectImport: true }],
    ["export * from './side-effect.js'; export default {};", { hasReexport: true }],
    ["export { value } from './side-effect.js'; export default {};", { hasReexport: true }],
    ["export default { later: () => import('./dynamic.js') };", { hasDynamicImport: true }],
    ["const __vize_parcel_\\u0072emember=7; export default {};", {}],
    ["export default { invalid", { defaultExportEnd: undefined }],
  ];
  for (const [code, override] of cases) {
    assert.equal(await instrumentHmr(code, shapeFor(code, override), runtime, "host"), code);
  }
  assert.equal(
    await instrumentHmr("export default {};", undefined, runtime, "host"),
    "export default {};",
  );
});
