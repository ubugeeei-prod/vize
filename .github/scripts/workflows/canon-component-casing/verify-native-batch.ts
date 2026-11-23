import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv, textFile, writeJson, fileSha256 } from "../shared/files.ts";

const root = join(requiredEnv("RUNNER_TEMP"), "component-casing");
const pack = "tests/_fixtures/differential/typechecker/component-tag-aliases-11020/cases.json";
const stdoutFile = join(root, "pinned-batch.stdout");
const stdout = textFile(stdoutFile);
assert.ok(stdout.includes("running 1 test") && stdout.includes("1 passed; 0 failed; 0 ignored;"));
assert.equal(process.env.VIZE_TEST_DISABLE_TSGO, undefined);
const caseIds = readJson<{ cases: { id: string }[] }>(pack).cases.map((entry) => entry.id);
assert.deepEqual(caseIds, ["valid", "invalid", "repaired"]);
writeJson(join(root, "pinned-batch.json"), {
  test: "imported_component_tag_aliases_preserve_batch_prop_diagnostics",
  passed: 1,
  failed: 0,
  ignored: 0,
  caseIds,
  packSha256: fileSha256(pack),
  stdoutSha256: fileSha256(stdoutFile),
  stderrSha256: fileSha256(join(root, "pinned-batch.stderr")),
  nativeDisablePresent: false,
  timingUsable: false,
});
