import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import type * as TS from "typescript";
import {
  assertCurrentSfcModulePackets,
  sfcModuleSuccessors as fixture,
} from "./support/sfc-style-raw-module-successors.ts";
import { assertVdomChildStylePackets } from "./support/jsx-vdom-child-style-successors.ts";
const ts: typeof TS = createRequire(new URL("../../npm/cli/package.json", import.meta.url))(
  process.env.VIZE_STYLE_AST_TYPESCRIPT ?? "typescript",
);
const presentation = new Set([
  "pos",
  "end",
  "id",
  "kind",
  "flags",
  "transformFlags",
  "modifierFlagsCache",
  "singleQuote",
  "multiLine",
]);
function shape(node: TS.Node): unknown {
  if (ts.isParenthesizedExpression(node)) return shape(node.expression);
  if (
    ts.isShorthandPropertyAssignment(node) &&
    !node.objectAssignmentInitializer &&
    node.name.text !== "__proto__"
  )
    return {
      kind: ts.SyntaxKind.PropertyAssignment,
      fields: {},
      children: [shape(node.name), shape(node.name)],
    };
  if (ts.isSourceFile(node))
    return { kind: node.kind, fields: {}, children: node.statements.map(shape) };
  const fields = Object.fromEntries(
    Object.entries(node).filter(
      ([key, value]) =>
        !presentation.has(key) && ["string", "boolean", "number"].includes(typeof value),
    ),
  );
  const children: unknown[] = [];
  ts.forEachChild(node, (child) => {
    children.push(shape(child));
  });
  return { kind: node.kind, fields, children };
}
function moduleShape(source: string) {
  assert.deepEqual(
    ts.transpileModule(source, {
      fileName: "module.js",
      reportDiagnostics: true,
      compilerOptions: {
        allowJs: true,
        target: ts.ScriptTarget.Latest,
        module: ts.ModuleKind.ESNext,
      },
    }).diagnostics,
    [],
  );
  return shape(
    ts.createSourceFile("module.js", source, ts.ScriptTarget.Latest, false, ts.ScriptKind.JS),
  );
}
test("four raw SFC module spellings retain complete independent syntax trees and all non-code fields", () => {
  assert.equal(fixture.qualificationCredit, false);
  assert.equal(fixture.current.length, 4);
  assertVdomChildStylePackets("sfc", fixture.previous);
  assert.deepEqual(assertCurrentSfcModulePackets(fixture.current), fixture.previous);
  for (const [index, current] of fixture.current.entries()) {
    const previous = fixture.previous[index]!;
    const { code: before, ...beforeFields } = previous.output;
    const { code: after, ...afterFields } = current.output;
    assert.equal(typeof before, "string");
    assert.equal(typeof after, "string");
    assert.deepEqual(moduleShape(after as string), moduleShape(before as string));
    assert.deepEqual(beforeFields, afterFields);
    assert.deepEqual({ ...previous, output: current.output }, current);
  }
});
test("every actual code, input and opaque-field change is rejected", () => {
  for (let index = 0; index < 4; index++) {
    for (const mutate of [
      (r: Record<string, unknown> & { output: Record<string, unknown> }) => {
        r.output.code += "\nthrow 0;";
      },
      (r: Record<string, unknown>) => {
        r.source += "\n";
      },
      (r: Record<string, unknown> & { output: Record<string, unknown> }) => {
        r.output.foreign = true;
      },
    ]) {
      const changed = structuredClone(fixture.current);
      mutate(changed[index]!);
      assert.throws(() => assertCurrentSfcModulePackets(changed));
    }
  }
  assert.throws(() => assertCurrentSfcModulePackets(fixture.current.slice(1)));
});
