import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { auditPackets } from "../../tools/benchmarks/scripts/canon-slot-constructor-audit.ts";

function fixture() {
  const beforeText = 'historical\nconst q = "name";\nvoid (item)';
  const declarations = [
    "type $SlotConstructorArgumentLists<C> = [];",
    "type $SlotConstructorWithoutArguments<C> = C;",
    "type $SlotConstructorWithArguments<C> = C;",
    "type $NativeSlotsFactory<C> = [C] extends [new () => unknown] ? C : never;",
    ...["Without", "With"].flatMap((name) =>
      Array.from(
        { length: 2 },
        () =>
          `new (undefined as unknown as $SlotConstructor${name}Arguments<typeof WitnessChild>)();`,
      ),
    ),
  ].join("\n");
  const afterText = `${declarations}\nconst q = "name";\nvoid (item)`;
  const rows = (text: string) => {
    const q = text.indexOf('"name"'),
      value = text.lastIndexOf("item");
    return [
      [q, 1, 20, 4, 1, 0],
      [q + 1, 4, 20, 4, 0, 1048575],
      [q + 5, 1, 20, 4, 1, 0],
      [value, 4, 40, 4, 0, 1048575],
    ];
  };
  const before = {
    text: beforeText,
    mappings: rows(beforeText),
    diagnostics: [],
    extension: ".tsx",
  };
  const afterRows = rows(afterText),
    main = afterRows.pop()!;
  const after = {
    text: afterText,
    mappings: [...afterRows, [main[0]! - 1, 1, 40, 4, 1, 0], main, [main[0]! + 4, 1, 40, 4, 1, 0]],
    diagnostics: [],
    extension: ".tsx",
    diagnosticDirectives: {
      directives: [
        [0, 0, 0, afterRows[0]![0], 0, 0],
        [0, 0, afterRows[2]![0]! + 1, main[0]! - 1, 0, 0],
      ],
      unusedExpectDirectiveDiagnostics: [],
    },
  };
  return { before, after };
}
const clone = <T>(value: T): T => structuredClone(value);
void test("the constructor audit admits only the certified additive wrapper transition", () => {
  const { before, after } = fixture();
  assert.equal(auditPackets(before, after).mappings, 6);
});
void test("the actual constructor audit refuses missing, reordered, foreign and modified rows", () => {
  const { before, after } = fixture();
  for (let row = 0; row < after.mappings.length; row++) {
    const missing = clone(after);
    missing.mappings.splice(row, 1);
    assert.throws(() => auditPackets(before, missing));
    for (let field = 0; field < 6; field++) {
      const changed = clone(after);
      changed.mappings[row]![field]! += 1;
      assert.throws(() => auditPackets(before, changed));
    }
  }
  for (const mutate of [
    (p: typeof after) => {
      p.mappings.splice(3, 1);
    },
    (p: typeof after) => {
      p.mappings.splice(5, 1);
    },
    (p: typeof after) => {
      p.mappings.reverse();
    },
    (p: typeof after) => {
      p.mappings.splice(3, 0, p.mappings[3]!);
    },
    (p: typeof after) => {
      p.mappings[3]![2] = 20;
      p.mappings[5]![2] = 20;
    },
    (p: typeof after) => {
      p.text = p.text.slice(0, -1) + "]";
    },
    (p: typeof after) => {
      p.extension = ".ts";
    },
    (p: typeof after) => {
      p.diagnostics.push({ code: 1 } as never);
    },
  ]) {
    const changed = clone(after);
    mutate(changed);
    assert.throws(() => auditPackets(before, changed));
  }
  assert.throws(() => auditPackets(before, { ...after, unknownField: true }));
  assert.throws(() =>
    auditPackets({ ...before, unknownField: true }, { ...after, unknownField: true }),
  );
  const producer = fs.readFileSync(
    new URL("../../crates/vize_canon/src/batch/virtual_project/content_mapper.rs", import.meta.url),
  );
  assert.throws(
    () => auditPackets(before, after, "WitnessChild", Buffer.concat([producer, Buffer.from("\n")])),
    /unqualified #11675/u,
  );
});
