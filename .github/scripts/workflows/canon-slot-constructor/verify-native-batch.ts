import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv, textFile, writeJson, fileSha256 } from "../shared/files.ts";

const root = join(requiredEnv("RUNNER_TEMP"), "slot-evidence");
const pack =
  "tests/_fixtures/differential/typechecker/zero-argument-slot-constructor-11013/cases.json";
const stdoutFile = join(root, "fix-history.stdout");
const stdout = textFile(stdoutFile);
assert.ok(stdout.includes("running 1 test") && stdout.includes("1 passed; 0 failed; 0 ignored;"));

const caseIds = readJson<{ cases: { id: string }[] }>(pack).cases.map((entry) => entry.id);
assert.deepEqual(caseIds, [
  "zero-argument-valid",
  "generic-default-zero-valid",
  "slot-string-invalid",
]);
writeJson(join(root, "fix-history.json"), {
  test: "zero_argument_slot_constructors_preserve_mapper_and_authored_diagnostics",
  passed: 1,
  failed: 0,
  ignored: 0,
  caseIds,
  packSha256: fileSha256(pack),
  stdoutSha256: fileSha256(stdoutFile),
  stderrSha256: fileSha256(join(root, "fix-history.stderr")),
  nativeDisablePresent: false,
  nativeRequired: true,
  timingUsable: false,
});
