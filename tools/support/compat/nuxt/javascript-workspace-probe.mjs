// This child runs under the existing scoped native preload; no ambient SDK/compiler substitution.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { runtime } from "./javascript-workspace-runtime.mjs";
import { cliViteRuntime } from "./javascript-workspace-cli-runtime.mjs";
import { cliProducts, sourceCli } from "./javascript-workspace-cli.mjs";
import { editorProducts } from "../../../../tests/tooling/support/lsp/javascript-workspace.ts";
import { files, save, sha } from "./javascript-workspace-project.mjs";
const [root, serialized] = process.argv.slice(2);
const context = JSON.parse(serialized);
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
assert.equal(process.env.VIZE_NUXT_NATIVE_CUSTODY, context.bindingFile);
context.environment = { ...process.env };
const requested = path.join(root, "node_modules/@typescript/typescript-linux-x64");
const packageRoot = fs.realpathSync(requested);
const sdk = fs.realpathSync(path.join(packageRoot, "lib"));
const provider = fs.realpathSync(path.join(sdk, "tsc"));
assert.equal(path.dirname(provider), sdk);
assert.equal(path.dirname(sdk), packageRoot);
const manifest = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json")));
assert.equal(manifest.name, "@typescript/typescript-linux-x64");
assert.equal(manifest.version, "7.0.2");
const members = files(packageRoot);
assert.equal(members["lib/tsc"].sha256, sha(fs.readFileSync(provider)));
const probe = spawnSync(provider, ["--version"], { encoding: "utf8", timeout: 30_000 });
save(context.artifacts, "native-provider.json", {
  requested,
  packageRoot,
  sdk,
  provider,
  manifest,
  members,
  probe: {
    status: probe.status,
    signal: probe.signal,
    stdout: probe.stdout,
    stderr: probe.stderr,
    error: probe.error?.message ?? null,
  },
  lock: fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8"),
});
assert.equal(probe.error, undefined);
assert.equal(probe.signal, null);
assert.equal(probe.status, 0);
assert.match(probe.stdout, /7\.0\.2/);
process.env.CORSA_PATH = provider;
const failures = [];
const remember = (stage, error) =>
  failures.push({
    stage,
    error: error instanceof Error ? (error.stack ?? error.message) : String(error),
  });
let runtimeResult, cliResult;
try {
  runtimeResult = await runtime(root, context);
} catch (error) {
  remember("framework-runtime", error);
}
try {
  cliResult = await cliProducts(root, context, provider);
} catch (error) {
  remember("whole-cli-products", error);
}
if (context.cohort.id === "vite" && cliResult)
  try {
    await cliViteRuntime(root, context, cliResult.compiled);
  } catch (error) {
    remember("cli-executable-runtime", error);
  }
try {
  await editorProducts(root, context, sourceCli(root, context.artifacts));
} catch (error) {
  remember("whole-editor-products", error);
}
save(context.artifacts, "proof.json", {
  source: context.binding.source,
  cohort: context.cohort,
  runtimeResult,
  cliResult,
  provider,
  failures,
  scope: context.corpus.scope,
});
assert.deepEqual(failures, [], "every required tool remains a terminal obligation");
