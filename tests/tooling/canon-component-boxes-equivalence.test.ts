import assert from "node:assert/strict";
import { test } from "node:test";
import { GATE_ALIASES } from "./support/emit-props-gate-fixture.ts";
import {
  BOX_HELPERS,
  BOX_TEMPLATES,
  boxCases,
  checkBoxes,
  helperDeclarations,
  readSource,
} from "./support/component-boxes-oracle.ts";

// The inline boxes the generator still emits outside the shared contract are
// the oracle: each `$Component*` helper must have exactly their types and
// printed text for every component shape, `any`, `never` and unions included.
const asset = readSource(
  "crates/vize_canon/src/virtual_ts/helpers/declarations/component-checks.d.ts.txt",
);
const declarations = helperDeclarations(asset);

void test("the oracle reads the generator's own text", () => {
  for (const [name, [file, template]] of Object.entries(BOX_TEMPLATES))
    assert.ok(readSource(file).includes(template), `${name} is no longer the generator's text`);
});

void test("every shared box has exactly the inline box's type and printed text", () => {
  const { content, pairs } = boxCases(declarations);
  assert.ok(pairs.length >= 200, `${pairs.length} pairs`);
  for (const exactOptionalPropertyTypes of [false, true]) {
    const { diagnostics, differences } = checkBoxes(content, pairs, exactOptionalPropertyTypes);
    assert.deepEqual(diagnostics, [], `exactOptionalPropertyTypes=${exactOptionalPropertyTypes}`);
    assert.deepEqual(differences, [], `exactOptionalPropertyTypes=${exactOptionalPropertyTypes}`);
  }
});

void test("the oracle rejects a box that changes a component", () => {
  for (const [name, helper, from, to] of [
    [
      "distributes over a union",
      BOX_HELPERS.props,
      "[C] extends [{ new (...args: any[]): { $props: infer __P } }]",
      "C extends { new (...args: any[]): { $props: infer __P } }",
    ],
    [
      "drops the any branch union",
      BOX_HELPERS.emitProps,
      "? (C extends { __vizeEmitProps?: infer __EP } ? [__EP] : [])",
      "? []",
    ],
    [
      "maps the root keys homomorphically",
      BOX_HELPERS.nativeFallthrough,
      "keyof (C & {})",
      "keyof C",
    ],
    [
      "reads the root keys as a key type",
      BOX_HELPERS.nativeFallthrough,
      "keyof (C & {})",
      "keyof C & PropertyKey",
    ],
    [
      "reads the events of a different argument",
      BOX_HELPERS.eventsResolver,
      ": (props: any) => $ComponentEvents<C>;",
      ": (props: any) => $ComponentEvents<unknown>;",
    ],
  ] as const) {
    const line = declarations.split("\n").find((entry) => entry.startsWith(`type ${helper}<`))!;
    const broken = declarations.replace(line, line.replace(from, to));
    assert.notEqual(broken, declarations, name);
    const { content, pairs } = boxCases(broken);
    const { diagnostics, differences } = checkBoxes(content, pairs, false);
    assert.ok(diagnostics.length > 0 && differences.length > 0, name);
  }
});

void test("the shared emit gate is the per-module gate aliases' own text", () => {
  const shared = GATE_ALIASES.replaceAll("__vize_emit_native_input", "$EmitGateNativeInput")
    .replaceAll("type __vize_emit_resolver", "export type $EmitGateResolver")
    .replaceAll("type __vize_emit_input", "export type $EmitGateInput")
    .replaceAll("{emit_props_factory}", "$EmitPropsFactory")
    .trimEnd()
    .split("\n");
  assert.equal(shared.length, 3);
  for (const line of shared)
    assert.equal(asset.split("\n").filter((entry) => entry === line).length, 1, line);
  for (const name of ["__vize_emit_native_input", "__vize_emit_resolver", "__vize_emit_input"])
    assert.ok(!shared.join("\n").includes(name), name);
});
