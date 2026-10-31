import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { slotProviderTypes } from "./support/canon-slot-provider-types.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = `
declare const WitnessChild: any;
// @ts-ignore Independent compiler-error control.
type ErrorProvider = typeof missingProvider;
type SlotIsAny<T> = 0 extends 1 & T ? true : false;
type SlotReport<T> = SlotIsAny<T> extends true ? 'any' : 'number';
declare function slotReport<T>(value: T): SlotReport<T>;
declare function expectNumber(value: 'number'): void;
function declared({ item }: $SlotPayload<any, 'default', typeof WitnessChild>) { expectNumber(slotReport(item)); }
function erroneous({ item }: $SlotPayload<any, 'default', ErrorProvider>) { expectNumber(slotReport(item)); }
function unresolved({ item }: $SlotPayload<any, 'default'>) { expectNumber(slotReport(item)); }
function openTyped({ item }: $SlotPayload<any, 'default', new () => { $slots: any }>) { expectNumber(slotReport(item)); }
function typed({ item }: $SlotPayload<{ default(props: { item: number }): unknown }, 'default', typeof WitnessChild>) { expectNumber(slotReport(item)); }
function broad({ item }: $SlotPayload<{ default(props: { item: number }): unknown }, string>) { expectNumber(slotReport(item)); }
`;

for (const variant of ["embedded", "shared", "published"] as const) {
  test(`${variant} slot payload retains declared and intrinsic error-any identities`, () => {
    const helpers = path.join(root, "crates/vize_canon/src/virtual_ts/helpers/declarations");
    let declarations: string;
    if (variant === "published") {
      const text = readFileSync(path.join(root, "npm/cli/virtual-ts.d.ts"), "utf8");
      const source = ts.createSourceFile("virtual-ts.d.ts", text, ts.ScriptTarget.Latest, true);
      const names = new Set([
        "$SilentAny",
        "$RegisteredSlotProvider",
        "$NamedSlotPayload",
        "$SlotPayload",
      ]);
      const aliases = source.statements.filter(
        (node): node is ts.TypeAliasDeclaration =>
          ts.isTypeAliasDeclaration(node) && names.has(node.name.text),
      );
      assert.equal(aliases.length, names.size);
      declarations = aliases.map((node) => node.getFullText(source)).join("\n");
    } else if (variant === "shared") {
      declarations = readFileSync(path.join(helpers, "shared-slot-payload.d.ts.txt"), "utf8");
      declarations = declarations.slice(
        declarations.indexOf("{") + 1,
        declarations.lastIndexOf("}"),
      );
    } else declarations = readFileSync(path.join(helpers, "slot-payload.d.ts.txt"), "utf8");
    declarations = `type $IsAny<T> = 0 extends 1 & T ? true : false;\n${declarations}`;
    const text = declarations + fixture;
    const observation = slotProviderTypes(
      path.join(root, "tests", `slot-provider-${variant}.ts`),
      text,
    );
    const bindings = observation.rows.filter((row) => row.role === "slot-binding");
    assert.deepEqual(
      bindings.map(({ display, any, intrinsicName }) => ({ display, any, intrinsicName })),
      [
        { display: "any", any: true, intrinsicName: "any" },
        { display: "any", any: true, intrinsicName: "error" },
        { display: "any", any: true, intrinsicName: "error" },
        { display: "any", any: true, intrinsicName: "any" },
        { display: "number", any: false, intrinsicName: "number" },
        { display: "any", any: true, intrinsicName: "any" },
      ],
    );
    assert.deepEqual(
      observation.rows
        .filter((row) => row.role === "slot-report")
        .map(({ display, any, intrinsicName }) => ({ display, any, intrinsicName })),
      [
        { display: '"any"', any: false, intrinsicName: null },
        { display: "any", any: true, intrinsicName: "error" },
        { display: "any", any: true, intrinsicName: "error" },
        { display: '"any"', any: false, intrinsicName: null },
        { display: '"number"', any: false, intrinsicName: null },
        { display: '"any"', any: false, intrinsicName: null },
      ],
    );
    const diagnostics = observation.diagnostics.filter((row) => row.file === observation.file);
    assert.deepEqual(
      diagnostics.map(({ code, message }) => ({ code, message })),
      Array.from({ length: 3 }, () => ({
        code: 2345,
        message: `Argument of type '"any"' is not assignable to parameter of type '"number"'.`,
      })),
    );
  });
}
