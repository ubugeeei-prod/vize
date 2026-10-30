import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
const capture = process.env.VIZE_NATIVE_DTO_CAPTURE;
const selector =
  "lsp_client::editor_lsp::snapshot_source::tests::native_beta::actual_sdk_semantic_error_repair_navigation_and_two_file_scope_are_complete";
const artifacts = fs
  .readFileSync(path.join(capture, "build.jsonl"), "utf8")
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line))
  .filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.target.name === "vize_canon" &&
      row.profile.test &&
      row.executable,
  );
assert.equal(artifacts.length, 1, "exactly one actual source test executable required");
const executable = artifacts[0].executable;
const hash = createHash("sha256").update(fs.readFileSync(executable)).digest("hex");
const args = [
  "-ff",
  "-ttt",
  "-s",
  "16777216",
  "-xx",
  "-e",
  "trace=read,write,readv,writev,process",
  "-o",
  path.join(capture, "syscalls"),
  executable,
  selector,
  "--ignored",
  "--exact",
  "--nocapture",
];
const stdout = fs.openSync(path.join(capture, "test.stdout"), "w");
const stderr = fs.openSync(path.join(capture, "test.stderr"), "w");
let result;
try {
  result = spawnSync("strace", args, {
    cwd: process.env.VIZE_NATIVE_DTO_SOURCE_ROOT,
    env: process.env,
    stdio: ["ignore", stdout, stderr],
    timeout: 600000,
  });
} finally {
  fs.closeSync(stdout);
  fs.closeSync(stderr);
}
fs.writeFileSync(
  path.join(capture, "source-execution.json"),
  JSON.stringify(
    {
      source: process.env.VIZE_NATIVE_DTO_SOURCE_SHA,
      executable,
      executableSha256: hash,
      command: "strace",
      args,
      status: result.status,
      roleKind:
        process.env.VIZE_NATIVE_DTO_ROLE === "before" &&
        process.env.VIZE_NATIVE_DTO_SDK_VERSION === "7.1.0-beta"
          ? "baseline-observation-known-dto-refusal"
          : process.env.VIZE_NATIVE_DTO_ROLE === "before" &&
              process.env.VIZE_NATIVE_DTO_SDK_VERSION === "7.0.2"
            ? "baseline-observation-registration-boundary"
            : "functional-source-control",
      signal: result.signal,
      error: result.error?.message ?? null,
      note: "Trace the actual source observer externally. Production receives the actual SDK image path, retaining its native image authentication.",
    },
    null,
    2,
  ) + "\n",
);
assert.equal(result.error, undefined);
assert.equal(result.signal, null);
assert.equal(result.status, 0);
