import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const capture = process.env.VIZE_IMPORTED_RAW_CAPTURE;
const name = process.argv[2];
if (!capture || !name)
  throw new Error("Expected a callback test name and VIZE_IMPORTED_RAW_CAPTURE");

const stdout = readFileSync(path.join(capture, "test.stdout"));
const stderr = readFileSync(path.join(capture, "test.stderr"));
const status = Number(readFileSync(path.join(capture, "test.status"), "utf8").trim());
const executed = Number(
  /^test result: ok\. 1 passed; 0 failed; 0 ignored; /m.test(stdout.toString("utf8")),
);
const receipt = {
  source: execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
  run: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT,
  name,
  status,
  executed,
  stdoutSha256: createHash("sha256").update(stdout).digest("hex"),
  stderrSha256: createHash("sha256").update(stderr).digest("hex"),
  historical118Resolved: false,
  nativeClean: false,
  nativeKnownDebt: true,
  timingUsable: false,
};
writeFileSync(path.join(capture, "test-run.json"), JSON.stringify(receipt, null, 2) + "\n");
if (status !== 0) process.exitCode = status;
else if (executed !== 1) throw new Error("derived callback fixture did not execute exactly once");
