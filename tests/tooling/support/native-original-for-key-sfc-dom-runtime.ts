import assert from "node:assert/strict";
import { executeSetupComponent } from "./native-selected-setup-sfc-dom-runtime.ts";

export type ForKeyFixture = {
  id: string;
  source: string;
  script: string;
  template: string;
  bindings: string[];
  immutableBindings: string[];
  updates: Record<string, unknown>;
  forKeyRuntime: { tag: string; initial: string[]; updated: string[] };
};

export function assertForKeyFixture(value: unknown): asserts value is ForKeyFixture {
  assert(value && typeof value === "object" && !Array.isArray(value));
  const fixture = value as Record<string, unknown>;
  for (const field of ["id", "source", "script", "template"]) {
    assert.equal(typeof fixture[field], "string", field);
    assert((fixture[field] as string).length > 0, field);
  }
  const bindings = fixture.bindings;
  const immutableBindings = fixture.immutableBindings;
  assert(Array.isArray(bindings) && bindings.length === 1);
  assert(bindings.every((name: unknown) => typeof name === "string" && name.length > 0));
  assert(Array.isArray(immutableBindings));
  assert(
    immutableBindings.every((name: unknown) => typeof name === "string" && bindings.includes(name)),
  );
  assert(fixture.updates && typeof fixture.updates === "object" && !Array.isArray(fixture.updates));
  for (const name of Object.keys(fixture.updates)) {
    assert(bindings.includes(name) && !immutableBindings.includes(name));
  }
  const expected = fixture.forKeyRuntime;
  assert(expected && typeof expected === "object" && !Array.isArray(expected));
  const vectors = expected as Record<string, unknown>;
  assert.equal(typeof vectors.tag, "string");
  assert((vectors.tag as string).length > 0);
  for (const phase of ["initial", "updated"]) {
    const values = vectors[phase];
    assert(Array.isArray(values));
    assert(values.every((value: unknown) => typeof value === "string"));
    // These independently declared vectors are the zero-based callback keys,
    // including UTF-16 string units; generated code never supplies this oracle.
    assert.deepEqual(
      values,
      values.map((_, index) => String(index)),
    );
  }
}

export async function executeForKeyComponent(
  code: string,
  fixture: ForKeyFixture,
  native: boolean,
) {
  assertForKeyFixture(fixture);
  // This test-only adapter reuses the existing real Vue host's vector checks.
  // It neither rewrites component code nor establishes a compiler read role.
  const result = await executeSetupComponent(
    code,
    { ...fixture, forValueRuntime: fixture.forKeyRuntime },
    native,
  );
  const collection = fixture.bindings[0];
  const patchFlag = fixture.immutableBindings.includes(collection) ? 64 : 256;
  for (const [state, values] of [
    [result.initial, fixture.forKeyRuntime.initial],
    [result.updated, fixture.forKeyRuntime.updated],
  ] as const) {
    assert.deepEqual(state, {
      type: "fragment",
      props: {},
      children: values.map((value) => ({
        type: fixture.forKeyRuntime.tag,
        props: {},
        children: value,
        patchFlag: 1,
      })),
      patchFlag,
    });
  }
  return result;
}
