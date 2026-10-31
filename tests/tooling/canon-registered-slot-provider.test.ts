import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { slotProviderTypes } from "./support/canon-slot-provider-types.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = `
export type PropertyKey = never;
// @ts-ignore Exact compiler-error entry, independent of a missing registry key.
type ErrorProvider = typeof missingProvider;
type Registry = {
  Regular: any;
  Error: ErrorProvider;
  Typed: new () => { $slots: { default(props: { item: number }): unknown } };
};
type SlotIsAny<T> = 0 extends 1 & T ? true : false;
type SlotReport<T> = SlotIsAny<T> extends true ? 'any' : 'number';
declare function slotReport<T>(value: T): SlotReport<T>;
declare function expectNumber(value: 'number'): void;
export function regular({ item }: $SlotPayload<any, 'default', $RegisteredSlotProvider<Registry, 'Regular'>>) { expectNumber(slotReport(item)); }
export function erroneous({ item }: $SlotPayload<any, 'default', $RegisteredSlotProvider<Registry, 'Error'>>) { expectNumber(slotReport(item)); }
export function missing({ item }: $SlotPayload<any, 'default', $RegisteredSlotProvider<Registry, 'Absent'>>) { expectNumber(slotReport(item)); }
export function fallback({ item }: $SlotPayload<any, 'default', $RegisteredSlotProvider<Registry, 'Absent', 'Regular'>>) { expectNumber(slotReport(item)); }
export function primary({ item }: $SlotPayload<{ default(props: { item: number }): unknown }, 'default', $RegisteredSlotProvider<Registry, 'Typed', 'Regular'>>) { expectNumber(slotReport(item)); }
export function unknownRegistry({ item }: $SlotPayload<any, 'default', $RegisteredSlotProvider<unknown, 'Regular'>>) { expectNumber(slotReport(item)); }
export function anyRegistry({ item }: $SlotPayload<any, 'default', $RegisteredSlotProvider<any, 'Regular'>>) { expectNumber(slotReport(item)); }
export function defaultFallback({ item }: $SlotPayload<any, 'default'>) { expectNumber(slotReport(item)); }
export function dependentDefault({ item }: $SlotPayload<any, 'default', $RegisteredSlotProvider<any, never>>) { expectNumber(slotReport(item)); }
export function authoredFallback({ item }: $SlotPayload<any, 'default', $RegisteredSlotProvider<Registry, 'Absent', 'AlsoAbsent', 'Regular'>>) { expectNumber(slotReport(item)); }
export function previousDefault({ item }: $SlotPayload<any, 'default', $SilentAny>) { expectNumber(slotReport(item)); }
`;

for (const variant of ["embedded", "shared", "published"] as const) {
  test(`${variant} registry ownership preserves exact entry and error identities`, () => {
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
    const observation = slotProviderTypes(
      path.join(root, "tests", `registered-slot-provider-${variant}.ts`),
      declarations + fixture,
      { noUnusedLocals: true, noUnusedParameters: true },
    );
    assert.deepEqual(
      observation.rows
        .filter((row) => row.role === "slot-binding")
        .map(({ flags, any, intrinsicName }) => ({ flags, any, intrinsicName })),
      [
        "any",
        "error",
        "error",
        "any",
        "number",
        "error",
        "error",
        "error",
        "error",
        "any",
        "error",
      ].map((intrinsicName) => ({
        flags: intrinsicName === "number" ? 64 : 1,
        any: intrinsicName !== "number",
        intrinsicName,
      })),
    );
    assert.deepEqual(
      observation.rows
        .filter((row) => row.role === "slot-report")
        .map(({ display, flags, any, intrinsicName }) => ({ display, flags, any, intrinsicName })),
      [
        "any",
        "error",
        "error",
        "any",
        "number",
        "error",
        "error",
        "error",
        "error",
        "any",
        "error",
      ].map((identity) =>
        identity === "error"
          ? { display: "any", flags: 1, any: true, intrinsicName: "error" }
          : { display: `"${identity}"`, flags: 1024, any: false, intrinsicName: null },
      ),
    );
    assert.deepEqual(
      observation.diagnostics.map(({ code, message }) => ({ code, message })),
      Array.from({ length: 3 }, () => ({
        code: 2345,
        message: `Argument of type '"any"' is not assignable to parameter of type '"number"'.`,
      })),
    );
    // Imported-only files reach this helper solely through SlotPayload's
    // default type argument. They must not acquire an unused local alias.
    const imported = slotProviderTypes(
      path.join(root, "tests", `registered-slot-import-only-${variant}.ts`),
      declarations +
        `
        declare const WitnessChild: any;
        type SlotIsAny<T> = 0 extends 1 & T ? true : false;
        type SlotReport<T> = SlotIsAny<T> extends true ? 'any' : 'number';
        declare function slotReport<T>(value: T): SlotReport<T>;
        declare function expectNumber(value: 'number'): void;
        export function imported({ item }: $SlotPayload<any, 'default', typeof WitnessChild>) {
          expectNumber(slotReport(item));
        }
      `,
      { noUnusedLocals: true, noUnusedParameters: true },
    );
    assert.deepEqual(
      imported.diagnostics.map(({ code, message }) => ({ code, message })),
      [
        {
          code: 2345,
          message: `Argument of type '\"any\"' is not assignable to parameter of type '\"number\"'.`,
        },
      ],
    );
    assert.deepEqual(
      imported.rows
        .filter((row) => row.role === "slot-binding")
        .map(({ flags, intrinsicName }) => ({ flags, intrinsicName })),
      [{ flags: 1, intrinsicName: "any" }],
    );
  });
}
