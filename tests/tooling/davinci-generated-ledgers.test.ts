import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const generator = "tools/support/compat/davinci/generated-ledgers.mjs";
const names = [
  "analysis-consumption-summary.md",
  "consumer-migration-summary.md",
  "rule-parity.md",
  "sourcelocation-summary.md",
  "storage-summary.md",
];

test("the run bundle preserves authored guides and rejects edited or extra artifacts", () => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-generated-ledgers-"));
  const guides = ["rule-parity.md", "storage-summary.md"].map((name) => ({
    file: path.join(root, "docs/davinci/plan", name),
    text: fs.readFileSync(path.join(root, "docs/davinci/plan", name), "utf8"),
  }));
  const run = (mode: string) =>
    spawnSync(process.execPath, [generator, mode, "--out-dir", scratch], {
      cwd: root,
      encoding: "utf8",
    });
  try {
    const write = run("--write");
    assert.equal(write.status, 0, `${write.stdout}${write.stderr}`);
    assert.deepEqual(
      fs.readdirSync(scratch).sort((left, right) => (left < right ? -1 : left > right ? 1 : 0)),
      names,
    );
    const clean = run("--check");
    assert.equal(clean.status, 0, `${clean.stdout}${clean.stderr}`);
    fs.appendFileSync(path.join(scratch, "rule-parity.md"), "<!-- injected edit -->\n");
    fs.writeFileSync(path.join(scratch, "retired-report.md"), "stale report\n");
    const stale = run("--check");
    assert.equal(stale.status, 1, "edited or extra run artifact was accepted");
    assert.match(stale.stderr, /rule-parity\.md drifted from the current sources/u);
    assert.match(stale.stderr, /unexpected run artifact.*retired-report\.md/u);
    for (const guide of guides) assert.equal(fs.readFileSync(guide.file, "utf8"), guide.text);
  } finally {
    fs.rmSync(scratch, { recursive: true, force: true });
  }
});
