import assert from "node:assert/strict";
import {
  appendFileSync,
  copyFileSync,
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
const scripts = path.join(checkout, ".github/scripts/workflows");
assert.equal(realpathSync(scripts), scripts);
const owners = readdirSync(scripts, { withFileTypes: true }).filter(
  (entry) => entry.name !== "shared",
);
assert.equal(owners.length, 1, "the sparse checkout must contain exactly one workflow");
const [owner] = owners;
assert.ok(owner.isDirectory() && /^[a-z0-9-]+$/u.test(owner.name));

// Later source checkouts can replace every file under GITHUB_WORKSPACE.
const directory = mkdtempSync(path.join(realpathSync(RUNNER_TEMP), "vize-workflow-scripts-"));
for (const name of [owner.name, "shared"]) {
  const source = path.join(scripts, name);
  assert.ok(lstatSync(source).isDirectory(), "expected workflow script directory");
  const target = path.join(directory, name);
  mkdirSync(target);
  for (const file of readdirSync(source, { withFileTypes: true })) {
    assert.ok(file.isFile(), "workflow script must be a regular file: " + file.name);
    copyFileSync(path.join(source, file.name), path.join(target, file.name));
  }
}
appendFileSync(GITHUB_ENV, "VIZE_CI_FILES=" + path.join(directory, owner.name) + "\n");
rmSync(checkout, { recursive: true });
