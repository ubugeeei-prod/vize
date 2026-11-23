import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
const git = (...args: string[]) => execFileSync("git", args, { encoding: "utf8" }).trim();
const source = git("rev-parse", "HEAD");
assert.equal(source, process.env.PRODUCER_SOURCE);
const tagCommit = git("rev-parse", `refs/tags/${process.env.RELEASE_TAG}^{commit}`);
git("merge-base", "--is-ancestor", source, tagCommit);
if (process.env.REQUIRED_SOURCE)
  git("merge-base", "--is-ancestor", process.env.REQUIRED_SOURCE, source);
const file = path.join(
  requiredEnv("RUNNER_TEMP"),
  "typescript-provider-public/public-download.json",
);
const receipt = JSON.parse(fs.readFileSync(file, "utf8"));
fs.writeFileSync(
  file,
  JSON.stringify({ ...receipt, tagCommit, checkedOutSource: source }, null, 2),
);
