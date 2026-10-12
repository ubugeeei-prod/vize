import assert from "node:assert/strict";

export const originalNestedSource =
  "<script setup>const value=1</script><template><i>{{value}}</i></template>";
export const originalNestedRefusal = {
  completeModule: false,
  diagnostics: [],
  name: "nested",
  reason:
    "Ssr(SsrError { node: Some(NodeId(1)), span: Span { start: 49, end: 58 }, kind: Unsupported(Operation) })",
  source: originalNestedSource,
};

export function withoutOriginalNested(rows: any[]) {
  assert.equal(rows.length, 20);
  assert.equal(rows[15].name, "nested");
  assert.equal(rows[15].source, originalNestedSource);
  if (Object.hasOwn(rows[15], "reason")) assert.deepEqual(rows[15], originalNestedRefusal);
  const successor = [...rows.slice(0, 15), ...rows.slice(16)];
  assert.deepEqual([...successor.slice(0, 15), rows[15], ...successor.slice(15)], rows);
  return successor;
}

export function directChildHistoricalSuccessor(packet: any) {
  return { ...packet, refusals: withoutOriginalNested(packet.refusals) };
}

// JSON cannot retain these exact pinned provider carriers. Reify the expected
// side only; the complete live runtime remains under strict equality.
export function directChildRuntimeExpectation(frozen: any, sourceMapPrototype: object) {
  assert.deepEqual(
    frozen.rows.map((row: any) => row.name),
    [
      "original-nested",
      "nested-let-js",
      "nested-ts-let",
      "nested-unicode-var",
      "nested-const-attrs",
      "nested-ts-literal",
      "nested-literal-escaping",
      "nested-retained-comments",
    ],
  );
  const expected = structuredClone(frozen);
  for (const index of [0, 1, 3, 4, 6, 7]) {
    const map = expected.rows[index].external.map;
    assert.equal(Object.getPrototypeOf(map), Object.prototype);
    assert.equal(Object.hasOwn(map, "file"), false);
    assert.equal(Object.hasOwn(map, "sourceRoot"), false);
    map.file = undefined;
    map.sourceRoot = undefined;
  }
  for (const index of [2, 5]) {
    const dependencies = expected.rows[index].external.dependencies;
    assert.equal(dependencies.length, 1);
    const map = dependencies[0].originalMap;
    assert.equal(Object.getPrototypeOf(map), Object.prototype);
    assert.equal(Object.hasOwn(map, "file"), false);
    map.file = undefined;
    Object.setPrototypeOf(map, sourceMapPrototype);
  }
  return expected;
}

export function assertDirectChildRuntime(actual: any, frozen: any, sourceMapPrototype: object) {
  const expected = directChildRuntimeExpectation(frozen, sourceMapPrototype);
  const shape = (a: any, b: any) => {
    assert.equal(typeof a, typeof b);
    if (b === null || typeof b !== "object") return;
    assert.equal(Object.getPrototypeOf(a), Object.getPrototypeOf(b));
    assert.deepEqual(new Set(Reflect.ownKeys(a)), new Set(Reflect.ownKeys(b)));
    for (const key of Reflect.ownKeys(b)) {
      const actualDescriptor = Object.getOwnPropertyDescriptor(a, key)!;
      const expectedDescriptor = Object.getOwnPropertyDescriptor(b, key)!;
      assert.deepEqual(
        { ...actualDescriptor, value: undefined },
        { ...expectedDescriptor, value: undefined },
      );
      shape(actualDescriptor.value, expectedDescriptor.value);
    }
  };
  shape(actual, expected);
  assert.deepEqual(actual, expected);
}
