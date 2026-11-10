import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { auditPackets } from "../../tools/benchmarks/scripts/canon-slot-constructor-audit.ts";
import { assertConstructorWrapperMappings } from "../../tools/benchmarks/scripts/canon-slot-constructor-wrapper-mappings.ts";

function fixture() {
  const beforeText = 'historical\nconst q = "name";\nvoid (item)';
  const declarations = [
    "type $SlotConstructorArgumentLists<C> = [];",
    "type $SlotConstructorWithoutArguments<C> = C;",
    "type $SlotConstructorWithArguments<C> = C;",
    "type $NativeSlotsFactory<C> = [C] extends [new () => unknown] ? C : never;",
    ...["Without", "With"].map(
      (name) =>
        `new (undefined as unknown as $SlotConstructor${name}Arguments<typeof WitnessChild>)();`,
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
    diagnosticDirectives: {
      directives: Array.from({ length: 3 }, () => [
        20,
        4,
        beforeText.indexOf('"name"'),
        beforeText.indexOf('"name"') + 6,
        0,
        0,
      ]),
      unusedExpectDirectiveDiagnostics: [],
    },
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
        [20, 4, afterText.indexOf('"name"'), afterText.indexOf('"name"') + 6, 0, 0],
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
void test("the constructor audit refuses copied slot-host branches", () => {
  const { before, after } = fixture();
  const branch =
    "new (undefined as unknown as $SlotConstructorWithoutArguments<typeof WitnessChild>)();";
  const copied = clone(after);
  copied.text = copied.text.replace(branch, `${branch}\n${branch}`);
  assert.throws(
    () => auditPackets(before, copied),
    /reused template host must retain each direct branch exactly once/u,
  );
});
void test("the constructor audit refuses added or uniquely removed authored directives", () => {
  const { before, after } = fixture();
  const added = clone(after);
  const authored = added.diagnosticDirectives.directives.at(-1)!;
  added.diagnosticDirectives.directives.push(authored, authored, authored);
  assert.throws(() => auditPackets(before, added), /may not add authored directive copies/u);
  const removed = clone(after);
  removed.diagnosticDirectives.directives.pop();
  assert.throws(
    () => auditPackets(before, removed),
    /may not add or remove the only authored directive/u,
  );
});
void test("the constructor audit admits only the exact named component navigation successor", () => {
  const text =
    'const q = "name";\ntype TinyComponent = { readonly tiny: true };\ndeclare const __vize_component__: TinyComponent;\ndeclare const Tiny: typeof __vize_component__;\nexport default Tiny;\n';
  const q = Buffer.from(text).indexOf("name");
  const name = Buffer.from(text).indexOf("Tiny", Buffer.from(text).indexOf("declare const Tiny:"));
  const before = {
    text,
    rows: [[q, 4, 20, 4, 0, 1048575]] as Array<[number, number, number, number, number, number]>,
  };
  const current = {
    text,
    rows: [
      [q, 4, 20, 4, 0, 1048575],
      [name, 4, 0, 0, 1, 248],
    ] as Array<[number, number, number, number, number, number]>,
  };
  const producer = fs.readFileSync(
    new URL("../../crates/vize_canon/src/batch/virtual_project/content_mapper.rs", import.meta.url),
  );
  assertConstructorWrapperMappings(before, current, (value) => value, producer);
  for (const field of [0, 1, 2, 3, 4, 5]) {
    const changed = clone(current);
    changed.rows[1]![field]! += 1;
    assert.throws(() =>
      assertConstructorWrapperMappings(before, changed, (value) => value, producer),
    );
  }
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
void test("the wrapper admission accepts only exact disjoint slot pattern gaps", () => {
  const text = "prefix { item } suffix";
  const pattern = Buffer.from(text).indexOf("{ item }");
  const binding = pattern + 2;
  const before = {
    text,
    rows: [[binding, 4, 42, 4, 0, 1048575]] as Array<
      [number, number, number, number, number, number]
    >,
  };
  const current = {
    text,
    rows: [
      [pattern, 2, 40, 2, 0, 1048575],
      ...before.rows,
      [binding + 4, 2, 46, 2, 0, 1048575],
    ] as Array<[number, number, number, number, number, number]>,
  };
  const producer = fs.readFileSync(
    new URL("../../crates/vize_canon/src/batch/virtual_project/content_mapper.rs", import.meta.url),
  );
  assertConstructorWrapperMappings(before, current, (value) => value, producer);
  for (const row of [0, 2]) {
    for (let field = 0; field < 6; field++) {
      const changed = clone(current);
      changed.rows[row]![field]! += 1;
      assert.throws(() =>
        assertConstructorWrapperMappings(before, changed, (value) => value, producer),
      );
    }
  }
});
void test("the wrapper admission removes only duplicate component-prop hover ownership", () => {
  const text = 'void __vize_props_nav_0.value;\n({ "value": 1 });\n';
  const navigation = Buffer.from(text).indexOf("value");
  const check = Buffer.from(text).lastIndexOf("value");
  const before = {
    text,
    rows: [
      [navigation, 5, 20, 5, 0, 1048575],
      [check, 5, 20, 5, 0, 1048575],
    ] as Array<[number, number, number, number, number, number]>,
  };
  const current = clone(before);
  current.rows[1]![5] &= ~1;
  const producer = fs.readFileSync(
    new URL("../../crates/vize_canon/src/batch/virtual_project/content_mapper.rs", import.meta.url),
  );
  assertConstructorWrapperMappings(before, current, (value) => value, producer);

  for (const mutate of [
    (image: typeof current) => (image.rows[0]![5] &= ~1),
    (image: typeof current) => (image.rows[1]![5] &= ~2),
    (image: typeof current) => (image.rows[1]![2] += 1),
    (image: typeof current) =>
      (image.text = image.text.replace("__vize_props_nav_", "__vize_other_nav_")),
  ]) {
    const changed = clone(current);
    mutate(changed);
    assert.throws(() =>
      assertConstructorWrapperMappings(before, changed, (value) => value, producer),
    );
  }
});
void test("the wrapper admission drops only the rows of a removed unchecked slot replay", () => {
  const replay =
    "\n  // Component props in v-slot scope: #default\n  void function _slot_props_default_7({ item }: Payload) {\n    void item;\n  };\n";
  const head = 'const q = "name";\n';
  const tail = "void item;\n";
  const rows = (text: string) => {
    const at = (needle: string, from = 0) => Buffer.from(text).indexOf(needle, from);
    const binding = at("{ item }") + 2;
    return [
      [at("name"), 4, 20, 4, 0, 1048575],
      ...(binding > 1 ? [[binding, 4, 40, 4, 0, 1048575]] : []),
      [text.lastIndexOf("item"), 4, 60, 4, 0, 1048575],
    ] as Array<[number, number, number, number, number, number]>;
  };
  const before = { text: head + replay + tail, rows: rows(head + replay + tail) };
  const current = { text: head + "\n" + tail, rows: rows(head + "\n" + tail) };
  const producer = fs.readFileSync(
    new URL("../../crates/vize_canon/src/batch/virtual_project/content_mapper.rs", import.meta.url),
  );
  assertConstructorWrapperMappings(before, current, (value) => value, producer);
  // The replay must be gone, and every row outside it must stay.
  assert.throws(
    () => assertConstructorWrapperMappings(before, before, (value) => value, producer),
    /no unchecked slot replay remains/u,
  );
  const missing = clone(current);
  missing.rows.pop();
  assert.throws(() =>
    assertConstructorWrapperMappings(before, missing, (value) => value, producer),
  );
});
