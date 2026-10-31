import assert from "node:assert/strict";
import { mkdirSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { Recorder } from "../../config/windows_consumer/common.ts";
import { assertCustody, inside } from "../computed_event_custody.ts";
import { authenticate } from "./authority.ts";
import { profiles, type Role } from "./contract.ts";
import { install } from "./install.ts";
import { observe } from "./observe.ts";

assert.equal(process.env.GITHUB_ACTIONS, "true");
assert.equal(process.env.GITHUB_REF, "refs/heads/main");
assert.equal(process.version, "v24.14.0");
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
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
  "VIZE_PUBLIC_NATIVE_CUSTODY",
  "VIZE_OXLINT_NATIVE_CUSTODY",
])
  assert.ok(!process.env[name], "initial override forbidden: " + name);
const [beforeBlob, headBlob, output] = process.argv.slice(2);
assert.ok(beforeBlob && headBlob && output && process.env.RUNNER_TEMP);
assert.ok(inside(realpathSync(process.env.RUNNER_TEMP), output));
mkdirSync(output);
const record = new Recorder(realpathSync(output));
const result: Record<string, unknown> = {
  schema: "vize.slotInstance.installedPublic.v1",
  accepted: false,
  profiles,
  consumer: {
    source: process.env.GITHUB_SHA,
    run: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
  },
  capsuleTrust:
    "TP supplies immutable capsules after original producer archive/SRI authentication; this receiver joins those exact receipt bytes to fresh untouched installed images",
  scope:
    "five public NoAny fixes, two observed unknown controls, ordinary default-zero repair and complete 2/8/9/17 preservation surface",
  original27Qualified: false,
  whole11023Qualified: false,
  timingUsable: false,
  roles: [],
};
record.json("qualification.json", result);
let moduleCells = 0,
  semanticCompilerProcesses = 0;
try {
  for (const [role, blob] of [
    ["before", beforeBlob],
    ["head", headBlob],
  ] as const) {
    const roleOutput = join(output, role);
    mkdirSync(roleOutput);
    const roleRecord = new Recorder(roleOutput),
      authority = await authenticate(role satisfies Role, blob, roleRecord);
    const installed = await install(
      join(roleOutput, "registry-installation"),
      role,
      authority,
      roleRecord,
    );
    const observations = observe(role, installed, roleRecord);
    moduleCells += observations.length;
    semanticCompilerProcesses += observations.reduce(
      (sum, cell) =>
        sum +
        cell.targetBad.length +
        (cell.targetFixed?.length ?? 0) +
        cell.preservationBad.length +
        cell.preservationFixed.length,
      0,
    );
    assertCustody(installed.images);
    (result.roles as unknown[]).push({
      role,
      profile: profiles[role],
      capsule: blob,
      actualInstalledGraph: installed.graph,
      observations,
    });
    record.json("qualification.json", result);
  }
  assert.equal(moduleCells, 6);
  assert.equal(semanticCompilerProcesses, 42);
  result.accepted = true;
  result.moduleCells = moduleCells;
  result.semanticCompilerProcesses = semanticCompilerProcesses;
} catch (error) {
  result.error = error instanceof Error ? error.message : String(error);
  throw error;
} finally {
  record.json("qualification.json", result);
}
