import assert from "node:assert/strict";
import { test } from "node:test";

import {
  SPAN_FEATURES_ALL,
  sixElementTuples,
} from "../../tools/support/compat/content-mapper/wire-tuples.ts";

void test("five-element spans and ignore directives decode to their TypeScript defaults", () => {
  const message = {
    id: 3,
    result: {
      mappings: [
        [0, 5, 10, 5, 0],
        [5, 1, 15, 1, 1, 0],
      ],
      diagnosticDirectives: {
        directives: [
          [0, 0, 0, 1, 0],
          [1, 2, 3, 4, 1, 0],
        ],
        unusedExpectDirectiveDiagnostics: [],
      },
    },
  };
  assert.deepEqual(sixElementTuples(message).result, {
    mappings: [
      [0, 5, 10, 5, 0, SPAN_FEATURES_ALL],
      [5, 1, 15, 1, 1, 0],
    ],
    diagnosticDirectives: {
      directives: [
        [0, 0, 0, 1, 0, 0],
        [1, 2, 3, 4, 1, 0],
      ],
      unusedExpectDirectiveDiagnostics: [],
    },
  });
  assert.equal(SPAN_FEATURES_ALL, 1048575);
});

void test("messages without a transform result are unchanged", () => {
  for (const message of [{ id: 1, result: null }, { id: 2, error: { code: 1 } }, null]) {
    assert.deepEqual(sixElementTuples(structuredClone(message)), message);
  }
});
