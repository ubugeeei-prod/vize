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
