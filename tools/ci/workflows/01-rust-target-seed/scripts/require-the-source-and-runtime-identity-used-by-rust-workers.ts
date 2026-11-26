import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
const manifest = JSON.parse(readFileSync("package.json", "utf8"));
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
assert.equal(process.versions.node, manifest.devEngines.runtime.version);
assert.equal(
  execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
  process.env.GITHUB_SHA,
);
