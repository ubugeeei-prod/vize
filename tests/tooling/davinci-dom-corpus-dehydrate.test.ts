import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { dehydrateCorpus } from "../../legacy-tools/fixtures/davinci-dom-corpus-workflow.mjs";

test("S2 DOM corpus dehydrate unregisters selected fixture gitlinks", () => {
  const artifact = mkdtempSync(join(tmpdir(), "vize-dom-corpus-"));
  const calls = [];
  const runCommand = (command, args) => {
    calls.push([command, args]);
    return "";
  };
  try {
    writeFileSync(
      join(artifact, "selected-gitlinks.txt"),
      "tests/_fixtures/_git/element\ntests/_fixtures/_git/primevue\n",
    );
    assert.equal(dehydrateCorpus({ artifact, runCommand }), 0);
    assert.deepEqual(calls, [
      [
        "git",
        [
          "submodule",
          "deinit",
          "--force",
          "--",
          "tests/_fixtures/_git/element",
          "tests/_fixtures/_git/primevue",
        ],
      ],
    ]);
  } finally {
    rmSync(artifact, { recursive: true, force: true });
  }
});
