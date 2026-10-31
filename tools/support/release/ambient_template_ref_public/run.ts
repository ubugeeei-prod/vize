import assert from "node:assert/strict";
import { mkdirSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { Recorder } from "../../config/windows_consumer/common.ts";
import { assertCustody, inside } from "../computed_event_custody.ts";
import { authenticate, release } from "./authority.ts";
import { install } from "./install.ts";
import { loadFixture, observe } from "./observe.ts";

assert.equal(process.env.GITHUB_ACTIONS, "true");
assert.equal(process.env.GITHUB_EVENT_NAME, "workflow_dispatch");
assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
assert.equal(process.env.GITHUB_REF, "refs/heads/main");
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
assert.equal(process.version, "v24.14.0");
for (const key of [
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
  assert.ok(!process.env[key], "initial runtime override forbidden: " + key);
const [version, releaseRun, blob, output] = process.argv.slice(2);
assert.ok(version && releaseRun && blob && output && process.argv.length === 6);
assert.ok(process.env.RUNNER_TEMP);
assert.ok(inside(realpathSync(process.env.RUNNER_TEMP), output));
mkdirSync(output);
const record = new Recorder(realpathSync(output));
const result: Record<string, unknown> = {
  schema: "vize.ambientTemplateRef.installedPublic.v1",
  status: "RUNNING",
  accepted: false,
  role: "C12-original-10980",
  ...release,
  authorityBlob: blob,
  consumerHead: process.env.GITHUB_SHA,
  consumerRun: process.env.GITHUB_RUN_ID,
  consumerAttempt: process.env.GITHUB_RUN_ATTEMPT,
  scope:
    "four original ambient declaration CLI phases only; no installed mapper, independent oracle, or future ten-declaration role",
  timingUsable: false,
  capsuleTrust:
    "release owner supplies an immutable capsule after whole producer ZIP authentication; this receiver joins genuine receipts to untouched installed images",
};
record.json("qualification.json", result);
try {
  record.json("original-inputs.json", loadFixture());
  const authority = await authenticate(version, releaseRun, blob, record);
  const installed = await install(join(output, "registry-installation"), authority, record);
  result.observations = observe(installed, output, record);
  assertCustody(installed.images);
  result.status = "FOUR_PHASES_ACCEPTED";
  result.accepted = true;
  result.actualPublicCliCommands = 4;
} catch (error) {
  result.status = "FAILED";
  result.error = error instanceof Error ? error.message : String(error);
  throw error;
} finally {
  record.json("qualification.json", result);
}
