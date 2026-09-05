import assert from "node:assert/strict";
import { test } from "node:test";

import { HOST_TEST_REFERENCES_COMMAND } from "../../editors/vscode/src/host-test-core.ts";
import {
  HOST_TEST_REFERENCES_COMMAND as suiteHostReferencesCommand,
  assertRealHostReferences,
} from "../../editors/vscode/test/real-host-references-oracle.mjs";

test("the real host references oracle pins authored SFC locations", () => {
  assert.equal(HOST_TEST_REFERENCES_COMMAND, suiteHostReferencesCommand);
  const uri = "file:///workspace/src/App.vue";
  const references = [
    location(uri, 3, 6, 3, 11),
    location(uri, 9, 19, 9, 24),
    location(uri, 10, 10, 10, 15),
  ];

  assert.deepEqual(assertRealHostReferences(references, { uri }), [
    { range: [3, 6, 3, 11], uri },
    { range: [9, 19, 9, 24], uri },
    { range: [10, 10, 10, 15], uri },
  ]);
  assert.throws(() => assertRealHostReferences(null, { uri }), /must return locations/);
  assert.throws(
    () => assertRealHostReferences(references.slice(1), { uri }),
    /must include the script declaration/,
  );
  assert.throws(
    () => assertRealHostReferences([...references, location(`${uri}.ts`, 0, 0, 0, 5)], { uri }),
    /generated virtual document/,
  );
});

function location(
  uri: string,
  startLine: number,
  startChar: number,
  endLine: number,
  endChar: number,
) {
  return {
    range: {
      end: { character: endChar, line: endLine },
      start: { character: startChar, line: startLine },
    },
    uri,
  };
}
