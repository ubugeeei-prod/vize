import assert from "node:assert/strict";
import { test } from "node:test";

import { buildGitCliffArgs } from "../../tools/release/regenerate-changelog.mjs";

test("changelog regeneration builds git-cliff args from supported flags", () => {
  assert.deepEqual(buildGitCliffArgs([]), ["--config", "cliff.toml", "--output", "CHANGELOG.md"]);
  assert.deepEqual(buildGitCliffArgs(["--unreleased", "--latest"]), [
    "--config",
    "cliff.toml",
    "--output",
    "CHANGELOG.md",
    "--unreleased",
    "--latest",
  ]);
  assert.deepEqual(buildGitCliffArgs(["--tag", "v1.2.3"]), [
    "--config",
    "cliff.toml",
    "--output",
    "CHANGELOG.md",
    "--tag",
    "v1.2.3",
  ]);
});

test("changelog regeneration validates tag and unknown arguments before running git-cliff", () => {
  assert.throws(() => buildGitCliffArgs(["--tag"]), /Missing value for --tag/);
  assert.throws(() => buildGitCliffArgs(["--tag", "--latest"]), /Missing value for --tag/);
  assert.throws(() => buildGitCliffArgs(["--bogus"]), /Unknown argument: --bogus/);
});
