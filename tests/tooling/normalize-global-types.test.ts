import assert from "node:assert/strict";
import { test } from "node:test";

import { normalizeGlobalTypes } from "../../npm/vize/src/config.ts";

/**
 * `normalizeGlobalTypes` is a pure, IO-free helper exported from
 * npm/vize/src/config.ts. It expands shorthand string declarations into
 * `{ type }` objects and, when the input is a single-key `{ types: {...} }`
 * wrapper (plain non-array object), unwraps that nested map before expanding.
 *
 * These are characterization tests of the current implementation: every
 * expected value below was observed by running the real function against the
 * fixtures, so they double as regression guards.
 */

test("expands string shorthands to {type} and preserves object form", () => {
  // A bare string value is shorthand for a GlobalTypeDeclaration { type }.
  assert.deepEqual(normalizeGlobalTypes({ Foo: "FooType" }), {
    Foo: { type: "FooType" },
  });

  // A value that is already an object passes through unchanged (including any
  // extra fields such as defaultValue).
  assert.deepEqual(normalizeGlobalTypes({ Bar: { type: "BarType", defaultValue: "x" } }), {
    Bar: { type: "BarType", defaultValue: "x" },
  });
});

test("unwraps a nested {types:{...}} object wrapper", () => {
  // When the top-level config is a wrapper whose `types` key holds a plain
  // (non-array, non-null) object, that nested map is used as the source and
  // each entry is normalized: strings expand, objects pass through.
  assert.deepEqual(
    normalizeGlobalTypes({
      types: { A: "AT", B: { type: "BT", defaultValue: "1" } },
    }),
    {
      A: { type: "AT" },
      B: { type: "BT", defaultValue: "1" },
    },
  );
});

test("treats a `types` array/string value as NOT a wrapper", () => {
  // Wrapper detection requires the `types` value to be a non-array object.
  // An array value is not a wrapper, so `types` is treated as an ordinary
  // global-type name. Its value is an array (not a string), so it passes
  // through unchanged.
  assert.deepEqual(normalizeGlobalTypes({ types: ["A", "B"] }), {
    types: ["A", "B"],
  });

  // A string `types` value is likewise an ordinary entry, and being a string
  // it expands to the { type } shorthand form.
  assert.deepEqual(normalizeGlobalTypes({ types: "S" }), {
    types: { type: "S" },
  });
});

test("a top-level key literally named `type` is shorthand-expanded, not a wrapper", () => {
  // Only the plural `types` key triggers unwrapping. The singular `type` is an
  // ordinary global-type name; its string value expands to { type }.
  assert.deepEqual(normalizeGlobalTypes({ type: "X" }), {
    type: { type: "X" },
  });
});
