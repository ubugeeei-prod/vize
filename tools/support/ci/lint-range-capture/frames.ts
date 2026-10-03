import assert from "node:assert/strict";
import { lstatSync, readdirSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { directory, existingJson, sha256 } from "./common.ts";
import { encodeFrames } from "./codec.ts";
import { historicalInvalidCount, originalRun, originalSource } from "./pins.ts";

const [rootArg, expectedSource] = process.argv.slice(2);
assert.ok(rootArg && /^[0-9a-f]{40}$/u.test(expectedSource ?? ""));
const root = resolve(rootArg),
  out = directory(root);
function inventory(path: string): { path: string; bytes: Buffer }[] {
  assert.equal(lstatSync(path).isSymbolicLink(), false, "Never follow output symlinks");
  if (lstatSync(path).isDirectory())
    return readdirSync(path)
      .sort()
      .flatMap((name) => inventory(join(path, name)));
  assert.ok(lstatSync(path).isFile());
  return [{ path: relative(out, path), bytes: readFileSync(path) }];
}
const verification = existingJson(root, "report-verification.json"),
  exit = existingJson(root, "process-exit.json");
const files = inventory(out);
if (verification?.complete) {
  assert.equal(verification.expectedSource, expectedSource);
  for (const row of [...verification.reports, verification.summary]) {
    const file = files.find((file) => file.path === relative(out, row.path));
    assert.ok(file, "Missing verified original artifact");
    assert.equal(sha256(file.bytes), row.sha256, "Original bytes changed after verification");
  }
}
if (exit) assert.equal(exit.expectedSource, expectedSource);
console.log(
  "Original run comparison only: " +
    JSON.stringify({ originalRun, originalSource, historicalInvalidCount, fixedFreshCount: false }),
);
for (const line of encodeFrames(
  {
    source: expectedSource,
    runId: process.env.GITHUB_RUN_ID ?? null,
    attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
    actualProcessExit: exit?.status ?? null,
    actualSignal: exit?.signal ?? null,
    complete: verification?.complete === true,
    acceptance: false,
    files: files.length,
  },
  files,
))
  console.log(line);
