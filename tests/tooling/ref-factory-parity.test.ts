import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import { expectedRefFactoryScriptErrors } from "../../tools/benchmarks/scripts/ref-factory-native-qualification.ts";
import { readRefFactoryAuthority } from "../../tools/benchmarks/scripts/ref-factory-authority.ts";
import {
  compareFactoryPacket,
  factoryCodeEdits,
  factoryForward,
  type FactoryPacket,
} from "../../tools/benchmarks/scripts/ref-factory-parity.ts";

const oldFactory = "  " + readRefFactoryAuthority().declaration.wholeUtf8;

test("owned script negatives retain complete concise JSON messages and exact authored coordinates", () => {
  const probe = fs.readFileSync(
    "tests/_fixtures/differential/typechecker/shared-ref-factory/Probe.vue.txt",
    "utf8",
  );
  assert.deepEqual(expectedRefFactoryScriptErrors(probe, null), []);
  assert.deepEqual(
    expectedRefFactoryScriptErrors(
      probe.replace("const numeric: number", "const numeric: string"),
      "numeric",
    ),
    [
      {
        file: "src/Probe.vue",
        line: 9,
        column: 7,
        code: 2322,
        message: "Type 'number' is not assignable to type 'string'.",
      },
    ],
  );
  assert.deepEqual(expectedRefFactoryScriptErrors(probe, "noAny"), [
    {
      file: "src/Probe.vue",
      line: 8,
      column: 7,
      code: 2322,
      message: "Type 'true' is not assignable to type 'false'.",
    },
  ]);
  const declaration = "const numeric: number = child.value!.current()";
  assert.equal(probe.split(declaration).length, 2);
  for (const foreign of [
    probe.replace(declaration, ""),
    probe + "\n" + declaration,
    probe.replace(declaration, "") + "\n" + declaration,
  ]) {
    assert.throws(() => expectedRefFactoryScriptErrors(foreign, "numeric"));
  }
});
function packet(factory: string, importDelta = 0): FactoryPacket {
  const beginning = "// 雪🌸\n";
  const prefix = beginning + "x".repeat(9600 - Buffer.byteLength(beginning));
  const first = "const value=1;\n",
    second = "const other=value;\n";
  const firstStart = Buffer.byteLength(prefix + factory),
    firstEnd = firstStart + first.length;
  const comment = `// @vize-map: ${firstStart - importDelta}:${firstEnd - importDelta} -> 0:${first.length}\n`;
  const secondStart = Buffer.byteLength(prefix + factory + first + comment);
  const code = prefix + factory + first + comment + second;
  const rows = [
    [firstStart, firstEnd, 0, first.length],
    [secondStart, secondStart + second.length, first.length, first.length + second.length],
  ].map(([start, end, authoredStart, authoredEnd]) => ({
    generated: { start: start - importDelta, end: end - importDelta },
    authored: { start: authoredStart, end: authoredEnd },
    subSpans: [
      {
        generated: { start: start + 6 - importDelta, end: start + 7 - importDelta },
        authored: { start: authoredStart + 6, end: authoredStart + 7 },
      },
    ],
    features: 255,
    kind: "Script",
  }));
  const length = Buffer.byteLength(code);
  const imported = (offset: number) =>
    offset < 20 ? offset : offset < 20 + importDelta ? 20 : offset - importDelta;
  const composite = Array.from({ length: length + 1 }, (_, index) => {
    const pre = imported(index);
    const row = rows.find((row) => row.generated.start <= pre && pre < row.generated.end);
    return row
      ? [
          row.authored.start + pre - row.generated.start,
          pre - row.generated.start,
          "Some(ScriptSetup)",
        ]
      : null;
  });
  return {
    file: "src/Probe.vue",
    source: first + second,
    code,
    authoredBase: 0,
    rows,
    semanticLinks: [
      {
        source: { start: secondStart + 12 - importDelta, end: secondStart + 17 - importDelta },
        target: { start: firstStart + 6 - importDelta, end: firstStart + 11 - importDelta },
        kind: "Binding",
      },
    ],
    propDefaultKeyLinks: [],
    authoredBlocks: Array(first.length + second.length + 1).fill("ScriptSetup") as string[],
    importMapDebug: "ImportSourceMap { adjustments: [] }",
    importForward: Array.from({ length: length + 1 }, (_, index) => imported(index)),
    importReverse: Array.from({ length: length - importDelta + 1 }, (_, index) =>
      index < 20 ? index : index + importDelta,
    ),
    compositeForward: composite,
  };
}

test("full UTF8 code/maps/vectors preserve a factory edit and a map-comment digit-width change", () => {
  const before = packet(oldFactory),
    after = packet(factoryForward);
  const result = compareFactoryPacket(before, after, oldFactory);
  assert.deepEqual(result, { changed: true, edits: 2 });
  assert.equal(Buffer.byteLength(before.code) - Buffer.byteLength(after.code), 315);
  assert.equal(before.rows[0].generated.start - after.rows[0].generated.start, 314);
  assert.equal(before.rows[1].generated.start - after.rows[1].generated.start, 315);
});

test("the same whole packet law composes a prior three-byte import rewrite", () => {
  assert.deepEqual(
    compareFactoryPacket(packet(oldFactory, 3), packet(factoryForward, 3), oldFactory),
    { changed: true, edits: 2 },
  );
});

test("mapping, ownership, authored/source, unknown metadata and full import-vector mutations refuse", () => {
  const before = packet(oldFactory),
    after = packet(factoryForward);
  const changes: ((value: FactoryPacket) => void)[] = [
    (value) => {
      value.rows[0].authored.start++;
    },
    (value) => {
      value.rows[1].generated.start++;
    },
    (value) => {
      value.rows[0].features = 1;
    },
    (value) => {
      value.semanticLinks[0].target.end++;
    },
    (value) => {
      value.importForward[0]++;
    },
    (value) => {
      value.importReverse.pop();
    },
    (value) => {
      value.compositeForward[0] = [1, 0, "Some(Script)"];
    },
    (value) => {
      value.source += "foreign";
    },
    (value) => {
      Object.assign(value, { foreign: true });
    },
  ];
  for (const change of changes) {
    const poisoned = structuredClone(after);
    change(poisoned);
    assert.throws(() => compareFactoryPacket(before, poisoned, oldFactory));
  }
});

test("missing/duplicate/changed factory code and edits overlapping authored mappings refuse", () => {
  const before = packet(oldFactory),
    after = packet(factoryForward);
  for (const code of [
    after.code + factoryForward,
    after.code.replace(factoryForward, ""),
    after.code.replace(".$RefInstanceFactory<_C>", ".$RefInstanceFactory<any>"),
    after.code + "\nforeign",
  ]) {
    assert.throws(() => factoryCodeEdits(before.code, code, oldFactory, 0));
  }
  const mapped = structuredClone(before);
  mapped.rows[0].generated.start = 9600;
  assert.throws(() => compareFactoryPacket(mapped, after, oldFactory));
  const originalOnly = structuredClone(before);
  originalOnly.compositeForward[9600] = [1, 0, "Some(Script)"];
  assert.throws(() => compareFactoryPacket(originalOnly, after, oldFactory));
  for (const field of ["importForward", "importReverse"] as const) {
    const poisoned = structuredClone(before);
    poisoned[field][9610]++;
    assert.throws(() => compareFactoryPacket(poisoned, after, oldFactory));
  }
});
