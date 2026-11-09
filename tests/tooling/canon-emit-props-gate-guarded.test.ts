// A `v-if` use calls the factory before its guard; the guarded gated call
// must still have exactly the guarded direct call's type for every shape.
import assert from "node:assert/strict";
import { test } from "node:test";
import {
  helpers,
  GATE_ALIASES,
  prelude,
  shapes,
  literalsFor,
  guardedUsage,
  diagnostics,
} from "./support/emit-props-gate-fixture.ts";

void test("a guarded gated call has exactly the guarded direct call's type", () => {
  const lines = [
    ...prelude,
    helpers,
    GATE_ALIASES.replaceAll("{emit_props_factory}", "$EmitPropsFactory"),
    ...Object.entries(shapes).flatMap(([name, shape]) =>
      literalsFor(name).flatMap((literal, index) => guardedUsage(name, shape, literal, index)),
    ),
  ].join("\n");
  for (const exactOptionalPropertyTypes of [false, true]) {
    assert.deepEqual(
      diagnostics(lines, exactOptionalPropertyTypes),
      [],
      `exactOptionalPropertyTypes=${exactOptionalPropertyTypes}`,
    );
  }
});
