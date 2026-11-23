import assert from "node:assert/strict";
import { readFileSync, writeFileSync, realpathSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
const upstream = execFileSync("git", ["-C", "typescript-content-mapper", "rev-parse", "HEAD"], {
  encoding: "utf8",
}).trim();
assert.equal(upstream, "d6c4afddb2c55f4a9dea7b59293a99a8fdea1799");
const binary = realpathSync(`${process.env.RUNNER_TEMP}/instance-globals-source-host/tsgo`);
const bytes = readFileSync(binary);
const result = spawnSync(binary, ["--version"], { encoding: "utf8" });
assert.equal(result.status, 0);
assert.equal(result.stderr, "");
const receipt = {
  schema: "vize.instance-globals-source-host",
  version: 1,
  repository: "microsoft/TypeScript",
  upstream,
  source: execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
  run: process.env.GITHUB_RUN_ID,
  binary,
  sha256: createHash("sha256").update(bytes).digest("hex"),
  size: bytes.length,
  compilerVersion: {
    status: result.status,
    signal: result.signal,
    error: null,
    stdout: result.stdout,
    stderr: result.stderr,
  },
};
writeFileSync(
  `${process.env.RUNNER_TEMP}/instance-globals-10617/source-host.json`,
  JSON.stringify(receipt, null, 2) + "\n",
);
