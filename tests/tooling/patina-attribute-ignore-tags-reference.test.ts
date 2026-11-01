import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { capture } from "./support/patina-attribute-ignore-tags-reference.ts";
import { original, root } from "./support/patina-attribute-ignore-tags-contract.ts";

await test("literal tags retain all eighteen original whole independent verify/fix/requery packets", async () => {
  const wanted = structuredClone(original);
  for (const row of wanted.rows)
    for (const phase of [row.verify, row.fix, row.requery]) {
      assert.equal(phase[0]!.filePath, path.join(wanted.sourceRoot, row.filename));
      phase[0]!.filePath = path.join(root, row.filename);
    }
  wanted.sourceRoot = root;
  assert.deepEqual(await capture(), wanted);
  assert.deepEqual(
    original.rows.map((row) => row.verify[0]!.messages.length),
    [3, 0, 3, 3, 3, 3, 0, 8, 7, 7, 8, 7, 8, 7, 8, 4, 0, 0],
  );
});
