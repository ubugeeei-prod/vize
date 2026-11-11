import assert from "node:assert/strict";
import { test } from "node:test";

import { authoredDirectives } from "../../tools/support/compat/content-mapper/producer-unmapped-ignores.ts";

const text = "ab cd";
const mappings = [[0, 2, 0, 2, 1, 0]];
const authored = [1, 1, 0, 2, 1, 0];
const gap = [0, 0, 2, 5, 0, 0];

function packet(directives: number[][]) {
  return {
    text,
    mappings,
    diagnosticDirectives: {
      directives,
      unusedExpectDirectiveDiagnostics: [],
    },
  };
}

test("the producer's unmapped IGNORE rows are accepted", () => {
  assert.deepEqual(
    authoredDirectives(packet([authored])),
    authoredDirectives(packet([authored, gap]), true),
  );
  assert.deepEqual(
    authoredDirectives(packet([authored])),
    authoredDirectives(packet([gap, authored]), true),
  );
});

test("whitespace-only producer gaps may be retained internally or omitted from the CLI wire", () => {
  const whitespace = {
    text: "a \n\tb",
    mappings: [
      [0, 1, 0, 1, 1, 0],
      [4, 1, 4, 1, 1, 0],
    ],
  };
  assert.equal(authoredDirectives(whitespace, true), null);
  assert.equal(
    authoredDirectives(
      {
        ...whitespace,
        diagnosticDirectives: {
          directives: [[0, 0, 1, 4, 0, 0]],
          unusedExpectDirectiveDiagnostics: [],
        },
      },
      true,
    ),
    null,
  );
});

test("an IGNORE over a mapped range is rejected", () => {
  assert.throws(() => authoredDirectives(packet([[0, 0, 0, 2, 0, 0]]), true), /unmapped gaps/);
});

test("a widened unmapped gap is rejected", () => {
  assert.throws(() => authoredDirectives(packet([[0, 0, 1, 5, 0, 0]]), true), /unmapped gaps/);
});

test("an extra IGNORE tuple is rejected", () => {
  assert.throws(() => authoredDirectives(packet([gap, [0, 0, 2, 4, 0, 0]]), true), /unmapped gaps/);
});

test("a current head that drops the producer population is rejected", () => {
  assert.throws(() => authoredDirectives(packet([authored]), true), /unmapped gaps/);
});

test("an authored directive change is still rejected", () => {
  const before = authoredDirectives(packet([authored]));
  const after = authoredDirectives(packet([[1, 1, 0, 2, 2, 0], gap]), true);
  assert.notDeepEqual(before, after);
});

const navigation =
  "a\n  const __vize_props_nav_0 = undefined as unknown as P;\n  // @ts-ignore Navigation-only references; no diagnostic owner.\n  void __vize_props_nav_0.k; void __vize_props_nav_0['k-2'];\nb";

/** Spans for `a`, both keys and `b`. */
function spans(source: string): [number, number][] {
  return [
    [0, 1],
    [source.indexOf(".k;") + 1, 1],
    [source.indexOf("k-2"), 3],
    [source.length - 1, 1],
  ];
}

function navigationPacket(source: string, directives: number[][]) {
  return {
    text: source,
    mappings: spans(source).map(([start, length]) => [start, length, 0, 1, 1, 0]),
    diagnosticDirectives: { directives, unusedExpectDirectiveDiagnostics: [] },
  };
}

test("a navigation block and its gaps may share one IGNORE", () => {
  const end = navigation.length - 1;
  assert.equal(
    authoredDirectives(navigationPacket(navigation, [[0, 0, 1, end, 0, 0]]), true),
    null,
  );
  const [, [k], [k2]] = spans(navigation);
  const legacy = [
    [0, 0, 1, k!, 0, 0],
    [0, 0, k! + 1, k2!, 0, 0],
    [0, 0, k2! + 3, end, 0, 0],
  ];
  assert.equal(authoredDirectives(navigationPacket(navigation, legacy), true), null);
});

test("one IGNORE over references the comment does not cover is rejected", () => {
  for (const source of [
    navigation.replace("// @ts-ignore Navigation-only", "// Navigation-only"),
    navigation.replace("; void __vize_props_nav_0['k-2']", "; void other['k-2']"),
    navigation.replace("__vize_props_nav_0 =", "__vize_props_nav_x ="),
  ]) {
    const end = source.length - 1;
    assert.throws(
      () => authoredDirectives(navigationPacket(source, [[0, 0, 1, end, 0, 0]]), true),
      /unmapped gaps/,
      source,
    );
  }
});
