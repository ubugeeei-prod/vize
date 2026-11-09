import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const corpusCoverage = path.join(root, "tools/commands/davinci/corpus-coverage.ts");

test("corpus coverage maintenance uses the stable Node command", () => {
  const result = spawnSync(process.execPath, [corpusCoverage, "--check"], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /corpus-coverage\.md is up to date/);

  const usage = spawnSync(process.execPath, [corpusCoverage], { cwd: root, encoding: "utf8" });
  assert.equal(usage.status, 1);
  assert.match(usage.stderr, /node tools\/commands\/davinci\/corpus-coverage\.ts/);
});
