import assert from "node:assert/strict";
import {
  appendFileSync,
  copyFileSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  realpathSync,
  rmSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const checkout = realpathSync(fileURLToPath(new URL("../../../", import.meta.url)));
const { GITHUB_WORKSPACE, RUNNER_TEMP, GITHUB_ENV } = process.env;
assert.ok(GITHUB_WORKSPACE && RUNNER_TEMP && GITHUB_ENV, "GitHub runner paths are required");
const workspace = realpathSync(GITHUB_WORKSPACE);
assert.equal(checkout, path.join(workspace, ".vize-workflow-tools"));

// A workflow's programs live in tools/ci/workflows/<name>/scripts/ (or, until every
// workflow moves there, .github/scripts/workflows/<name>/); shared ones in tools/ci/lib/scripts/.
function owners(parent: string, nested: string): string[] {
  const directory = path.join(checkout, parent);
  if (!existsSync(directory)) return [];
  assert.equal(realpathSync(directory), directory);
  return readdirSync(directory, { withFileTypes: true }).map((entry) => {
    assert.ok(entry.isDirectory() && /^[a-z0-9-]+$/u.test(entry.name), entry.name);
    return path.posix.join(parent, entry.name, nested);
  });
}
const programs = [
  ...owners("tools/ci/workflows", "scripts"),
  ...owners(".github/scripts/workflows", ""),
];
assert.equal(programs.length, 1, "the sparse checkout must contain exactly one workflow");
const [owner] = programs;

// Later source checkouts can replace every file under GITHUB_WORKSPACE. The staged copy
// keeps the repository layout, so relative references between programs still resolve.
const directory = mkdtempSync(path.join(realpathSync(RUNNER_TEMP), "vize-workflow-scripts-"));
for (const relative of [owner!, "tools/ci/lib/scripts"]) {
  const source = path.join(checkout, relative);
  assert.ok(lstatSync(source).isDirectory(), "expected workflow script directory");
  const target = path.join(directory, relative);
  mkdirSync(target, { recursive: true });
  for (const file of readdirSync(source, { withFileTypes: true })) {
    assert.ok(file.isFile(), "workflow script must be a regular file: " + file.name);
    copyFileSync(path.join(source, file.name), path.join(target, file.name));
  }
}
appendFileSync(GITHUB_ENV, "VIZE_CI_FILES=" + path.join(directory, owner!) + "\n");
rmSync(checkout, { recursive: true });
