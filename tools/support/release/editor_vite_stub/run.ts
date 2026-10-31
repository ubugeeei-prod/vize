import assert from "node:assert/strict";
import { mkdirSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { Recorder } from "../../config/windows_consumer/common.ts";
import { authenticate } from "./authority.ts";
import { install } from "./install.ts";
import { configAcceptance } from "./config.ts";
import { editorAcceptance } from "./editor.ts";

assert.equal(
  process.env.GITHUB_ACTIONS,
  "true",
  "public installs and product executions belong to Actions",
);
assert.equal(process.version, "v26.6.0", "reported Node 26 host, pinned for all three platforms");
for (const key of [
  "NODE_OPTIONS",
  "NODE_PATH",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_NATIVE_PATH",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "VIZE_CONFIG_CACHE",
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
])
  assert.ok(!process.env[key], "initial runtime override must be empty: " + key);
const [version, source, blob, output] = process.argv.slice(2);
assert.ok(version && source && blob && output);
mkdirSync(output, { recursive: false });
const record = new Recorder(realpathSync(output));
const authority = authenticate(version, source, blob, record);
const project = join(record.root, "consumer"),
  tooling = join(record.root, "package-manager");
const installed = install(project, tooling, version, authority, record);
configAcceptance(realpathSync(project), installed.identity, installed.original, record);
await editorAcceptance(realpathSync(project), installed.identity, installed.original, record);
record.json("qualification.json", {
  schema: "vize.editor-vite-stub-public.v1",
  version,
  source,
  workflowSource: process.env.GITHUB_SHA,
  runId: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT,
  platform: process.platform,
  arch: process.arch,
  node: process.version,
  authorityBlob: blob,
  identity: installed.identity,
  config: "all five phases passed",
  editor: "two deliberate stale whole-link recovery cycles passed",
  limitations: [
    "Typescript 6.0.3 and pnpm 12.1.0 pin hosts that were unspecified in the original report",
    "Fresh original root-Vite-absent topology required, never manufactured by deleting a real dependency",
    "No claim that the minimal fixture naturally creates the old whole-install link",
    "Producer receipt bytes and origins come from the authenticated TP capsule; no duplicate producer archive download",
  ],
});
