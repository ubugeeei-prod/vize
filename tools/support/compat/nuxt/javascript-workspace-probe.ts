// This child runs under the existing scoped native preload; no ambient SDK/compiler substitution.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { runtime } from "./javascript-workspace-runtime.ts";
import { cliViteRuntime } from "./javascript-workspace-cli-runtime.ts";
import { cliProducts, sourceCli } from "./javascript-workspace-cli.ts";
import { editorProducts } from "../../../../tests/tooling/support/lsp/javascript-workspace.ts";
import { asRecord, parseJson, readJson } from "../../json-value.ts";
import { files, parseWorkspaceContext, save, sha } from "./javascript-workspace-project.ts";
const [root, serialized] = process.argv.slice(2);
assert.ok(root && serialized, "usage: javascript-workspace-probe.ts ROOT CONTEXT_JSON");
const context = parseWorkspaceContext(parseJson(serialized), { ...process.env });
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
assert.equal(process.env.VIZE_NUXT_NATIVE_CUSTODY, context.bindingFile);
const requested = path.join(root, "node_modules/@typescript/typescript-linux-x64");
const packageRoot = fs.realpathSync(requested);
const sdk = fs.realpathSync(path.join(packageRoot, "lib"));
const provider = fs.realpathSync(path.join(sdk, "tsc"));
assert.equal(path.dirname(provider), sdk);
assert.equal(path.dirname(sdk), packageRoot);
const manifest = asRecord(readJson(path.join(packageRoot, "package.json")), "provider manifest");
assert.equal(manifest.name, "@typescript/typescript-linux-x64");
assert.equal(manifest.version, "7.0.2");
const members = files(packageRoot);
assert.equal(members["lib/tsc"]?.sha256, sha(fs.readFileSync(provider)));
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
const failures: { stage: string; error: string }[] = [];
const remember = (stage: string, error: unknown) =>
  failures.push({
    stage,
    error: error instanceof Error ? (error.stack ?? error.message) : String(error),
  });
let runtimeResult: unknown, cliResult: unknown;
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
const compiled = path.join(context.artifacts, "compiled-whole-packets.json");
if (context.cohort.id === "vite" && fs.existsSync(compiled))
  try {
    await cliViteRuntime(root, context, readJson(compiled));
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
