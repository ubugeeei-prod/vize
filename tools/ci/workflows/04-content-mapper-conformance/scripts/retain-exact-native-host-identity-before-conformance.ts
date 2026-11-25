import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
const binary = `${process.env.RUNNER_TEMP}/tsgo`;
const upstream = execFileSync("git", ["-C", "typescript-content-mapper", "rev-parse", "HEAD"], {
  encoding: "utf8",
}).trim();
assert.equal(upstream, process.env.CONTENT_MAPPER_TYPESCRIPT_SHA);
const bytes = readFileSync(binary);
const version = spawnSync(binary, ["--version"]);
assert.equal(version.status, 0);
assert.equal(version.signal, null);
assert.equal(version.stderr.length, 0);
const receipt = {
  source: process.env.GITHUB_SHA,
  run: process.env.GITHUB_RUN_ID,
  upstream,
  binary,
  sha256: createHash("sha256").update(bytes).digest("hex"),
  size: bytes.length,
  version: {
    status: version.status,
    stdout: version.stdout.toString("base64"),
    stderr: version.stderr.toString("base64"),
  },
};
writeFileSync(
  `${process.env.RUNNER_TEMP}/content-mapper-native-host.json`,
  JSON.stringify(receipt, null, 2) + "\n",
);
