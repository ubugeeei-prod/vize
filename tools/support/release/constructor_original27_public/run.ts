import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { Recorder } from "../../config/windows_consumer/common.ts";
import { assertCustody, inside } from "../computed_event_custody.ts";
import { authenticate } from "./authority.ts";
import { loadOriginal27, fixturePath, assertInputs } from "./fixtures.ts";
import { install } from "./install.ts";
import {
  independentOracles,
  observe,
  repairCopy,
  writeProject,
} from "./observe.ts";

assert.equal(
  process.env.GITHUB_ACTIONS,
  "true",
  "public products run only in Actions",
);
assert.equal(
  process.env.GITHUB_REF,
  "refs/heads/main",
  "merged main workflow required",
);
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
assert.equal(process.version, "v24.14.0");
for (const name of [
  "NODE_OPTIONS",
  "NODE_PATH",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_NATIVE_PATH",
  "VIZE_NATIVE_BINDING_PATH",
  "VIZE_TEST_DISABLE_TSGO",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
])
  assert.ok(!process.env[name], "initial runtime override forbidden: " + name);
const [version, releaseRun, blob, fixtureRoot, output] = process.argv.slice(2);
assert.ok(version && releaseRun && blob && fixtureRoot && output);
assert.ok(process.env.RUNNER_TEMP);
assert.ok(inside(realpathSync(process.env.RUNNER_TEMP), output));
mkdirSync(output);
const record = new Recorder(realpathSync(output));
const result: Record<string, unknown> = {
  schema: "vize.original27.installedPublic.v1",
  status: "RUNNING",
  accepted: false,
  version,
  releaseRun,
  authorityBlob: blob,
  consumerHead: process.env.GITHUB_SHA,
  consumerRun: process.env.GITHUB_RUN_ID,
  consumerAttempt: process.env.GITHUB_RUN_ATTEMPT,
  node: process.version,
  platform: process.platform,
  arch: process.arch,
  scope:
    "original27 authored-error conservation with two separately retained generated mapper debts",
  capsuleTrust:
    "release owner supplies the exact immutable blob after whole producer ZIP authentication; this role joins receipt bytes to installed identity without duplicate archive download",
  umbrella11023Closed: false,
  publicInstanceNoAnyQualified: false,
  timingUsable: false,
};
record.json("qualification.json", result);
try {
  const original = await loadOriginal27(fixtureRoot);
  record.json("original-fixture-custody.json", original.custody);
  record.json("original-manifest.json", original.inputs.manifest);
  copyFileSync(
    join(original.root, fixturePath, "historical-public-summary.json"),
    join(output, "historical-public-summary.json"),
  );
  const authority = await authenticate(version, releaseRun, blob, record);
  result.producer = authority.ownership;
  const installed = await install(
    join(output, "registry-installation"),
    version,
    authority,
    original,
    record,
  );
  const project = join(output, "original27-project");
  writeProject(project, original.inputs.files, installed.modules);
  writeProject(
    join(output, "original58-inputs"),
    original.inputs.files,
    installed.modules,
  );
  independentOracles(record, project, installed);
  // The frozen original population always completes before a separate repair copy is created.
  result.original27 = observe(
    record,
    "original27",
    project,
    original.inputs.files,
    original.inputs.rows,
    installed,
    version,
  );
  record.json("qualification.json", result);
  result.repair = repairCopy(
    record,
    output,
    original.inputs,
    installed,
    version,
  );
  assert.deepEqual(
    assertInputs(project, original.inputs.files),
    original.inputs.manifest,
  );
  assertCustody(original.custody);
  assertCustody(installed.images);
  result.status = "AUTHORED_CONSERVATION_WITH_GENERATED_DEBT";
  result.accepted = true;
  result.generatedDebtCount = 2;
} catch (error) {
  result.status = "FAILED";
  result.error = error instanceof Error ? error.message : String(error);
  throw error;
} finally {
  record.json("qualification.json", result);
}
