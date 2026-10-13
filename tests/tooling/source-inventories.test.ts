import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { runLanes } from "../../tools/support/compat/davinci/source-inventories.ts";

await test("parallel source preflights keep lane order and fail on any lane", async () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-source-inventories-"));
  try {
    const results = await runLanes(
      [
        [["-e", "process.stdout.write('first ok'); process.exit(0)"]],
        [["-e", "process.stderr.write('second failed'); process.exit(3)"]],
        [
          ["-e", "process.exit(4)"],
          ["-e", "require('node:fs').writeFileSync('must-not-run', '')"],
        ],
        [
          ["-e", "require('node:fs').writeFileSync('written', 'ledger')"],
          ["-e", "process.stdout.write(require('node:fs').readFileSync('written', 'utf8'))"],
        ],
      ],
      cwd,
    );
    assert.deepEqual(
      results.map(({ status }) => status),
      [0, 3, 4, 0],
    );
    assert.match(results[0]?.output ?? "", /first ok/);
    assert.match(results[1]?.output ?? "", /second failed/);
    // A dependent command never runs after its lane failed.
    assert.equal(existsSync(join(cwd, "must-not-run")), false);
    // Commands within a lane stay ordered: the check reads what the write produced.
    assert.match(results[3]?.output ?? "", /ledger$/);
    assert.equal(readFileSync(join(cwd, "written"), "utf8"), "ledger");
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
